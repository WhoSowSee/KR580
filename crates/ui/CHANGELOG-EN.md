# Changelog

## [2.0.0] - 2026-07-29

### Breaking Changes

- Removed the unused public `CoreCommand` and `CoreEvent` types, `Memory64K::write_word`, and the `ValidationError::InvalidRegister` and `CoreError::Validation` variants

### Features

- Added: localized in-app changelog reader
- Added: raw `.krs` subprogram loading and saving with a selectable RAM range, plus system `.krs` file associations
- Added: pointer-hover switching between open top-level menus
- Added: drag-and-drop opening for `.580` and `.krs` files with a highlighted drop surface, a cursor-following hint, and confirmation before discarding unsaved changes
- Updated: import dialog with integrated file selection and drag-and-drop support

### Bug Fixes

- Fixed: active emulation speed being lost after closing settings without saving
- Fixed: settings search filtering categories but not individual options
- Fixed: cancelled file dialogs clearing the unsaved-changes state
- Fixed: an unnecessary border around the selected theme
- Fixed: duplicated translation tables and remaining unlocalized interface text
- Fixed: export target scrolling and overflow of long target names
- Fixed: checkbox styling in the export dialog
- Fixed: modal focus indicators and export/import target dropdown behavior
- Fixed: floppy image example encoding and formatting
- Fixed: floppy and HDD contents not refreshing after external file changes
- Fixed: export target add and delete actions using a blue keyboard-focus border instead of a white one
- Fixed: the `kr580` crates.io package failing verification because the embedded changelog files were outside the package

### Documentation

- Updated: README feature callouts and screenshot gallery organization

## [1.1.0] - 2026-07-21

### Features

- Added: configurable keyboard shortcuts
- Added: Alt+Shift+Enter shortcut for returning from a memory operand target
- Added: additional dark and light color schemes
- Added: native printer setup and driver properties, replacing built-in PDF export
- Added: precise virtualized memory scrollbar

### Bug Fixes

- Fixed: overly complex unified monitor screen title
- Fixed: oversized On/Off toggles in the English locale
- Fixed: device panels remaining open behind modal dialogs
- Fixed: operand highlighting being disabled in default settings
- Fixed: settings closing after save and leaking modal interactions
- Fixed: help article scrolling and unnecessary settings scrolling
- Fixed: unbalanced mnemonic text size in the processor schematic
- Fixed: inconsistent schematic tooltip text size
- Fixed: keyboard navigation and focus indicators across menus and dialogs
- Fixed: detached printer dialogs and modal interaction stability
- Fixed: default schematic column spacing
- Fixed: capitalization of localized device footer values
- Fixed: printer property localization depending on the driver language

### Documentation

- Updated: README image URLs for crates.io and docs.rs rendering
- Fixed: README source run and build commands
- Updated: localized UI screenshots
- Updated: in-app help structure and user-facing content

## [1.0.0] - 2026-06-23

### Added

- Desktop KR580 emulator with deterministic CPU state, 64 KiB RAM, interrupts, halt state, cycle counters, and tact-level execution
- Native iced GUI with RAM editing, register editing, status register view, instruction stepping, tact stepping, paced run, and burst run modes
- External device windows for monitor, floppy, HDD, network adapter, and printer through typed `IoBus` ports
- Versioned `.580` snapshots, raw `.krs` subprogram loading, TXT/XLSX import and export, and printer PDF export
- Graphical installer, graphical uninstaller, terminal launcher, optional `.580` and `.krs` file associations, and portable/system install modes
- Release packaging pipeline for installer artifacts, Linux Debian packages, Snap packages, GitHub Actions artifacts, and tag-based GitHub releases
