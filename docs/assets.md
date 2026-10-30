# Assets

Documentation artwork lives under `assets/`; build/runtime assets live under
`crates/ui/assets/`. Source images and derived files have separate roles.

## README star history

Four SVGs under `.github/assets/` cover Russian/English and light/dark READMEs.
The READMEs select absolute raw GitHub URLs so charts also work on crates.io.
They link to the repository's stargazers.

Regenerate from the repository root:

```sh
python3 .github/scripts/gen_star_history.py --repo WhoSowSee/KR580
python3 -B .github/scripts/test_gen_star_history.py
```

The standard-library generator orders daily counts and renders a deterministic
cumulative series without timestamps. Empty/single-day series are supported;
unchanged data yields unchanged SVGs. `GITHUB_TOKEN` is optional for public
local requests.

The scheduled workflow refreshes Monday/Thursday at 06:17 UTC and supports
manual dispatch. Repository/App permissions and publishing are documented in
[Star-history publisher](maintenance.md#star-history-publisher).

## Documentation artwork

`assets/brand/logo.svg` is the README wordmark. Localized screenshots under
`assets/screenshots/{ru,en}/` show the main app and five device windows.
These files are documentation inputs and are not embedded in the application.

## Monitor bitmap font

`crates/ui/src/view/monitor_font/cp866.rs` contains all 256 CP866 glyph slots.
The 8×8 bold glyphs are mapped from Alex Frush's public-domain
[512_8 font](https://github.com/alexfru/512_8/tree/726f6fa1d09775f9962a47fe78ce1aeacaa4cf34).
Byte indexing keeps Canvas and image export independent of OS code pages,
installed fonts and rasterizers.

`monitor_font/mod.rs` defines 8×12 cell geometry and lookup. The checked-in table
needs no asset-generation/build step; tests cover byte slots and text sequencing.
Application chrome uses platform UI fonts and monospace readouts separately.

## Icon sources

All masters and derivatives are under `crates/ui/assets/icons/`.

| Master | Derived output | Consumers |
|---|---|---|
| `icon.png` | `icon-*.png`, `icon.ico`, `KR580.icns` | Runtime window, Windows executable, Linux desktop/package and macOS app icons |
| `file-580.png` | `file-580-256.png`, `file-580.ico`, `KR580Document.icns` | `.580`/`.krs` document integration |
| `installer-setup.png` | `installer-setup.ico` | Standalone Windows setup |
| `installer-uninstall.png` | `installer-uninstall.ico` | Installed Windows uninstaller |

`icon-64.png` is embedded for runtime windows. `icon-256.png` is used by Linux
integration and packages. ICOs are
multi-resolution; keep the largest frame first for Windows image previewers.

Derived PNG/ICO/ICNS files are committed so ordinary builds need no master-image
resize step. Masters remain available for regeneration but are excluded from
the published crate. Keeping build inputs inside the UI crate also lets packaged
include_bytes paths and resources resolve without a workspace-level mirror.

## Regenerating the icon set

After changing a master, regenerate all derivatives and rebuild:

```sh
just icons
```

Direct commands:

```powershell
powershell -File scripts/generate_icons.ps1
```

```sh
bash scripts/generate_icons.sh
```

PowerShell uses Windows System.Drawing. Bash uses ImageMagick (`magick`, falling
back to `convert`); install it through the platform package manager. The scripts
read all four masters. Raster sizes and frame sets are defined in the generators.

ICNS conversion uses Apple's iconutil where available, otherwise Python's
standard library assembles PNG layers. The encoder is checked before rewriting
outputs; ICNS staging validates its signature and length before replacement.
Native DMG verification additionally decodes both ICNS files with iconutil.

## Where the assets are consumed

- Runtime windows embed the 64 px PNG and create an iced icon from RGBA pixels.
- Windows build resources assign product names/icons per binary: KR for the
  launcher, KR580 for the GUI, and the setup/uninstall roles for their payloads.
  `KR580_WINDOWS_ICON_KIND` selects those build roles.
- The document ICO is embedded as PE resource 2 for Explorer associations.
- Linux `integration_assets` embeds the application/document 256 px PNGs for
  XDG hicolor paths. Native package builders use the same canonical files.
- macOS bundles install application/document ICNS files referenced by the
  canonical Info.plist.

The Windows build script generates/links resources for each binary separately.
A combined build must not apply the launcher's description to the GUI.
Changing the selected ICO/document ICO triggers its resource rebuild; changes
to embedded PNG/SVG files trigger ordinary recompilation.

## SVG icon sets

`actions/` contains toolbar/menu/window glyphs and `devices/` contains the five
peripheral glyphs. They are embedded at build time and share currentColor-based
tinting. SVG changes require rebuilding, not PNG/ICO regeneration.

The update menu uses the [Lucide Download glyph](https://github.com/lucide-icons/lucide/blob/main/icons/download.svg)
and cached rotation frames of the shared refresh glyph. Keep the filled error
glyph's dot separated from its stem so it remains legible at menu size.
