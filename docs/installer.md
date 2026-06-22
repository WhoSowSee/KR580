# Installer

`k580-ui` builds four desktop-facing binaries:

- `k580` - the GUI emulator.
- `kr` - the terminal launcher and file-association helper.
- `k580-installer` - the graphical installer.
- `k580-uninstaller` - the graphical uninstaller payload installed as
  `app/uninstaller`.

The user-facing setup artifact is built by `scripts/build_installer.ps1`
on Windows or `scripts/build_installer.sh` on Unix/macOS. The scripts first
build `k580` and `kr`, then build `k580-uninstaller` with the uninstall icon,
then rebuild `k580-installer` with the setup icon and those binaries embedded.
The resulting file under `dist/` is the installer a new user runs before `kr`
exists.

`kr --install` remains a maintenance entry point for developer builds. In an
installed layout it launches the installed uninstaller binary with `--setup`,
so the setup UI is explicit and the installed file still reads as an
uninstaller.

## Installer Window

The graphical setup uses an undecorated iced window with its own monochrome
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
system language use English strings.

The installer surface is a single black panel with no left information rail and
no secondary role label after `KR580 Setup` in the custom title bar. Option
tiles, the folder field, Browse, and the primary install button use fixed
heights so text cannot drift below the button body or overlap neighbouring
controls.
The Browse control is a compact `96x42` secondary button beside the `44px`
folder field. The result area sizes to its current state: Ready and error
messages stay compact, Installing adds a progress bar, and the installed report
expands only when it has real content. After installation, equal vertical
spacers center the post-install checkbox between the installed report and the
Done action.
While files are being copied, the result area switches to an Installing state
with a monochrome progress bar that advances on an installer-only timer tick.
After a successful install, portable installs offer a checked "Open installation
folder" action, and system installs offer a checked "Launch KR580" action. The
`Done` button stays pinned at the bottom, runs the selected finish action, and
closes the installer; if that action fails, the installer stays open and shows
the error below the checkbox.
System mode also shows a checked "Create desktop shortcut" option. Both System
and Portable mode show a checked "Associate .580 files with KR580" option.
Portable mode hides Windows scope because it always installs to the selected
folder for the current user.
Selected option tiles keep a dark fill with a white border instead of an
inverted white slab, so the two-column form keeps even visual weight.
Installed-state messages use user-facing wording and do not expose the internal
`app/`, `bin/`, or portable data folders.
The default window is `680x760` logical pixels with a `640x720` minimum so the
primary Install action remains visible with 2x DPI scaling.

## Uninstaller Window

The installed `app/uninstaller` is also an iced GUI binary. Running it directly
from an installed layout or through `uninstaller --uninstall <install root>`
opens a compact undecorated black/white window with the same custom chrome,
DWM rounded corners on Windows, and SVG caption buttons as the installer. The
content is intentionally simpler than setup: install folder, status panel,
monochrome progress bar, and the final localized Close/`Закрыть` button.

Uninstall cleanup starts after the window opens so the user sees immediate
progress instead of a silent background operation. The first phase removes the
managed PATH entry, Start Menu/search launcher, optional desktop shortcut,
optional `.580` association recorded in `install.json`, and uninstall
registration. The install directory itself is scheduled for deletion only when
the user presses the final Close/`Закрыть` button, letting the uninstaller
process exit before Windows or Unix removes the folder that contains it.

## Build The Setup

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_installer.ps1
```

Unix/macOS:

```sh
bash scripts/build_installer.sh
```

The scripts produce:

- Windows: `dist/KR580-Setup-<version>-windows-<arch>.exe`
- Unix/macOS: `dist/KR580-Setup-<version>-<os>-<arch>`

If the Windows target artifact is locked by a running installer, the PowerShell
script writes the same setup under a numbered suffix such as
`KR580-Setup-<version>-windows-<arch>-1.exe` instead of failing after the
release build has already completed.

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
`assets/icons/installer-setup.ico` as its main PE icon. The installed
`app/uninstaller.exe` is a separate payload binary built from the same entry
logic with `assets/icons/installer-uninstall.ico`, so Explorer and Apps &
Features show the uninstall badge instead of the setup badge.

System mode stores application settings in the platform config directory:

- Windows: `%APPDATA%\KR580\settings.json`, falling back to `%LOCALAPPDATA%`.
- macOS: `~/Library/Application Support/KR580/settings.json`.
- Linux/Unix: `$XDG_CONFIG_HOME/kr580/settings.json`, falling back to `~/.config/kr580`.

Portable mode defaults to `%USERPROFILE%\KR580` on Windows and `$HOME/KR580`
on Unix/macOS. It stores settings in `<install root>/data/settings.json`.
Temporary floppy-buffer and image files still use `std::env::temp_dir()`, so
throwaway files stay in the OS temp area instead of the portable data folder.
Portable mode does not create Start Menu/search entries, desktop shortcuts,
or uninstall registry/application entries. If its `.580` checkbox is selected,
the file association points directly to that portable `app/k580` binary.
Running the portable `app/uninstaller` removes the recorded `.580` association,
removes the exact `<install root>/bin` PATH entry if it exists, and then removes
the portable folder after the final Close/`Закрыть` action. Manual folder
deletion removes only the files; use `uninstaller` or `kr --unregister-file-type`
first if a portable `.580` association was created and must be removed.

Uninstall is integrated into System mode. Windows registers `KR580` in Apps &
Features with an uninstall command that runs the installed `uninstaller
--uninstall <install root>`. That command opens the graphical uninstaller, shows
cleanup progress, removes the exact KR580 PATH entry, Start Menu/search
launcher, optional desktop shortcut, optional `.580` association recorded in
`install.json`, uninstall registry entry, and then schedules the install root
for deletion after the user presses `Закрыть`. Linux/Unix and macOS remove their
user launcher entries and the managed PATH block through the same GUI flow
before deleting the install root.

Unpacked developer builds without `install.json` keep the legacy behavior and
write `settings.json` beside the executable.

## Scope And PATH

Windows system installs can target either the current user or all users. The
all-users option writes under `Program Files` by default and uses the machine
environment key for PATH, so it requires the normal Windows elevation rights.

Linux and macOS installs are user-scoped. Their PATH checkbox writes a managed
KR580 block to `~/.profile` on Linux/Unix and `~/.zprofile` on macOS.

The PATH checkbox adds only `<install root>/bin`, which contains `kr`. The GUI
binary lives under `<install root>/app`, so PATH does not expose `k580`.
