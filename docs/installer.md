# Installer

`kr580` builds four desktop-facing binaries:

- `k580` - the GUI emulator.
- `kr` - the terminal launcher and file-association helper.
- `k580-installer` - the graphical installer.
- `k580-uninstaller` - the graphical uninstaller payload installed as
  `app/uninstaller`.

The user-facing setup artifact is built by `scripts/build_installer.ps1`
on Windows or `scripts/build_installer.sh` on Linux/Unix. The scripts first
build `k580` and `kr`, then build `k580-uninstaller` with the uninstall icon,
then rebuild `k580-installer` with the setup icon and those binaries embedded.
The resulting file under `dist/` is the installer a new user runs before `kr`
exists.

macOS releases use `scripts/build_macos_dmg.sh` instead. It packages the native
`k580` executable directly as `KR580.app`, copies the canonical plist and both
ICNS resources, and creates an unsigned compressed DMG with an `/Applications`
link. The user drags the complete app into Applications; no setup executable or
uninstaller is nested in the image.

`kr --install` remains a maintenance entry point for developer builds. In an
installed layout it launches the installed uninstaller binary with `--setup`,
so the setup UI is explicit and the installed file still reads as an
uninstaller.

## Installer Window

The graphical setup uses an undecorated iced window with its own Tokyo Night
title bar. The custom chrome owns drag, minimize, maximize/restore, and close
actions instead of relying on the native OS title bar. The caption buttons use
the same `window-minimize`, `window-maximize`, `window-restore`, and
`window-close` SVG assets and `32x24` button metrics as the emulator chrome.
The setup and uninstall windows also subscribe to native close requests, so
platform close shortcuts such as Windows Alt+F4 route through the same close
path as the custom caption button.
On Windows, the installer also applies the same DWM rounded-corner preference
as the emulator window when the window opens.

Setup and uninstall choose their UI language from the operating system at
startup: Russian system UI uses Russian strings, while English and every other
system language use English strings. The Russian mode choices are `Системный`
and `Портативный`, matching the masculine `Режим` heading, and the installation
location section is labeled `Путь` (`Path` in English).

