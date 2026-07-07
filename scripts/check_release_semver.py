#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "rich>=15.0.0",
#     "semver>=3.0.4",
#     "typer>=0.26.8",
# ]
# ///
"""Block invalid stable releases before publishing to crates.io."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import textwrap
import tomllib
from pathlib import Path
from typing import Annotated

import semver
import typer
from rich.console import Console


Version = semver.Version
console = Console()
error_console = Console(stderr=True)


def fail(message: str) -> None:
    error_console.print(message, style="red")
    raise typer.Exit(1)


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
    return json.loads(metadata)["packages"][0]["version"]


def parse_tag_version(tag_name: str) -> Version:
    version = tag_name.removeprefix("v")
    try:
        return Version.parse(version)
    except ValueError as exc:
        fail(f"release tag {tag_name!r} is not a valid Semantic Version: {exc}")


def parse_existing_tag_version(tag_name: str) -> Version | None:
    try:
        return Version.parse(tag_name.removeprefix("v"))
    except ValueError:
        return None


def is_stable(version: Version) -> bool:
    return version.prerelease is None


def is_compatible_line(current: Version, other: Version) -> bool:
    if not is_stable(current) or not is_stable(other):
        return False
    if current.major == 0:
        return current.minor != 0 and other.major == 0 and other.minor == current.minor
    return other.major == current.major


def stable_tags(repo: Path) -> list[tuple[Version, str]]:
    tags = run(["git", "tag", "--list", "v*"], cwd=repo, capture=True).splitlines()
    versions = []
    for tag in tags:
        version = parse_existing_tag_version(tag)
        if version is None:
            continue
        if is_stable(version):
            versions.append((version, tag))
    return sorted(versions)


def compatible_baseline(
    current: Version, tags: list[tuple[Version, str]]
) -> tuple[Version, str] | None:
    candidates = [
        (version, tag)
        for version, tag in tags
        if version < current and is_compatible_line(current, version)
    ]
    return candidates[-1] if candidates else None


def latest_compatible_existing_tag(
    current: Version, tags: list[tuple[Version, str]]
) -> tuple[Version, str] | None:
    candidates = [
        (version, tag)
        for version, tag in tags
        if version != current and is_compatible_line(current, version)
    ]
    return candidates[-1] if candidates else None


def check_stable_version_progression(
    tag_name: str, current: Version, tags: list[tuple[Version, str]]
) -> None:
    latest_compatible = latest_compatible_existing_tag(current, tags)
    if latest_compatible is not None:
        latest_version, previous_tag = latest_compatible
        if current <= latest_version:
            fail(
                f"{tag_name} is not greater than latest compatible release tag "
                f"{previous_tag}."
            )
        console.print(
            f"Checking version progression against previous compatible tag {previous_tag}."
        )
        return

    existing = [(version, tag) for version, tag in tags if version != current]
    if not existing:
        console.print(f"{tag_name} is the first stable release tag.")
        return

    latest_version, latest_tag = existing[-1]
    if current <= latest_version:
        fail(
            f"{tag_name} is not greater than latest stable release tag {latest_tag}. "
            "Maintenance releases need an earlier compatible baseline tag in the same release line."
        )

    console.print(
        f"Checking version progression against latest stable tag {latest_tag}."
    )


def toml_literal(value: object) -> str:
    if isinstance(value, str):
        return json.dumps(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return f"[{', '.join(json.dumps(item) for item in value)}]"
    fail(f"unsupported Cargo.toml dependency value: {value!r}")


def render_public_constructor_jieba_requirement(dependency: object) -> str | None:
    if isinstance(dependency, str):
        return toml_literal(dependency)
    if not isinstance(dependency, dict):
        return None

    supported_keys = ["version", "features"]
    items = [
        f"{key} = {toml_literal(dependency[key])}"
        for key in supported_keys
        if key in dependency
    ]
    return f"{{ {', '.join(items)} }}" if items else None


def baseline_jieba_requirement(repo: Path, baseline_tag: str) -> str | None:
    cargo_toml = run(
        ["git", "show", f"{baseline_tag}:Cargo.toml"], cwd=repo, capture=True
    )
    manifest = tomllib.loads(cargo_toml)
    dependency = manifest.get("dependencies", {}).get("jieba-rs")
    return render_public_constructor_jieba_requirement(dependency)


def run_public_dependency_smoke(repo: Path, baseline_tag: str) -> None:
    jieba_requirement = baseline_jieba_requirement(repo, baseline_tag)
    if not jieba_requirement:
        console.print(
            "No baseline jieba-rs dependency found; skipping public dependency smoke test."
        )
        return

    console.print(
        "Checking documented CangJieTokenizer construction against "
        f"baseline jieba-rs requirement {jieba_requirement!r}."
    )
    with tempfile.TemporaryDirectory(prefix="cang-jie-public-dep-") as tmp:
        crate = Path(tmp)
        repo_path = toml_literal(str(repo))
        (crate / "src").mkdir()
        (crate / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""\
                [package]
                name = "cang-jie-public-dependency-smoke"
                version = "0.0.0"
                edition = "2024"

                [dependencies]
                cang-jie = {{ path = {repo_path} }}
                jieba-rs = {jieba_requirement}
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
        fail(
            f"release tag {tag_name} does not match Cargo.toml version {crate_version}"
        )

    current = parse_tag_version(tag_name)
    if not is_stable(current):
        console.print(f"{tag_name} is a pre-release tag; skipping strict SemVer gate.")
        return

    tags = stable_tags(repo)
    check_stable_version_progression(tag_name, current, tags)

    baseline = compatible_baseline(current, tags)
    if baseline is None:
        console.print(
            f"{tag_name} starts a new effective-major release line; "
            "breaking changes are allowed."
        )
        return

    _, baseline_tag = baseline
    console.print(f"Checking SemVer compatibility against baseline {baseline_tag}.")
    run(
        ["cargo", "semver-checks", "check-release", "--baseline-rev", baseline_tag],
        cwd=repo,
    )
    run_public_dependency_smoke(repo, baseline_tag)


def self_test() -> None:
    tags = [
        (parse_tag_version("v0.19.0"), "v0.19.0"),
        (parse_tag_version("v0.20.0"), "v0.20.0"),
        (parse_tag_version("v1.2.3"), "v1.2.3"),
        (parse_tag_version("v1.3.0"), "v1.3.0"),
    ]
    assert compatible_baseline(parse_tag_version("v0.20.1"), tags) == (
        parse_tag_version("v0.20.0"),
        "v0.20.0",
    )
    assert compatible_baseline(parse_tag_version("v0.21.0"), tags) is None
    assert compatible_baseline(parse_tag_version("v1.3.1"), tags) == (
        parse_tag_version("v1.3.0"),
        "v1.3.0",
    )
    assert compatible_baseline(parse_tag_version("v1.4.0"), tags) == (
        parse_tag_version("v1.3.0"),
        "v1.3.0",
    )
    assert latest_compatible_existing_tag(
        parse_tag_version("v0.19.2"),
        [
            (parse_tag_version("v0.19.0"), "v0.19.0"),
            (parse_tag_version("v0.19.3"), "v0.19.3"),
            (parse_tag_version("v0.20.0"), "v0.20.0"),
        ],
    ) == (parse_tag_version("v0.19.3"), "v0.19.3")
    assert (
        render_public_constructor_jieba_requirement(
            {"version": "0.9.0", "default-features": False}
        )
        == '{ version = "0.9.0" }'
    )
    assert toml_literal(r"C:\tmp\cang-jie") == r'"C:\\tmp\\cang-jie"'
    assert error_console.stderr
    assert not is_stable(parse_tag_version("v0.20.0-alpha.1"))
    console.print("self-test passed")


def main(
    repo: Annotated[
        Path,
        typer.Option(
            "--repo",
            help="Repository root to inspect.",
            resolve_path=True,
            file_okay=False,
            dir_okay=True,
        ),
    ] = Path.cwd(),
    tag: Annotated[
        str | None,
        typer.Option(
            "--tag",
            help="Release tag to validate. Defaults to GITHUB_REF_NAME.",
        ),
    ] = None,
    self_test_flag: Annotated[
        bool,
        typer.Option("--self-test", help="Run lightweight internal assertions."),
    ] = False,
) -> None:
    if self_test_flag:
        self_test()
        return

    tag_name = tag or os.environ.get("GITHUB_REF_NAME")
    if not tag_name:
        fail("--tag or GITHUB_REF_NAME is required")
    if shutil.which("cargo-semver-checks") is None:
        fail("cargo-semver-checks is required")

    check_release(repo.resolve(), tag_name)


if __name__ == "__main__":
    typer.run(main)
