# Assets

Static assets shipped with the workspace live under `assets/`.

## `assets/brand/`

| File | Purpose |
|---|---|
| `logo.svg` | Vector KR580 wordmark used by `README.md` and `README-EN.md`. |

The brand SVG is documentation-facing and is not embedded into the application binary.

## `assets/fonts/`

| File | Purpose |
|---|---|
| `RobotoMono.ttf` | Embedded Unicode monospace font used by printer PDF output. |
| `OFL-RobotoMono.txt` | SIL Open Font License for the bundled Roboto Mono file. |

`kr580` keeps the platform font routing: `Segoe UI Variable` for
chrome and labels on Windows, and iced's generic monospace selector for
register, memory, command, and input readouts. Windows renders a covered
startup-only warmup layer for both paths while the main window is still
cloaked, so the renderer pays the cold glyph cost before the first
visible run. The internal `kr580` printer module embeds `RobotoMono.ttf`
through `include_bytes!`; PDF export therefore does not depend on fonts installed
on the host system and can render CP866-decoded Cyrillic consistently.

The font file is mirrored under `crates/ui/assets/fonts/` for the same reason
as the icon mirror: the published `kr580` crate must be self-contained.

## `assets/screenshots/`

| File | Purpose |
|---|---|
| `ru/emulator.png`, `en/emulator.png` | Full-emulator screenshots used by the default Russian `README.md` and English `README-EN.md`. |
| `ru/monitor.png`, `ru/floppy.png`, `ru/hdd.png`, `ru/network.png`, `ru/printer.png` | Russian external-device screenshots used by `README.md`. |
| `en/monitor.png`, `en/floppy.png`, `en/hdd.png`, `en/network.png`, `en/printer.png` | English external-device screenshots used by `README-EN.md`. |

These files are documentation screenshots. They are not embedded into the application binary.

## `assets/icons/`

| File | Purpose |
|---|---|
| `icon.png` | Application icon master. Treated as the source of truth; every `icon-*.png` and `icon.ico` is regenerated from it. |
| `icon-16.png`, `icon-32.png`, `icon-48.png`, `icon-64.png`, `icon-128.png`, `icon-256.png` | Standalone PNGs used at runtime (currently `icon-64.png` is embedded as the iced window icon) and reserved for future installer/desktop-entry packaging. |
| `icon.ico` | Multi-resolution Windows application icon containing `256, 96, 64, 48, 40, 32, 24, 20, 16` frames in that order so default Windows previewers (Photos, Paint, IconViewer) display the 256×256 layer when the file is opened directly. Embedded into the `.exe` PE resource via `winresource`. |
| `file-580.png` | `.580` file-type icon master. Treated as the source of truth; `file-580.ico` is regenerated from it. |
| `file-580.ico` | Multi-resolution Windows `.580` file-type icon containing `256, 128, 96, 64, 48, 40, 32, 24, 20, 16` frames. Embedded into the `.exe` PE resource as resource id `2` via `winresource`, so Explorer can show it for files associated with the application. |
| `installer-setup.png` | Standalone setup icon master. Treated as the source of truth for `installer-setup.ico`. |
| `installer-setup.ico` | Multi-resolution Windows setup icon containing `256, 128, 96, 64, 48, 40, 32, 24, 20, 16` frames. Embedded as the main PE icon when `KR580_WINDOWS_ICON_KIND=setup`. |
| `installer-uninstall.png` | Installed uninstaller icon master. Treated as the source of truth for `installer-uninstall.ico`. |
| `installer-uninstall.ico` | Multi-resolution Windows uninstaller icon containing `256, 128, 96, 64, 48, 40, 32, 24, 20, 16` frames. Embedded as the main PE icon when `KR580_WINDOWS_ICON_KIND=uninstaller`. |

The pre-rendered files are checked into the repository so the binary
does not have to decode or resize the master image at build or run time.

