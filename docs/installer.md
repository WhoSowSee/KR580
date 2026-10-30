# Installer

`kr580` builds the GUI, the `kr` terminal launcher, a standalone
`k580-installer`, and a `k580-uninstaller` payload. New users run the standalone
setup artifact; `kr --install` is a maintenance entry point, not a prerequisite.
macOS releases instead distribute a drag-installable `KR580.app` in a DMG.

## Installed layout

Standalone setup owns this layout under its selected root:

```text
<install root>/
  install.json
  app/
    kr580
    uninstaller
  bin/
    kr
  data/              # portable settings
```

Windows executable names include `.exe`. Only `bin/` is added to PATH.
In this layout `kr --install` starts `app/uninstaller --setup`; running the
installed uninstaller directly or with `--uninstall <root>` opens removal.

A supported manifest identifies managed files. Upgrading such a root can retire
the owned legacy `app/k580`; a folder without a valid manifest does not authorize
overwriting colliding binary paths or deleting an unrelated legacy executable.

## Modes

| Mode | Integration | Settings |
|---|---|---|
| System | OS launcher/search and uninstall integration where supported; optional desktop shortcut | Platform configuration directory |
| Portable | Selected folder; no Start Menu, desktop or uninstall registration | `<root>/data/settings.json` |

Both modes can optionally add `bin/` to PATH and associate `.580`/`.krs` files.
Portable is current-user scoped and hides the Windows scope selector. Its
default root is `%USERPROFILE%\KR580` or `$HOME/KR580` on Unix.

System settings locations:

- Windows: `%APPDATA%\KR580\settings.json`, falling back to `%LOCALAPPDATA%`.
- macOS: `~/Library/Application Support/KR580/settings.json`.
- Linux: absolute `$XDG_CONFIG_HOME/kr580/settings.json`, otherwise
  `~/.config/kr580/settings.json`.

A standalone macOS bundle also uses Application Support. Snap uses
`$SNAP_USER_COMMON/settings.json`. Nix/RPM/Arch wrappers select writable XDG
configuration rather than a package-owned directory. Unpacked developer builds
without a manifest use adjacent `settings.json`, unless a bundle/package policy
selects another location. Temporary storage images use the OS temporary directory.

Windows System installs can target current user or all users. All-users writes
under Program Files and the machine environment need the corresponding rights;
the application does not silently elevate. Linux/macOS setup is user-scoped.

## Setup and uninstall UI

Setup is an undecorated iced window with shared caption controls, drag handling
and localized System/Portable, scope, path and integration choices. Language
follows the OS: Russian uses RU; other locales use EN. System mode adds the
desktop-shortcut option. Installing, completion and failure are explicit states.
The final action closes only after its selected finish action succeeds.

Uninstall uses the same chrome and three actual stages: system integration,
PATH/associations, then post-exit file removal. It begins after window opening.
Close is disabled during cleanup; a failure remains visible with its error.

A presentation timer approaches milestones confirmed by real task completions.
It stays below an unconfirmed next stage and reaches 100% only after the file
stage succeeds. The final Close exits; the already-started platform helper waits
for that process exit before removing the managed root.

