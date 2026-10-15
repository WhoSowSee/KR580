# Application updates

The Help menu contains one update row. The application checks only after an
explicit click; it does not poll on startup or download a release automatically.
There is no separate update dialog.

| State | Action and feedback |
| --- | --- |
| Idle | `Check for updates` starts an asynchronous version request. |
| Checking | The idle refresh arrows rotate counterclockwise beside `Checking for updates`; repeated activation is ignored. |
| Available | A Download arrow-and-tray icon and `Download update`; a separate click starts the download. |
| Downloading | A circular progress indicator shows received bytes against the release asset size. |
| Preparing | `Preparing update` and a rotating ring while the verified payload is staged. |
| Ready | A circle with a check and `Restart to update`; clicking closes and relaunches the application after replacement. |
| Current | A circle with a check and `Up to date` for three seconds, then the idle row. |
| Failed | An error icon and `Retry update`; the tooltip contains the error. Retrying a failed download retains its selected release. |

Russian labels use the same states, including `Проверить обновления`,
`Скачать обновление`, `Перезапустить и обновить` and `Обновлений нет`.
The row and menu retain their dimensions throughout these transitions.

Closing or switching away from Help preserves checks, available releases,
downloads and prepared updates. It immediately clears `Up to date`; a check
that finishes with Help closed also returns directly to idle if nothing is new.
Animation ticks are subscribed only while a busy update row is visible.
Ordinary UI ticks expire the three-second result. Errors remain available for
inspection and retry. Keyboard activation follows the existing top-menu ring
and keeps Help open for update actions.

## Release discovery and download

`runtime/updates/release.rs` requests the latest stable GitHub release from
`WhoSowSee/KR580` over HTTPS, with a versioned User-Agent and GitHub API headers.
The complete version request has a 15-second timeout and a 1 MiB response limit.
SemVer precedence determines whether it is newer than `CARGO_PKG_VERSION`;
build metadata does not affect ordering. Invalid versions, HTTP failures and
invalid asset metadata are errors, never `Up to date`.

`build.rs` embeds Cargo's `TARGET` as `KR580_BUILD_TARGET`. Matching release
packages are `KR580-Setup-<tag>-<target>.zip` for installer layouts and
`KR580-<version>-<target>.dmg` for macOS bundles. Packages must have an official
repository download URL, a nonzero size of at most 1 GiB, and a SHA-256 digest.
Downloads have a 15-second connection timeout and a 30-second read timeout.
`runtime/updates/download.rs` streams the package to a temporary directory,
checks its exact size and digest, then prepares the replacement off the UI
thread. A completed progress ring alone does not imply installation readiness.

The updater uses direct `zip` 9.0 support alongside the XLSX dependency's
transitive `zip` 8.6 entry. Its direct ZIP dependency is limited to non-macOS
builds and the flate2/zlib-rs Deflate codec; macOS uses DMG utilities.
`reqwest` 0.13 enables `rustls-no-provider`; the direct `rustls` 0.23 dependency
selects its `ring` provider and installs it before constructing clients, avoiding
the default `aws-lc-rs` build. Streaming SHA-256 reuses the same `ring` 0.17
package, with no extra direct features.
`semver` disables optional `std` integration; `tempfile` retains random-name
generation. Cargo may still enable shared ZIP features needed by XLSX, and
lockfile entries can include optional packages that are not compiled for the
current target.

Snap, Nix/RPM/Arch package wrappers, Nix-store executables, source/Cargo builds
without `install.json`, and targets without a matching package expose `Open release page` when a
new version is found. These distributions retain their existing update method.
The known `KR580_PACKAGE_KIND=nix`, `rpm` and `arch` markers explicitly disable
installer updates, preserving package-manager ownership even if an installer
manifest is present nearby. See [RPM and Arch Linux packages](linux_packages.md).
The application does not request elevated privileges: preparing an update to
a protected installation can fail with a permissions error.

## Applying a prepared update

On Windows and Linux, preparation extracts exactly one setup executable from
the ZIP, matching `KR580-Setup-<version>-<target>[.exe]` inside the archive.
The version is the package version without the release tag's optional `v`.
Its headless `--apply-update <root> <pid> <restart.json>` mode requires
embedded payloads and an existing version-1 installation manifest. It acquires
the installation lease, backs up the three installed executables, and stages
new GUI, launcher and uninstaller files beside their destinations. Only then
does its `READY` handshake enable the menu's restart action. A process handle
on Windows, or a PID existence check on Unix, delays replacement until the
emulator exits. The existing file journal restores earlier replacements if a
later replacement fails. Settings, documents, installation mode, PATH and
file associations are outside the replacement set. This path does not refresh
system integration metadata such as Windows' uninstall DisplayVersion.

On macOS, preparation mounts the DMG read-only, copies `KR580.app` beside the
current bundle, validates its identifier/version, and detaches the image. A
helper waits for exit and swaps bundles, restoring the previous bundle if the
incoming rename fails. Both helper paths have a bounded readiness handshake.
These are ordinary-error rollback mechanisms, not power-loss transactions.

Ordinary application exit applies the prepared payload without launching a
new instance. `Restart to update` writes a restart marker containing the
current saved document path and closes through the existing application exit
path. Unsaved changes first use the existing discard confirmation; cancelling
leaves the prepared update available. A present marker tells the helper to
relaunch after replacement. Temporary package/helper files are removed after
successful application. Failed preparation removes its temporary download
directory; errors during application retain diagnostic logs and recovery files.

Runtime updates do not change the emulator actor's command/event API. Their
state belongs to `ShellState.updates`; asynchronous completion messages carry
generations so stale results cannot replace a newer operation.
