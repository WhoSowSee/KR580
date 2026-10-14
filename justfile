set minimum-version := '1.56.0'
set default-list

[unix]
set shell := ['sh', '-cu']

[windows]
set shell := ['pwsh.exe', '-NoLogo', '-NoProfile', '-CommandWithArgs']

[doc('Check the justfile and workspace formatting, lints, tests, and release build')]
check-all: check-just check

[doc('Check workspace formatting, lints, tests, and the release build')]
check: fmt-check lint test build

[doc('Format the justfile and all workspace crates')]
fmt: fmt-just
    cargo fmt --all

[doc('Format the justfile')]
fmt-just:
    just --fmt

[doc('Check justfile syntax and formatting without editing files')]
check-just:
    just --fmt --check

[doc('Check formatting of all workspace crates without editing files')]
fmt-check:
    cargo fmt --all -- --check

[doc('Run Clippy on all workspace targets; warnings are errors')]
lint:
    cargo clippy --locked --workspace --all-targets -- -D warnings

[doc('Run workspace tests with optional Cargo arguments and a filter')]
[positional-arguments]
[unix]
test *args:
    cargo test --locked --workspace "$@"

[doc('Run workspace tests with optional Cargo arguments and a filter')]
[positional-arguments]
[windows]
test *args:
    @$ErrorActionPreference = 'Stop'; \
    cargo test --locked --workspace @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Build the release GUI and launcher binaries')]
build:
    cargo build --locked --release -p kr580 --bin kr580 --bin kr

[doc('Run the kr launcher with optional application arguments')]
[positional-arguments]
[unix]
run *args:
    cargo run --locked -p kr580 --bin kr -- "$@"

[doc('Run the kr launcher with optional application arguments')]
[positional-arguments]
[windows]
run *args:
    @$ErrorActionPreference = 'Stop'; \
    cargo run --locked -p kr580 --bin kr '--' @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Run the GUI directly and keep debug logs in the terminal')]
run-gui:
    cargo run --locked -p kr580 --bin kr580

[doc('Build the Windows setup executable with optional script arguments')]
[positional-arguments]
[windows]
package *args:
    @$ErrorActionPreference = 'Stop'; \
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1 @($args | Select-Object -Skip 1); \
    exit $LASTEXITCODE

[doc('Build the Linux setup executable with optional script arguments')]
[linux]
[positional-arguments]
package *args:
    bash scripts/build_installer.sh "$@"

[doc('Build the macOS DMG with optional script arguments')]
[macos]
[positional-arguments]
package *args:
    bash scripts/build_macos_dmg.sh "$@"

[doc('Regenerate PNG, ICO, and ICNS assets from the master artwork')]
[windows]
icons:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/generate_icons.ps1

[doc('Regenerate PNG, ICO, and ICNS assets from the master artwork')]
[unix]
icons:
    bash scripts/generate_icons.sh

[doc('Verify release artifact naming in a disposable directory')]
[unix]
test-packaging:
    bash scripts/verify_release_artifact_names.sh

[doc('Verify Linux desktop and MIME metadata; requires native desktop tools')]
[linux]
test-linux-metadata:
    bash scripts/verify_linux_metadata.sh
