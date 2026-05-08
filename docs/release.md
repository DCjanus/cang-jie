# Release Process

This crate publishes to crates.io from GitHub Actions when a `vX.Y.Z` tag is pushed.

## Before Tagging

- Work from the latest `master`.
- Confirm `Cargo.toml` has the version being released.
- Confirm the latest `master` CI run is green.

## Tag Version Rule

Release tags use the `vX.Y.Z` format. The version in the tag must match `package.version` in [Cargo.toml](../Cargo.toml).

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