The installer is a two-column instrument surface. A fixed `164px` rail carries
a `116x94` KR580 DIP outline below a deliberate top spacer, the centered package
version and title-cased platform/architecture metadata such as `Windows · x64`,
and four compact signal-group rows.
The title bar uses the separate square CPU glyph and the same canvas color as
the setup body. The rail and form share that canvas, while a thin divider
separates only their working areas. The rail uses a `32px` top spacer so the
visible DIP-to-top and bus-table-to-bottom intervals are balanced without
changing the window height. The
form occupies the remaining width as a flat sequence of mode, scope, path,
and integration sections; thin rules and spacing replace the previous nested
panel and option card hierarchy.
Mode and scope use joined `44px` segmented controls with one neutral outer frame
and a square internal seam. Idle and selected segments inherit the canvas;
selection uses only the blue left rail and radio dot. The rail replaces the
underlying gray edge and follows its outer corner radius. Hover uses the darker
panel token, pressing uses the stronger surface token, and neither state hides or
doubles the outer frame.
The fixed `12px` radio ring contains a `6px` selected dot, preserving an inset
between dot and outline. The path and compact `96x44` Browse action form
one framed control with a square seam and only external corner rounding. Its
input surface is transparent, keeping the left and lower outer rules visually
even. The path uses the emulator blue value color; Browse keeps its folder glyph.
Every setup checkbox uses one shared `14px` control size, including Integration
and the post-install Open-folder/Launch action. Integration retains its `12px`
vertical rhythm. Unchecked boxes inherit the canvas and retain only their outline.
The four signal rows form one transparent two-column table with a single rounded
outer frame and horizontal row rules. The first fixed column holds only the
number; the second holds a static green marker and the compact `ADDR`, `DATA`,
`CTRL`, or `INT` label. These labels identify KR580 signal groups rather than
reporting live installer telemetry. Individual rows have no fill or rounded
corners.
The bottom command bar spans the complete window width, so the rail/form divider
ends above it. It contains only the compact Install/Done action aligned to the
right; install state is communicated by the form, progress panel, result report,
and action label instead of a redundant left badge. An Installing progress panel
appears above the bar only while work is active and inherits the setup canvas,
retaining only its frame. The unfilled progress track uses the same canvas; only
its outline and blue completed range remain visible.
While files are being copied, the result area switches to an Installing state
with a blue progress bar that advances on an installer-only timer tick.
Success replaces the form with an installed report followed by a checked "Open
installation folder" action for Portable mode or "Launch KR580" for System mode.
Report lines omit terminal periods; completed operations use `TEXT`, while
skipped or unchanged operations use `MUTED`. The pinned `Done` button runs the
selected action and closes the installer; failures keep the window open and
appear below the checkbox.
Integration options are ordered as PATH, file associations, then the optional
desktop shortcut. Both modes show the first two; System mode adds the desktop
shortcut as the final option.
On Windows, the association option maps both `.580` and `.krs` through the
same KR580 ProgID, icon, and open command. Its user-facing ProgID name is
`KR580`, matching the executable's Windows file description.
On Linux, system integration owns the visible `kr580.desktop` launcher, while
file associations use a separate `NoDisplay` `kr580-file-handler.desktop`.
Toggling associations therefore cannot remove KR580 from the application menu.
Both entries expose the user-facing name `KR580`, including file-manager
Open With lists.
The hidden handler records its exact executable; a target build, portable copy,
or older installation cannot remove a handler owned by another path.
User-level integration follows `XDG_DATA_HOME`; when it is unset or relative,
KR580 uses `$HOME/.local/share`. Registration fails instead of writing into the
working directory when no absolute home location is available.
Generated Desktop Entry commands quote paths according to the freedesktop
rules, including reserved characters and literal percent signs.
Runtime registration, the Unix installer, and declarative packages render the
tracked templates under `crates/ui/assets/linux` instead of maintaining
independent Desktop Entry or shared-MIME copies.
Linux integration returns an error when `update-mime-database` or
`update-desktop-database` is missing or exits unsuccessfully, so the installer
and Settings cannot report a cache update as completed when it failed.
After updating those databases, Linux assigns `application/x-kr580` to the
hidden KR580 handler through `xdg-mime`.
Strict Snap confinement suppresses this in-app action because it cannot mutate
the host shared-MIME database; snapd owns the package's exported handler.
Portable mode hides Windows scope because it always installs to the selected
folder for the current user.
Installed-state messages use user-facing wording and do not expose the internal
`app/`, `bin/`, or portable data folders.
macOS integration uses one `KR580.app` contract: bundle identifier
`dev.kr580.emulator`, executable name `KR580`, and the canonical
`crates/ui/assets/macos/Info.plist` template. The installer and association
layer no longer generate different bundle names or metadata.
The bundle exports `dev.kr580.snapshot` for `.580` and
`dev.kr580.subprogram` for `.krs`, with distinct Finder type names and a shared
document icon. `KR580.icns` supplies the application icon.
Registration calls the public Core Services `LSRegisterURL` API and reports its
`OSStatus`; it does not invoke the private `lsregister` executable.
The in-app association action locates the `KR580.app` containing the running
executable, or the installer-owned bundle in `~/Applications`. It registers
that existing bundle without copying binaries, rewriting metadata, or deleting
the application when association state changes.
The same action makes `dev.kr580.snapshot` and `dev.kr580.subprogram` default to
the stable KR580 bundle identifier through the public Launch Services role API.
The default window is `720x600` logical pixels with a `680x560` minimum. The
fixed command bar uses the same canvas as the setup body and keeps the compact
`176x40` primary action visible at the minimum size and at high DPI. Hover uses
a slightly lighter blue from the same color family instead of switching to cyan.

## Uninstaller Window

The installed `app/uninstaller` is also an iced GUI binary. Running it directly
from an installed layout or through `uninstaller --uninstall <install root>`
opens a `760x500` undecorated Tokyo Night window (`700x460` minimum) with the
same CPU title glyph, minimize/maximize/restore/close controls, and Windows DWM
corner preference as setup. Its centered product header stacks the outlined DIP
mark above `КР580` and version/platform metadata; the separate removal-copy block
from the initial concept is intentionally absent. The read-only install path
occupies one border-only field below the header.

One joined border-only instrument block exposes three real cleanup stages:
`01 СИСТЕМА` removes Start Menu/desktop shortcuts and the uninstall entry,
`02 СВЯЗИ` removes the managed PATH entry and recorded `.580` / `.krs`
associations, and `03 ФАЙЛЫ` schedules removal of the install directory. A green
marker means complete, blue with a left rail means active, muted means pending,
and red means the active stage failed. The status line and progress bar below
the stage strip use the same outer frame and canvas.

