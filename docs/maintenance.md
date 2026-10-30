# Repository maintenance

[Testing](testing.md) owns workspace/CI commands; this guide covers repository
configuration, publishing checks and the README chart publisher.

## Repository configuration

| File | Responsibility |
|---|---|
| [`.editorconfig`](../.editorconfig) | Editor encoding, indentation and whitespace defaults |
| [`.gitattributes`](../.gitattributes) | Text normalization, Rust diffs and binary handling |
| [`.gitignore`](../.gitignore) | Build/package outputs and local tool/editor files |
| [`rustfmt.toml`](../rustfmt.toml) | Workspace Rust formatting |
| [`rust-toolchain.toml`](../rust-toolchain.toml) | Stable development channel with rustfmt/Clippy |
| [`justfile`](../justfile) | Developer command entry points |

Markdown hard breaks are retained. Program/device images, workbooks and raster
assets are binary; their bytes must not undergo line-ending conversion.
Inspect representative paths when changing these rules:

```sh
git check-attr text eol diff merge -- crates/core/src/flags.rs examples/floppy_sample.kpd crates/ui/assets/icons/icon.png
git ls-files --eol
```

The stable development channel does not raise the declared MSRV. Explicit
`cargo +1.88.0` or `RUSTUP_TOOLCHAIN` overrides select that compiler. Snap's
configured compiler ignores the development toolchain file; Nix uses its package
set's compiler. Validate each affected build environment.

## Release checks

Both crates inherit the workspace version. Native package scripts read Cargo
metadata; CLI/About and executable metadata use the package version.
Keep the UI's core path-dependency version and locked workspace entries aligned.

The root changelogs feed release automation; package copies are embedded by the
app. Update all four together:

- `CHANGELOG.md` must equal `crates/ui/CHANGELOG.md` byte for byte.
- `CHANGELOG-EN.md` must equal `crates/ui/CHANGELOG-EN.md` byte for byte.
- RU and EN must have identical release version/date indexes.

For a release, each non-release commit since the previous tag gets one bullet
per language. A mixed commit remains one bullet; exclude the version-bump commit.
Keep the embedded reader's version/date headings and Markdown list markers.
The repository's release convention avoids dash punctuation within bullet text.

Inspect publishable archives before publishing:

```sh
cargo package --workspace --locked
```

Both crates use the root Russian README. Check the normalized manifests and
README inside the `.crate` files. Root README language/documentation links use
absolute GitHub URLs so they also resolve from the registry's crate subdirectory.
Published registry versions cannot be overwritten.

Release naming, package content validation and native distribution checks are
linked from [Installer](installer.md#build-distribution-packages) and
[Linux packages](linux_packages.md).

## Dependency inspection

```sh
cargo machete --with-metadata --skip-target-dir .
cargo tree -p kr580 --target all -f "{p} features=[{f}]"
cargo tree -p kr580 --target all -e features -i roxmltree
cargo tree -p kr580 --target all -e features -i windows
```

The Windows build script consumes `winresource` and `embed-resource` behind a
target cfg, so the dependency scanner's documented exclusions cover those uses.
Cargo unifies features across users of a dependency; a disabled direct default
does not prove the effective feature is absent. Inspect the resolved graph.
Keep versions/features in Cargo manifests and lockfiles, with only the rationale
and verification method in prose.

## Star-history publisher

[The workflow](../.github/workflows/star-history.yml) tests the generator,
fetches history with read-only `GITHUB_TOKEN`, and stages only the four chart
SVGs. Unchanged output produces no commit. Artwork and regeneration commands
are documented in [Assets](assets.md#readme-star-history).

Publisher setup in GitHub:

1. Create a repository-scoped GitHub App with `Contents: read and write` and its
   mandatory metadata access; install it only on the intended repository.
2. Set Actions variable `STAR_HISTORY_APP_CLIENT_ID` and secret
   `STAR_HISTORY_APP_PRIVATE_KEY` to its client ID and PEM private key.
3. Allow that App to publish through the branch ruleset while keeping ordinary
   updates subject to the required Workspace checks.
4. Verify its first run and resulting push-triggered quality checks.

The workflow creates an App token only when charts changed and revokes it when
the job completes. A regular `GITHUB_TOKEN` push does not trigger the push-based
checks used for these chart commits. Keep publisher permissions limited to the
repository and generated files.
