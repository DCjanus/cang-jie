# Release Process

This crate publishes to crates.io from GitHub Actions when a `vX.Y.Z` tag is pushed.

## Before Tagging

- Work from the latest `master`.
- Confirm `Cargo.toml` has the version being released.
- Confirm the latest `master` CI run is green.

## Tag Version Rule

Release tags use the `vX.Y.Z` format. The version in the tag must match `package.version` in [Cargo.toml](../Cargo.toml).
Versions must follow [Semantic Versioning](https://semver.org/).

[release.yml](../.github/workflows/release.yml) checks this before authenticating to crates.io or running `cargo publish`.

## SemVer Gate

The release workflow runs [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) before authenticating to crates.io.

For stable tags, [check_release_semver.py](../scripts/check_release_semver.py) selects the latest compatible baseline tag:

- `0.y.z` patch releases compare against the latest `v0.y.*` tag.
- `1.y.z` and later compare against the latest lower tag with the same major version.
- New effective-major release lines, such as `v0.20.0` after `v0.19.0`, allow breaking changes and skip strict blocking.

Pre-release tags such as `v0.20.0-alpha.1` currently skip strict SemVer enforcement. Define a release-line policy before relying on pre-release tags for compatibility guarantees.

The gate also compiles a small downstream crate that uses the documented direct `CangJieTokenizer` construction pattern with the baseline `jieba-rs` requirement. This catches public dependency breaks that rustdoc-level SemVer checks may miss.

## Publishing

Push the tag to trigger the release workflow:

```console
git tag vX.Y.Z
git push origin vX.Y.Z
```

Wait for the release workflow to finish, then confirm the version is visible on crates.io before creating the GitHub Release.

## GitHub Release Notes

Keep release notes short and focused:

- Write a compact `Highlights` section.
- Keep `Breaking Changes` separate when compatibility changes exist.
- Preserve GitHub's generated `What's Changed`, `New Contributors`, and `Full Changelog` sections when useful.

Create the release from a local notes file and GitHub's generated sections:

```console
gh release create vX.Y.Z \
  --verify-tag \
  --notes-start-tag vA.B.C \
  --title "vX.Y.Z" \
  --notes-file /tmp/cang-jie-vX.Y.Z-notes.md \
  --generate-notes
```

Here, `vX.Y.Z` is `tag_name`, and `--notes-start-tag vA.B.C` is `previous_tag_name`.
