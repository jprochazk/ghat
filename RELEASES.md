# Release process

1. Run `fmt`, `clippy`, and `test`. If anything fails, abort!
2. Bump version in `Cargo.toml` according to semver, then run `cargo check` to update `Cargo.lock`
3. Commit the version bump, and tag a new `vX.Y.Z` release (see `git tag` for existing tags)
4. Push the commit and tag to remote
5. Run `cargo publish`

This will publish the source to `crates.io`, so that people can install via `cargo install`.

Asynchronously, CI will build artifacts for all supported platforms, and automatically create a GitHub release.

## Details

Most of the heavy lifting is done by [`dist`](https://github.com/axodotdev/cargo-dist/). The [release workflow](.github/workflows/release.yml) runs on every new tag.

To update `dist`, simply install the new version locally, then run `dist init`. Note that this command is _interactive_.
