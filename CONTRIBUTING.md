# Contributing to KR580

Check the [issues](https://github.com/WhoSowSee/KR580/issues) and
[open pull requests](https://github.com/WhoSowSee/KR580/pulls) for related work
before opening a new one.

AI-assisted contributions must follow the [AI Policy](AI_POLICY.md).

For feature requests, [use the feature request form](https://github.com/WhoSowSee/KR580/issues/new?template=feature.yml)
with the problem you want to solve and an example of the proposed workflow.
Discuss large features, public API changes, and changes to CPU semantics or file
formats before implementing them.

## Reporting a bug

[Use the bug report form](https://github.com/WhoSowSee/KR580/issues/new?template=bug.yml)
with:

- Your KR580 version (`kr --version` or the GUI's About dialog). For a source
  build, include the commit; if the application cannot start, give the release
  or package version you installed.
- Your operating system and version, CPU architecture, and installation method
  (standalone setup, DMG, package manager, or source build).
- The steps, relevant settings, and device state needed to reproduce the bug.
- What you expected and what happened, including errors.

For CPU or timing problems, include the instruction bytes, load address,
initial registers and flags, relevant RAM, and expected versus actual state
or T-state count. A small program is easier to investigate than a full session.

For file or device problems, include a minimal sample, relevant ports, and the
device configuration. Attach binary samples as a ZIP or paste a small hex dump.
For UI layout problems, include window size, display scaling, theme, language,
and a screenshot if useful. On Linux, also identify Wayland or X11.

## Working locally

Install Rust through [rustup](https://rustup.rs/).
[rust-toolchain.toml](rust-toolchain.toml) selects `stable` with the `minimal`
profile and the `rustfmt` and `clippy` components. The minimum supported Rust
version is declared in
[Cargo.toml](Cargo.toml); see [Testing](docs/testing.md#workspace-checks) for
MSRV verification.

[.editorconfig](.editorconfig) supplies editor defaults for encoding, indentation,
and whitespace, with exceptions for Markdown and binary files.
[rustfmt.toml](rustfmt.toml) defines the shared formatter settings, and
[.gitattributes](.gitattributes) defines LF and binary-file handling. See
[Repository configuration](docs/maintenance.md#repository-configuration) for their
validation and explicit compiler overrides used by packaging.

Fork [KR580](https://github.com/WhoSowSee/KR580), then:

```sh
git clone https://github.com/YOUR-USERNAME/KR580.git
cd KR580
git remote add upstream https://github.com/WhoSowSee/KR580.git
git switch -c fix/your-change
```

Run the launcher or the GUI directly:

```sh
cargo run --locked -p kr580 --bin kr -- --help
cargo run --locked -p kr580 --bin kr580
```

The root `justfile` provides equivalent developer commands. See
[Just recipes](docs/testing.md#just-recipes) for installation and platform
requirements.

## Making a change

Start with the [documentation index](docs/README.md), then read the architecture
overview and the guide for the component you are changing.
Check behavior changes against the documented CPU, device, and file-format
contracts and the existing tests. Discuss changes to those contracts before
implementation, and add regression coverage for the agreed behavior.

Keep your PR focused on one problem and follow the surrounding module structure:

- `crates/core` owns deterministic CPU state, instruction execution, timing,
  interrupts, and `PortBus`.
- `crates/ui` owns the desktop application, backend actor, devices, persistence,
  launcher, installer, and platform integration.

For a bug fix, add or extend a regression test when the behavior can be
automated. CPU integration tests belong under `crates/core/tests`; UI/backend,
device, file-format, and launcher integration tests belong under
`crates/ui/tests`. Keep unit tests near the module they exercise. CPU changes
must cover the applicable flags, control flow, memory effects, and T-state
counts; device changes must cover success, not-ready, busy, disconnect, and
asynchronous completion paths.

Keep source files at or below 400 lines, including blank lines and comments,
splitting by responsibility when needed. Write code comments and public API
documentation in English. Prefer readable code without comments that repeat its
implementation. Use one-line comments only for 8080/KR580 hardware quirks,
deliberate iced 0.14 or OS workarounds, `// SAFETY:` justifications, or subtle
invariants. Public doc comments may describe API contracts.

Keep each maintained documentation file at or below 500 lines, including blank
lines and code fences. Shorten repetition before splitting by topic; link to the
primary description of a contract rather than maintaining another copy.

Update the relevant `docs/` reference in the same change when behavior, public
APIs, dependencies, build scripts, file layout, assets, or platform integration
change. Keep [architecture.md](docs/architecture.md) aligned with crate
responsibilities and [ui_app.md](docs/ui_app.md) with its linked UI guides aligned
with state, messages, shortcuts, and platform behavior.

Keep the Russian [README.md](README.md) and English [README-EN.md](README-EN.md)
aligned. If a changelog changes, update all four files together:

- `CHANGELOG.md` and `crates/ui/CHANGELOG.md` must be byte-for-byte identical.
- `CHANGELOG-EN.md` and `crates/ui/CHANGELOG-EN.md` must be byte-for-byte identical.
- All four must expose the same release version/date index.

See [Repository maintenance](docs/maintenance.md#release-checks) for release changelog and package
archive checks.

## Checking your work

Run `just check-all` before submitting a code change. It checks justfile and Rust
formatting, runs Clippy and workspace tests, and builds the release GUI and
launcher. The Cargo equivalents are:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release -p kr580 --bin kr580 --bin kr
```

Use `just fmt` to apply formatting. `just check-just` checks the justfile itself.

Choose additional checks from [Testing](docs/testing.md): manual UI checks,
native printer tests, platform metadata checks, and release packaging require
their matching environment. `just package` runs the native packaging script;
see [Installer](docs/installer.md#build-distribution-packages).

After changing the master icon artwork, run `just icons` and include the updated
derived assets before rebuilding. See [Assets](docs/assets.md).

For documentation and template changes, check links, examples, and accuracy
against the implementation. Parse issue-form YAML and check its field structure
against the [GitHub form schema](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-githubs-form-schema).
Markdown embedded by the application, such as Help articles, also requires the
relevant application tests. Other documentation-only changes do not require Rust tests.

Report the commands you actually ran, their results, and any checks you skipped
with the reason. Passing on one OS does not verify another platform's behavior.

## Opening a pull request

Write commit subjects in English using the existing `type(scope): summary`
style with an imperative summary, for example:

```text
fix(ui): preserve keyboard focus after activation
test(core): cover conditional call timing
docs(contributing): document contributor checks
```

Commit your changes, update your branch from `upstream/main`, and rerun the
relevant checks before pushing to your fork. Open a pull request to
[WhoSowSee/KR580](https://github.com/WhoSowSee/KR580/pulls) and fill in the
[PR template](.github/pull_request_template.md).

Describe the problem, resulting behavior, compatibility effects, and checks you
ran. Link the related issue; include a minimal program or before/after state for
CPU changes and screenshots with window size and display scaling for visual UI
changes. Explain any checks you could not run. Push review fixes to the same
branch.

Wait for the required `Workspace` checks on Linux, Windows, and macOS before
merging. The Linux check also verifies MSRV, a release build, and packaging
metadata. See [Pull request CI](docs/testing.md#pull-request-ci) for the workflow
and the GitHub ruleset that enforces these statuses on `main`.
