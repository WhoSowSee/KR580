# Testing

Run checks from the repository root. [Contributing](../CONTRIBUTING.md) defines
the change/PR workflow; [Manual testing](manual_testing.md) covers native UI,
drivers and installation. Record commands actually run, results and tested OS.

## Workspace checks

`just check-all` checks justfile/Rust formatting, runs strict Clippy and workspace
tests, and builds the release GUI and launcher. Cargo equivalents:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release -p kr580 --bin kr580 --bin kr
```

The development toolchain selects stable; the workspace MSRV is Rust 1.88.
Install that toolchain and check the locked dependency set explicitly:

```sh
rustup toolchain install 1.88.0 --profile minimal
cargo +1.88.0 check --locked --workspace --all-targets
```

For public API/rustdoc changes, also run:

```sh
cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc
```

`--all-targets` covers Cargo targets on the host OS, not another OS's `cfg` paths.
Run platform checks natively. Windows error 1455 indicates memory/pagefile
pressure; reduce build parallelism, for example `-j 1`, and rerun the failed
command. A resource failure is not a passing check.

Documentation-only changes require link, example, line-budget and implementation
checks. Issue-form edits also require YAML/schema validation. Rust tests are
needed when edited Markdown is embedded by the application, as with in-app Help.

Changelog synchronization, publishable archive checks and dependency inspection
are in [Repository maintenance](maintenance.md).

## Just recipes

Install [just](https://github.com/casey/just#installation) 1.56.0 or newer:

```sh
cargo install just --locked
```

Windows recipes require PowerShell 7.4+ with `pwsh.exe` on PATH. Running `just`
lists commands. Recipes forward arguments as separate shell arguments and retain
the command's exit code; quote paths in the calling shell.

| Recipe | Purpose |
|---|---|
| `just check-all` | Justfile and complete workspace checks |
| `just fmt` | Apply justfile and Rust formatting |
| `just fmt-check` | Check Rust formatting |
| `just check-just` | Check justfile syntax and formatting |
| `just lint` / `just test` | Clippy/workspace tests |
| `just test -p kr580 --lib shutdown` | Forward a focused Cargo test selection |
| `just build` | Release GUI and launcher |
| `just run --help` / `just run-gui` | Launcher with arguments / direct debug GUI |
| `just package` | Native setup builder on Windows/Linux; DMG on macOS |
| `just icons` | Regenerate derived PNG/ICO/ICNS assets |
| `just test-packaging` | Unix package-validator and artifact-name checks |
| `just test-linux-metadata` | Native Linux Desktop Entry and MIME checks |

Platform prerequisites and packaging arguments are described in
[Installer](installer.md#build-distribution-packages) and [Assets](assets.md).

## Pull request CI

[Workspace quality](../.github/workflows/quality.yml) runs on branch pushes,
pull requests and manual dispatch.

| Check | Coverage |
|---|---|
| `Workspace ubuntu-latest` | Formatting, all-target Clippy/tests, desktop metadata, package/chart tests, artifact names, release GUI/launcher, CLI version and MSRV |
| `Workspace windows-latest` | Native formatting, all-target Clippy and workspace tests |
| `Workspace macos-latest` | Native formatting, all-target Clippy and workspace tests |

Each matrix job has a 45-minute timeout. New runs cancel older runs for the
same branch/PR. Cargo caches are restored for all events and saved on pushes.
Checkout does not persist credentials.

Keep these status names aligned with the repository's required checks when
changing the matrix. Branch protection is configured separately in GitHub.
Release packaging uses its tag/manual workflow and native/cross build matrix.

## Focused regression checks

Use the complete suite for integration changes. These entry points shorten
iteration without replacing the workspace gate:

| Area | Command | Main contract |
|---|---|---|
| CPU | `cargo test --locked -p k580-core` | Decode/execute, flags, I/O, IRQ, reset and T-state accounting |
| Snapshots | `cargo test --locked -p kr580 --test persistence_formats --test program_registers` | Exact `.580` layout, independent PC/SP/register fixtures, malformed sizes |
| Subprograms/atomic saves | `cargo test --locked -p kr580 --lib persistence::` | Bounded reads, inclusive ranges, no partial mutation and replacement failures |
| TXT/XLSX | `cargo test --locked -p kr580 --test export_options` | Full-range zero-byte round trips, target selection and TXT naming |
| Devices | `cargo test --locked -p kr580 --test device_bus --test device_budgets --test network_lifecycle` | Port routing, admission/buffer limits and network lifetime |
| Backend shutdown | `cargo test --locked -p kr580 --lib shutdown` | Accepted writes finish before receipt; failures do not skip other devices |
| UI shutdown | `cargo test --locked -p kr580 --bin kr580 app::shutdown::tests` | Dirty gate, accepted work, failed close and completion ordering |
| UI | `cargo test --locked -p kr580 --bin kr580` | Editing, navigation, drafts, worker completions, Help and updates |
| Linux associations | `cargo test --locked -p kr580 --test linux_associations` | Ownership, previous defaults and ordinary-error rollback |
| macOS bundle fixtures | `cargo test --locked -p kr580 --lib macos_bundle` | Bundle ownership, portable paths and malformed-manifest error propagation on Unix |
| Installer | `cargo test --locked -p kr580 --bin k580-installer --bin k580-uninstaller` | Staging, ownership, rollback and native adapters |

CPU tests classify all opcode slots and smoke-run documented instructions.
Semantic cases cover flags, branches, stack and device/timing effects; they are
not a substitute for the external binary suites listed in
[Assumptions](assumptions.md).

Concurrency regressions wait for actual completion with bounded deadlines.
Shutdown tests park runtimes to fill queues before draining; printer export
checks completion before reading output. Generation tests inject completions
explicitly. Tests of asynchronous actor changes must await acknowledgement
rather than assume sending a command already changed the snapshot.

Native registry, installation and association fixtures use isolated roots and
adapters. They must preserve the current user's real PATH, desktop and defaults.
The macOS bundle module is also compiled by Linux library tests through
`cfg(all(test, unix))`; its fixtures use temporary paths without calling Launch Services.
Performance regressions check zero allocation during metadata execution and
unchanged paused-device polls; the relevant test names are discoverable with
`cargo test --locked --workspace -- --list`.

## Native metadata and packages

Linux checks require the desktop/MIME tools used by the application:

```sh
bash scripts/verify_linux_metadata.sh
bash scripts/verify_release_artifact_names.sh
python3 -B -m unittest discover -s scripts -p test_linux_packages.py -v
python3 -B .github/scripts/test_gen_star_history.py
```

Windows can run the Python unit tests with `python` in place of `python3`.
The Linux integration suite uses deterministic command fixtures; an optional
real consumer check additionally requires `gio`:

```sh
cargo test --locked -p kr580 --lib file_assoc::linux::tests::desktop_entry_consumer_preserves_executable_and_file_arguments -- --ignored --exact
```

On macOS, build an image and run its native verifier:

```sh
bash scripts/verify_macos_dmg.sh --dmg <image> --architecture <arm64|x86_64> --version <version>
```

The verifier uses `hdiutil`, `plutil`, `iconutil` and `lipo`, mounts read-only,
checks bundle metadata/icons/architecture and detaches afterward. Package
contents, ELF/glibc checks and disposable Fedora/Arch smoke installations are
defined in [Linux packages](linux_packages.md#verification-and-release-wiring).

Installer build-script changes should capture effective commands for debug and
release, native/explicit targets and `KR580_CARGO=cross`: exactly one `--locked`
per build, correct icon roles and payload paths. A stale lockfile must fail
before building payloads. Command capture cannot replace native package builds.

## Opt-in external checks

Ordinary tests avoid live GitHub, official downloads and installed drivers.
Run these only with the corresponding environment:

```sh
cargo test --locked -p kr580 --test native_printer_properties -- --ignored --nocapture
cargo test --locked -p kr580 --bin kr580 runtime::updates::release::tests::live_release_check_completes_with_a_valid_result -- --ignored --exact
```

The printer test reapplies an installed driver's current option without sending
a physical job. The GitHub test performs a live release request.

For official ZIP extraction on Windows, download and verify the release archive
first, then provide its version without the optional `v` prefix:

```powershell
$env:KR580_UPDATE_TEST_ARCHIVE = 'C:\path\KR580-Setup-v5.0.0-x86_64-pc-windows-msvc.zip'
$env:KR580_UPDATE_TEST_VERSION = '5.0.0'
cargo test --locked -p kr580 --bin kr580 runtime::updates::install::tests::official_release_zip_extracts_installer -- --ignored --exact
```

Actual GUI replacement, printer dialogs/jobs and native integration still require
[manual smoke checks](manual_testing.md). Report those separately from fixtures.

## Samples and assets

Tests synthesize small programs and temporary files; the `square_program` case
loads a generated `.580` and verifies the monitor outline. Repository examples
are also available for manual device checks.

When master artwork changes, run `just icons`, include derived assets, then
rebuild. The checked-in PNG/ICO/ICNS files are build inputs; source artwork
changes alone do not update them. See [Assets](assets.md#regenerating-the-icon-set).
