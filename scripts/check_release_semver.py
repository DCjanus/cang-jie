#!/usr/bin/env python3
"""Block incompatible stable releases before publishing to crates.io."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import tempfile
import textwrap
import tomllib
from dataclasses import dataclass
from pathlib import Path


STABLE_VERSION_RE = re.compile(r"^v?(\d+)\.(\d+)\.(\d+)$")
VERSION_RE = re.compile(r"^v?(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$")


@dataclass(frozen=True, order=True)
class Version:
    major: int
    minor: int
    patch: int
    stable: bool = True

    @classmethod
    def parse(cls, value: str) -> "Version":
        stable_match = STABLE_VERSION_RE.match(value)
        if stable_match:
            return cls(*(int(part) for part in stable_match.groups()), stable=True)

        match = VERSION_RE.match(value)
        if not match:
            raise ValueError(f"unsupported version format: {value}")
        return cls(*(int(part) for part in match.groups()), stable=False)

    def is_compatible_line_with(self, other: "Version") -> bool:
        if not self.stable or not other.stable:
            return False
        if self.major == 0:
            return self.minor != 0 and other.major == 0 and other.minor == self.minor
        return other.major == self.major


def run(args: list[str], *, cwd: Path, capture: bool = False) -> str:
    result = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else ""


def cargo_package_version(repo: Path) -> str:
    metadata = run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=repo,
        capture=True,
    )
    import json

    return json.loads(metadata)["packages"][0]["version"]


def stable_tags(repo: Path) -> list[tuple[Version, str]]:
    tags = run(["git", "tag", "--list", "v*"], cwd=repo, capture=True).splitlines()
    versions = []
    for tag in tags:
        try:
            version = Version.parse(tag)
        except ValueError:
            continue
        if version.stable:
            versions.append((version, tag))
    return sorted(versions)


def compatible_baseline_tag(
    current: Version, tags: list[tuple[Version, str]]
) -> str | None:
    candidates = [
        (version, tag)
        for version, tag in tags
        if version < current and current.is_compatible_line_with(version)
    ]
    return candidates[-1][1] if candidates else None


def baseline_jieba_requirement(repo: Path, baseline_tag: str) -> str | None:
    cargo_toml = run(
        ["git", "show", f"{baseline_tag}:Cargo.toml"], cwd=repo, capture=True
    )
    manifest = tomllib.loads(cargo_toml)
    dependency = manifest.get("dependencies", {}).get("jieba-rs")
    if isinstance(dependency, str):
        return dependency
    if isinstance(dependency, dict):
        version = dependency.get("version")
        return str(version) if version else None
    return None


def run_public_dependency_smoke(repo: Path, baseline_tag: str) -> None:
    jieba_requirement = baseline_jieba_requirement(repo, baseline_tag)
    if not jieba_requirement:
        print(
            "No baseline jieba-rs dependency found; skipping public dependency smoke test."
        )
        return

    print(
        "Checking documented CangJieTokenizer construction against "
        f"baseline jieba-rs requirement {jieba_requirement!r}."
    )
    with tempfile.TemporaryDirectory(prefix="cang-jie-public-dep-") as tmp:
        crate = Path(tmp)
        (crate / "src").mkdir()
        (crate / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""\
                [package]
                name = "cang-jie-public-dependency-smoke"
                version = "0.0.0"
                edition = "2024"

                [dependencies]
                cang-jie = {{ path = {str(repo)!r} }}
                jieba-rs = {jieba_requirement!r}
                """
            ),
            encoding="utf-8",
        )
        (crate / "src/lib.rs").write_text(
            textwrap.dedent(
                """\
                use std::sync::Arc;

                use cang_jie::{CangJieTokenizer, TokenizerOption};
                use jieba_rs::Jieba;

                pub fn tokenizer() -> CangJieTokenizer {
                    CangJieTokenizer {
                        worker: Arc::new(Jieba::new()),
                        option: TokenizerOption::Default { hmm: false },
                    }
                }
                """
            ),
            encoding="utf-8",
        )
        run(["cargo", "check", "--manifest-path", str(crate / "Cargo.toml")], cwd=repo)


def check_release(repo: Path, tag_name: str) -> None:
    tag_version = tag_name.removeprefix("v")
    crate_version = cargo_package_version(repo)
    if tag_version != crate_version:
        raise SystemExit(
            f"release tag {tag_name} does not match Cargo.toml version {crate_version}"
        )

    current = Version.parse(tag_name)
    if not current.stable:
        print(f"{tag_name} is a pre-release tag; skipping strict SemVer gate.")
        return

    baseline_tag = compatible_baseline_tag(current, stable_tags(repo))
    if baseline_tag is None:
        print(
            f"{tag_name} starts a new effective-major release line; "
            "breaking changes are allowed."
        )
        return

    print(f"Checking SemVer compatibility against baseline {baseline_tag}.")
    run(
        ["cargo", "semver-checks", "check-release", "--baseline-rev", baseline_tag],
        cwd=repo,
    )
    run_public_dependency_smoke(repo, baseline_tag)


def self_test() -> None:
    tags = [
        (Version.parse("v0.19.0"), "v0.19.0"),
        (Version.parse("v0.20.0"), "v0.20.0"),
        (Version.parse("v1.2.3"), "v1.2.3"),
        (Version.parse("v1.3.0"), "v1.3.0"),
    ]
    assert compatible_baseline_tag(Version.parse("v0.20.1"), tags) == "v0.20.0"
    assert compatible_baseline_tag(Version.parse("v0.21.0"), tags) is None
    assert compatible_baseline_tag(Version.parse("v1.3.1"), tags) == "v1.3.0"
    assert compatible_baseline_tag(Version.parse("v1.4.0"), tags) == "v1.3.0"
    assert not Version.parse("v0.20.0-alpha.1").stable
    print("self-test passed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--tag", default=os.environ.get("GITHUB_REF_NAME"))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return
    if not args.tag:
        raise SystemExit("--tag or GITHUB_REF_NAME is required")
    if shutil.which("cargo-semver-checks") is None:
        raise SystemExit("cargo-semver-checks is required")

    check_release(args.repo.resolve(), args.tag)


if __name__ == "__main__":
    main()
