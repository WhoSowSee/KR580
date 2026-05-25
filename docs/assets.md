# Assets

Static assets shipped with the workspace live under `assets/`. Today this
is just the icon set for the desktop binary.

## `assets/icons/`

| File | Purpose |
|---|---|
| `icon.png` | Master image. Treated as the source of truth; everything else is regenerated from it. |
| `icon-16.png`, `icon-32.png`, `icon-48.png`, `icon-64.png`, `icon-128.png`, `icon-256.png` | Standalone PNGs used at runtime (currently `icon-64.png` is embedded as the iced window icon) and reserved for future installer/desktop-entry packaging. |
| `icon.ico` | Multi-resolution Windows icon containing `256, 96, 64, 48, 40, 32, 24, 20, 16` frames in that order so default Windows previewers (Photos, Paint, IconViewer) display the 256×256 layer when the file is opened directly. Embedded into the `.exe` PE resource via `winresource`. |

The pre-rendered files are checked into the repository so the binary
does not have to decode or resize the master image at build or run time.

## Regenerating the icon set

The scripts read `assets/icons/icon.png` and rewrite every other file in
the directory.

### PowerShell (Windows)

```powershell
powershell -File scripts/generate_icons.ps1
```

Uses `System.Drawing` (built into .NET on Windows) — no external
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

- `crates/ui/src/main.rs` embeds `assets/icons/icon-64.png` via
  `include_bytes!` and hands the bytes to
  `iced::window::icon::from_file_data`. This drives the title-bar /
  Alt-Tab / taskbar icon for the running application.
- `crates/ui/build.rs` (Windows only) embeds `assets/icons/icon.ico`
  into the PE resource section through the `winresource` crate. This
  drives the `.exe` icon shown by Explorer, the Start menu, pinned
  taskbar shortcuts, and the file picker.

When you replace the master `icon.png`, run the appropriate script and
rebuild. `cargo` re-embeds `icon-64.png` automatically because it is an
`include_bytes!` source. The build script triggers a Windows-resource
rebuild via `cargo:rerun-if-changed=…/icon.ico`.

## SVG icon sets

Two SVG icon families live alongside the PNG set:

| Directory | Purpose |
|---|---|
| `assets/icons/actions/` | Toolbar / menu / titlebar glyphs (`play`, `pause`, `step-forward`, `redo-dot`, `refresh-ccw`, `reset-ram`, `reset-registers`, `chevrons-right`, `cpu`, `clear-halt`, file/window/save/save-as/file-up/file-down, window caption buttons). Consumed by `crates/ui/src/view/icons.rs` through the `action_icon_bytes!` macro. |
| `assets/icons/devices/` | Peripheral chips on the bottom row of the schematic plate: `monitor.svg`, `floppy.svg`, `hdd.svg`, `network.svg`, `printer.svg`. Consumed through the `device_icon_bytes!` macro and exposed as `icons::device_monitor()` / `device_floppy()` / `device_hdd()` / `device_network()` / `device_printer()` getters. The chips are rendered by `view::schematic::device_chip` inside the `schematic_block_style` chassis with a hover tooltip wired the same way the action-panel buttons wire theirs. |

All SVGs are authored with `stroke="currentColor"` (or `fill="currentColor"`
for the solid HDD glyph) so iced's `svg::Style { color: Some(...) }`
callback can tint a single source file at any accent at runtime — no
per-colour duplicates. Files are embedded with `include_bytes!` at build
time via the two macros in `icons.rs`; replacing a glyph is a recompile,
not a runtime asset reload.