The same icon tree is mirrored under `crates/ui/assets/icons/`. The derived
PNG/ICO/SVG files are consumed by compile-time `include_bytes!` calls and by
the Windows resource build script, because a crates.io package cannot rely on
files outside the `kr580` crate root. The four large master PNGs are kept in
the repository mirror for regeneration but excluded from the published crate.
Regenerate icons through the scripts below instead of editing the mirror by
hand.

## Regenerating the icon set

The scripts read `assets/icons/icon.png`, `assets/icons/file-580.png`,
`assets/icons/installer-setup.png`, and
`assets/icons/installer-uninstall.png`, then rewrite every derived PNG/ICO in
one go.

### PowerShell (Windows)

```powershell
powershell -File scripts/generate_icons.ps1
```

Uses `System.Drawing` (built into .NET on Windows) – no external
dependency required.

### Bash (Linux/macOS/WSL)

```bash
./scripts/generate_icons.sh
```

Uses ImageMagick: prefers `magick` (v7+), falls back to `convert` (v6).
Install via your package manager:

- macOS: `brew install imagemagick`
- Debian/Ubuntu: `sudo apt install imagemagick`
- Arch: `sudo pacman -S imagemagick`

Both scripts run a Lanczos resample for every layer and strip metadata
to keep the PNG/ICO files small.

## Where the assets are consumed

- `crates/ui/src/app/windows.rs` embeds `crates/ui/assets/icons/icon-64.png` via
  `include_bytes!` and hands the bytes to
  `iced::window::icon::from_file_data`. This drives the title-bar /
  Alt-Tab / taskbar icon for the running application.
- `crates/ui/build.rs` (Windows only) embeds `crates/ui/assets/icons/icon.ico`
  into the PE resource section through the `winresource` crate. This
  drives the `.exe` icon shown by Explorer, the Start menu, pinned
  taskbar shortcuts, and the file picker.
- `crates/ui/build.rs` switches the main PE icon when
  `KR580_WINDOWS_ICON_KIND` is set: `setup` embeds
  `crates/ui/assets/icons/installer-setup.ico`, and `uninstaller` embeds
  `crates/ui/assets/icons/installer-uninstall.ico`.
- `crates/ui/build.rs` (Windows only) also embeds
  `crates/ui/assets/icons/file-580.ico` as PE resource id `2`. This drives the
  Explorer icon shown for `.580` files once the file association points at the
  built `.exe`.

When you replace any master PNG, run the appropriate script and rebuild.
`cargo` re-embeds `icon-64.png` automatically because it is an
`include_bytes!` source. The build script triggers a Windows-resource rebuild
via the selected main ICO and `cargo:rerun-if-changed=…/file-580.ico`.

## SVG icon sets

Two SVG icon families live alongside the PNG set:

| Directory | Purpose |
|---|---|
| `assets/icons/actions/` | Toolbar / menu / titlebar glyphs (`play`, `pause`, `step-forward`, `redo-dot`, `refresh-ccw`, `reset-ram`, `reset-registers`, `chevrons-right`, `cpu`, `clear-halt`, `binary`, `hard-drive-download`, `hard-drive-x`, `hard-drive-upload`, `bug`, `bug-off`, file/window/save/save-as/file-up/file-down, window caption buttons). Consumed by `crates/ui/src/view/icons.rs` through the `action_icon_bytes!` macro. |
| `assets/icons/devices/` | Peripheral chips on the bottom row of the schematic plate: `monitor.svg`, `floppy.svg`, `hdd.svg`, `network.svg`, `printer.svg`. Consumed through the `device_icon_bytes!` macro and exposed as `icons::device_monitor()` / `device_floppy()` / `device_hdd()` / `device_network()` / `device_printer()` getters. The chips are rendered by `view::schematic::device_chip` inside the `schematic_block_style` chassis with a hover tooltip wired the same way the action-panel buttons wire theirs. |

All SVGs are authored with `stroke="currentColor"` (or `fill="currentColor"`
for the solid HDD glyph) so iced's `svg::Style { color: Some(...) }`
callback can tint a single source file at any accent at runtime – no
per-colour duplicates. Files are embedded with `include_bytes!` at build
time via the two macros in `icons.rs`; replacing a glyph is a recompile,
not a runtime asset reload.
