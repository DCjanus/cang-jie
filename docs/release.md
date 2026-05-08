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