Native close requests use the same routing as caption buttons. Windows applies
DWM corners and creates shortcuts with a hidden child process. Setup and
installed uninstaller have separate role-specific PE icons, defined in
[Assets](assets.md#where-the-assets-are-consumed).

## Desktop integration

### Windows

System mode writes a scoped Start Menu shortcut, optional desktop shortcut and
Apps & Features entry. Its uninstall command points to
`app/uninstaller.exe --uninstall <root>`. Associations use the installed GUI,
KR580 ProgID and embedded document icon for both extensions.

PATH reads validate the registry string type and UTF-16 byte length before
and after reading. Malformed values fail without changing PATH. Current-user
and machine scopes use their corresponding environment/uninstall locations.

### Linux

System integration owns the visible `kr580.desktop`; file association owns the
separate NoDisplay `kr580-file-handler.desktop`. Both expose KR580 as the
user-facing name. Toggling association cannot remove the application launcher.

User metadata follows absolute `XDG_DATA_HOME`, otherwise `$HOME/.local/share`.
Without an absolute home location registration fails rather than using the
working directory. Desktop Entry/MIME output comes from canonical templates
under `crates/ui/assets/linux/`.

Commands quote reserved characters and literal percent signs. Paths containing
`%` use `/usr/bin/env --` before the quoted executable for GLib command validation.
Control characters are rejected. Launcher resolution uses a valid manifest for
`bin/kr` to `app/kr580`, or an adjacent GUI in ordinary Cargo layouts.

`update-mime-database`, `update-desktop-database` and `xdg-mime` must succeed.
The previous default and owning executable are saved in
`$XDG_DATA_HOME/kr580/file-association.json`; an advisory lock serializes changes.
Registration failures restore metadata, preferences and saved state.

Unregister removes only the matching executable's registration. It restores the
previous default only while KR580 is still selected, preserving a later user
choice. Removal and rollback failures are reported. Strict Snap hides the
in-app action because snapd owns host integration.

### macOS

The canonical bundle is `KR580.app`, identifier `dev.kr580.emulator`, executable
`kr580`, with `dev.kr580.snapshot` and `dev.kr580.subprogram` document types.
Metadata comes from `crates/ui/assets/macos/Info.plist`; both application and
document ICNS resources are embedded.

Registration uses public Launch Services APIs and reports their OSStatus.
For a real bundle it registers that bundle. Developer/System layouts can create
an owned `~/Applications/KR580.app` wrapper; Portable keeps its wrapper inside
the install root. Existing foreign bundles are preserved. Legacy owned wrappers
can migrate from `app/k580`.

Default handlers are assigned by document role. Public unregister is unsupported;
choose a replacement default through Finder. `kr -u` reports an error and is
omitted from macOS help. Removing an owned launcher is separate from choosing
the replacement handler.

### PATH and cleanup

Unix PATH choices write a managed block to `~/.profile` or macOS `~/.zprofile`.
Cleanup removes that block or the exact Windows `<root>/bin` entry, rather than
rewriting unrelated PATH content.

Use the installed uninstaller before deleting a portable folder if it registered
PATH or associations. Manual folder deletion removes files alone. On macOS,
portable bundle removal still leaves replacement-default selection to Finder.

## Installation transaction

Setup discovers embedded/adjacent payloads, validates manifest ownership and
layout, acquires a per-root lease and stages all three installed executables
before replacing any. Unknown manifest versions and managed-path symbolic links
are rejected. Portable user data is outside the replacement set.

The file journal retains bytes, permissions and fingerprints beside targets.
Each stage observes only its own writes. Ordinary-error rollback restores owned
changes and removes only new files/empty directories. A later foreign edit is
preserved and reported with its recovery backup. Existing profile/metadata
symbolic links retain their link/target relationship.

Native rollback follows the same ownership rule:

- Windows restores original registry bytes and types for PATH, uninstall and
  association values. Unrelated values, subkeys and permissions survive; inert
  empty new keys may remain. Conflicts retain original-value recovery JSON.
- Linux restores launcher/profile/MIME/default-state files and rebuilds caches.
  Shared association lock files remain coordination objects.
- macOS snapshots Viewer/Editor/Shell defaults separately. Roles with no prior
  handler use bundle claims rather than an unrestorable explicit preference.
  Rollback restores matching defaults and retains recovery JSON on conflict.

PATH and desktop/file integration complete before atomic `install.json`
replacement. A failure combines the original error with rollback failures and
recovery paths. This is not a power-loss transaction; abrupt exit can leave a
lease or recovery files requiring inspection.

The internal `--apply-update <root> <pid> <restart.json>` mode uses embedded
payloads and an existing supported installation. It stages under the lease,
signals readiness, waits for application exit and replaces only executables.
An existing restart marker requests relaunch; ordinary exit updates without it.
See [Updates](updates.md) for downloads, readiness, recovery and platform limits.

## Build distribution packages

`just package` chooses Windows setup, Linux setup or macOS DMG. Extra arguments
use the native script's syntax:

```powershell
just package -Profile debug -Target x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1
```

```sh
just package release --target x86_64-unknown-linux-gnu
bash scripts/build_installer.sh release --target x86_64-unknown-linux-gnu
```

On macOS:

```sh
just package release --target aarch64-apple-darwin
bash scripts/build_macos_dmg.sh release --target aarch64-apple-darwin
```

Setup scripts build GUI/launcher, then the uninstaller payload and finally the
installer with embedded binaries. Each build uses `--locked`. The Unix
`KR580_CARGO=cross` path first validates locked native metadata, so stale locks
fail before cross can resolve a different dependency set.

| Artifact | Native host name | Explicit target name |
|---|---|---|
| Windows setup | `dist/KR580-Setup-<version>-windows-<arch>.exe` | `dist/KR580-Setup-<version>-<target>.exe` |
| Unix setup | `dist/KR580-Setup-<version>-<os>-<arch>` | `dist/KR580-Setup-<version>-<target>` |
| macOS image | `dist/KR580-<version>-macos-<arch>.dmg` | `dist/KR580-<version>-<target>.dmg` |

A running Windows installer that locks its output causes the next build to use
a numbered suffix. The macOS builder runs natively and verifies plist, Mach-O
architecture, image contents/icons and Applications link. DMGs are unsigned and
not notarized; normal Gatekeeper first-open approval may be required.

Unix cross builds use `KR580_CARGO=cross`. `scripts/package_installer_deb.sh` wraps
the setup executable; its desktop runtime supplies D-Bus, a Zenity fallback and
desktop/MIME tools. The desktop environment supplies its portal backend.

### Snap

From Linux, build the ready-to-run app with:

```sh
snapcraft pack --use-lxd
```

The core24 definition builds only the GUI with its configured MSRV toolchain,
stages dynamically loaded desktop libraries and uses the GPU extension.
`rust-ignore-toolchain-file` keeps that compiler selection independent of
development stable. CI builds native amd64/arm64 with LXD and filters the
platform plan using `--build-for`.

snapd owns commands, installation, refresh, desktop export and removal. The
package contains no nested setup/uninstaller. Its Desktop Entry references the
staged PNG through the Snap path; validation resolves that prefix in a temporary
copy without changing the packaged entry.

Strict confinement cannot add host `.580`/`.krs` globs. Open With integration
depends on the host already knowing the MIME type; in-app opening still works.

### NixOS

`flake.nix` provides native x86_64/aarch64 Linux packages containing GUI/launcher
wrappers, desktop/MIME metadata and icons. Nix owns activation and updates;
standalone setup is not used. Wrappers provide desktop tools and the absolute
GUI wrapper path for associations, while a package marker selects XDG settings.

Release CI checks the flake and builds the x86_64 default package. Cargo output
is read from its target-specific directory, including native Nix builds.

### RPM and Arch

Native packages install application payloads and metadata directly through the
package manager. Architecture/glibc requirements, commands and container checks
are in [RPM and Arch packages](linux_packages.md).

### Release publication

`version_release_artifacts.sh` versions setup ZIP/DEB names and verifies already
versioned RPM/Arch/DMG/Snap names against the tag. Existing destinations are
rejected; repeated processing of valid names is harmless.

Tagged publication waits for the platform/package jobs and uses the matching
Russian changelog section, generated GitHub notes and Full Changelog comparison.
It links the English changelog at the same tag. See
[release checks](maintenance.md#release-checks).

## Installation API errors

`write_manifest`, `manifest_for_executable` and `current_integration_executable`
return `InstallModeError`. Variants distinguish serialization, read/write, JSON
parse, executable-override validation and current-executable resolution.
`Error::source()` retains original I/O/JSON causes; Display is diagnostic text.

Installer and file-association text results convert explicitly at their display
boundary. The manifest remains versioned independently of these Rust error types.