Cleanup starts only after the native window opens. All three stages run
automatically while the bottom action is disabled and reads `Removing...` /
`Удаление...`. Real stage changes confirm milestones at 12%, 40%, 68%, and 100%.
A presentation timer catches up to each confirmed milestone by two percentage
points every 30 milliseconds. While the current system operation is still
running, it continues at 0.1 percentage point per tick but reserves the final
point before the next milestone. The 40% Links milestone therefore keeps moving
toward 67% while Windows broadcasts PATH and association changes, without
claiming that the 68% Files milestone has completed. The file stage starts a
platform helper that waits for the uninstaller process to exit before deleting
its directory. If all operations finish before the animation, the Files stage
remains active until the displayed progress reaches 100%. Only then does the
localized Close/`Закрыть` action become available, and it merely exits the
already-completed workflow. A failed stage stops the animation at its current
value and stays visible in red with its error text.

## Build Distribution Packages

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1 -Target x86_64-pc-windows-msvc
```

Linux/Unix:

```sh
bash scripts/build_installer.sh
bash scripts/build_installer.sh release --target x86_64-unknown-linux-gnu
```

macOS:

```sh
bash scripts/build_macos_dmg.sh
bash scripts/build_macos_dmg.sh release --target aarch64-apple-darwin
```

The packaging scripts produce:

- Windows host builds: `dist/KR580-Setup-<version>-windows-<arch>.exe`
- Targeted Windows builds: `dist/KR580-Setup-<version>-<target>.exe`
- Linux/Unix host builds: `dist/KR580-Setup-<version>-<os>-<arch>`
- Targeted Linux/Unix builds: `dist/KR580-Setup-<version>-<target>`
- macOS host images: `dist/KR580-<version>-macos-<arch>.dmg`
- Targeted macOS images: `dist/KR580-<version>-<target>.dmg`

The macOS builder runs only on macOS. Before returning it validates the plist,
Mach-O architecture, disk image, mounted app layout, icons, bundle identity,
document UTIs, and Applications link through `verify_macos_dmg.sh`. The app and
DMG are intentionally unsigned and not
notarized, so Gatekeeper may require the normal explicit first-open approval.

`KR580_CARGO=cross` makes the Unix script invoke `cross build` for Linux target
matrices. `scripts/package_installer_deb.sh` wraps a built Linux setup
executable in a Debian package. `snap/snapcraft.yaml` builds the ready-to-run
GUI package used by CI on native `ubuntu-24.04` and `ubuntu-24.04-arm` runners;
the workflow installs Snapcraft from `9.x/stable`, filters the core24
`platforms` build plan with `snapcraft pack --build-for`, and builds through
LXD instead of destructive mode. The core24 build uses Ubuntu Noble package
names, stages the dynamically loaded X11/Wayland/font/dialog libraries, and
uses the GPU extension for driver-compatible graphics. Build dependencies stay
minimal because the graphics crates load those libraries at runtime.
The Rust plugin uses the workspace's tested Rust 1.88.0 MSRV and builds only
`crates/ui`'s `k580` binary. Its override calls the
plugin's default build before installing the application icon and rendering the
canonical package Desktop Entry. There are no setup/uninstall commands or
nested installation directories: snapd owns installation, refresh, removal,
command wrapping, interfaces, and desktop export.
The exported Desktop Entry advertises `application/x-kr580`, but strict Snap
cannot add the new `.580`/`.krs` glob definition to the host shared-MIME
database. File-manager association therefore works where that MIME type is
already known (for example from a native package); otherwise the user may still
select KR580 through Open With, while opening from KR580's own dialog works.
On Linux, rfd 0.17 loads `libdbus` for XDG dialogs and falls back to `zenity`.
The Debian setup package provides both plus `shared-mime-info`,
`desktop-file-utils`, and `xdg-utils`, which supply the cache update and
default-handler commands used by file-type registration. The desktop supplies
its portal backend.
If the Windows target artifact is locked by a running installer, the PowerShell
script writes the same setup under a numbered suffix such as
`KR580-Setup-<version>-windows-<arch>-1.exe` instead of failing after the
release build has already completed.

## NixOS Package

`flake.nix` exposes a Nix package for `x86_64-linux` and `aarch64-linux`.
That package installs the ready-to-run `k580` and `kr` binaries, desktop entry,
icons, and `.580` / `.krs` MIME metadata into the Nix store. It does not run the
graphical setup flow because NixOS owns PATH, desktop integration, and package
activation declaratively.

The release workflow runs `nix flake check --no-build` and builds
`.#packages.x86_64-linux.default`; tagged releases wait for that job before the
GitHub release is published. The derivation installs from Cargo's
target-specific output directory because the nixpkgs Cargo hook passes
`--target` even for native Linux builds.

## GitHub Release Notes

Tagged builds publish a GitHub release after every platform packaging job
completes. The release job extracts the matching version section from the
Russian `CHANGELOG.md`, appends GitHub's generated notes and `Full Changelog`
comparison, then adds an `English Changelog: [CHANGELOG-EN.md](...)` link to
the same tag. The link points at the exact release source so the English
archive stays available alongside the localized notes.

## Installed Layout

The setup writes a split layout under the selected root:

```text
<install root>/
  install.json
  app/
    k580
    uninstaller
  bin/
    kr
  data/              # portable mode only
```

On Windows the file names include `.exe`.

## Modes

System mode is an OS-integrated install, not just copied files in a default
folder. It still uses the same split layout on disk, but the installer also
registers the app with the desktop environment:

- Windows current-user installs create a Start Menu shortcut under the user's
  Start Menu, optionally create a desktop shortcut, and register
  `uninstaller --uninstall <install root>` under the current user's
  Apps & Features uninstall list.
- Windows all-users installs use the shared Start Menu/Desktop locations and
  the machine uninstall registry key. These writes require the normal Windows
  rights for those locations.
- Linux/Unix installs create a user `.desktop` launcher under
  `~/.local/share/applications`, optionally create a desktop launcher, and
  remove those entries during uninstall.
- macOS installs are user-scoped, use `~/Applications/KR580` as their default
  root, and create a small `~/Applications/KR580.app` launcher wrapper for
  application search. The optional desktop action creates `KR580.command`.

Windows shortcut creation uses a hidden PowerShell child process with
`CREATE_NO_WINDOW`, so System installs do not flash a terminal while creating
Start Menu or desktop shortcuts.

On Windows, the standalone setup artifact uses
`crates/ui/assets/icons/installer-setup.ico` as its main PE icon. The installed
`app/uninstaller.exe` is a separate payload binary built from the same entry
logic with `crates/ui/assets/icons/installer-uninstall.ico`, so Explorer and Apps &
Features show the uninstall badge instead of the setup badge.

System mode stores application settings in the platform config directory:

- Windows: `%APPDATA%\KR580\settings.json`, falling back to `%LOCALAPPDATA%`.
- macOS: `~/Library/Application Support/KR580/settings.json`.
- Linux/Unix: `$XDG_CONFIG_HOME/kr580/settings.json`, falling back to `~/.config/kr580`.

A standalone macOS application bundle follows the same Application Support
path even though it has no `install.json`. A confined Snap stores settings in
`$SNAP_USER_COMMON/settings.json`, so refreshes share one writable settings
file instead of attempting to modify the read-only mounted package.

Portable mode defaults to `%USERPROFILE%\KR580` on Windows and `$HOME/KR580`
on Unix/macOS. It stores settings in `<install root>/data/settings.json`.
Temporary floppy-buffer and image files still use `std::env::temp_dir()`, so
throwaway files stay in the OS temp area instead of the portable data folder.
Portable mode does not create Start Menu/search entries, desktop shortcuts,
or uninstall registry/application entries. If its file-association checkbox is selected,
the `.580` and `.krs` associations point directly to that portable `app/k580` binary.
Running the portable `app/uninstaller` removes the recorded associations,
removes the exact `<install root>/bin` PATH entry if it exists, and then removes
the portable folder after the final Close/`Закрыть` action. Manual folder
deletion removes only the files; use `uninstaller` or `kr --unregister-file-type`
first if portable file associations were created and must be removed.

Uninstall is integrated into System mode. Windows registers `KR580` in Apps &
Features with an uninstall command that runs the installed `uninstaller
--uninstall <install root>`. That command opens the graphical uninstaller, shows
cleanup progress, removes the exact KR580 PATH entry, Start Menu/search
launcher, optional desktop shortcut, optional `.580` / `.krs` associations recorded in
`install.json`, uninstall registry entry, and then schedules the install root
for deletion after the user presses `Закрыть`. Linux/Unix and macOS remove their
user launcher entries and the managed PATH block through the same GUI flow
before deleting the install root.

Unpacked developer builds without `install.json` keep the legacy behavior and
write `settings.json` beside the executable unless they run inside a macOS app
bundle or Snap confinement.

## Scope And PATH

Windows system installs can target either the current user or all users. The
all-users option writes under `Program Files` by default and uses the machine
environment key for PATH, so it requires the normal Windows elevation rights.

Linux and macOS installs are user-scoped. Their PATH checkbox writes a managed
KR580 block to `~/.profile` on Linux/Unix and `~/.zprofile` on macOS.

The PATH checkbox adds only `<install root>/bin`, which contains `kr`. The GUI
binary lives under `<install root>/app`, so PATH does not expose `k580`.
