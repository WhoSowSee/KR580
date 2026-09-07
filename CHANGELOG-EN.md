# Changelog

## [3.1.0] - 2026-09-07

### Features

- Added: Tab and Shift+Tab navigation through external device buttons, including detached windows and the monitor hexadecimal buffer, with Enter or Space activation and persistent selection after activation

### Bug Fixes

- Fixed: the stack pointer SP is correctly restored from `.580` files and preserved when saving again
- Fixed: registers A, B, C, D, E, H, L, W, and Z are restored from `.580` files and preserved when saving again
- Fixed: saving a `.krs` subprogram no longer marks changes to registers or memory outside the selected range as saved
- Fixed: a failed `.krs` subprogram save no longer damages the existing file
- Fixed: the `.krs` subprogram range follows the bytes actually loaded and is reused for subsequent saves

### Build and Packaging

- Fixed: Windows displays the terminal launcher `kr.exe` as KR while the GUI application `kr580.exe` retains the name KR580

## [3.0.0] - 2026-09-06

### Breaking Changes

- Changed: renamed the GUI executable from `k580` to `kr580`; updated the installer, launcher, associations, and packages
- Changed: the public `file_assoc::is_registered` function is now Windows-only, and `kr -u` on macOS returns an unsupported-operation error; corrected release artifact names, desktop integration ownership, Linux association rollback, and Settings responsiveness while applying associations

### Features

- Added: macOS application and document icons, `.580` and `.krs` file type declarations, and application bundle metadata
- Added: setting KR580 as the default application for `.580` and `.krs` from Settings on Linux and macOS

### Bug Fixes

- Fixed: the monitor uses a complete CP866 bitmap font for Cyrillic, box drawing, and other byte glyphs
- Fixed: Windows file association handling and registration of both `.580` and `.krs` extensions
- Fixed: removing Linux associations preserves handlers owned by other application installations
- Fixed: Linux file registration respects user XDG data directories
- Fixed: executable paths in Linux desktop entries are correctly quoted and escaped
- Fixed: Linux application and file type icons are embedded and available during association registration
- Fixed: Linux MIME and desktop database update failures are reported to the user
- Fixed: macOS association registration uses the existing application bundle
- Fixed: documents opened from Finder are handled through macOS events with unsaved-change checks
- Fixed: packaged applications use writable user settings directories
- Fixed: Snap hides the unavailable system file association action
- Fixed: file handlers use the consistent KR580 display name on Windows and Linux
- Fixed: focus, Tab navigation, and hexadecimal address input in subprogram Open and Save dialogs; the initial Enter action matches the Cancel button

### Build and Packaging

- Updated: Windows executable descriptions distinguish the application, installer, and uninstaller
- Updated: the visible Linux launcher is separate from the hidden file handler, so disabling associations keeps the application in the menu
- Updated: the Debian package includes desktop integration dependencies
- Updated: the macOS installer and file associations share one `KR580.app` bundle contract
- Fixed: Windows resources are skipped when building for other target platforms
- Updated: macOS application registration uses the public Launch Services API
- Added: macOS distribution as a DMG with drag-and-drop installation of `KR580.app` into Applications
- Updated: Linux metadata is centralized in shared templates for the application, installer, and packages
- Updated: Snap packages the ready-to-run GUI application instead of a nested installer
- Added: automated checks for Linux desktop integration and macOS DMG contents
- Fixed: Clippy errors on Rust 1.88 and the `lipo` argument order when building and verifying macOS DMGs

## [2.3.0] - 2026-09-04

### Features

- Added: contextual shortcuts for memory address search and cell value replacement
- Added: notices after resetting settings and keyboard shortcuts

### Bug Fixes

- Fixed: saving settings no longer resets the content scroll position
- Fixed: Tab navigation preserves the active inline memory editing mode
- Fixed: remapped memory editing shortcuts no longer type characters into the active field

## [2.2.0] - 2026-09-03

### Features

- Added: compact scrollbar to the monitor byte stream window
- Added: edge hints and smoother scrolling to Settings content
- Added: optional display of the open file name in the window title bar
- Added: persisted default monitor layout selection with Unified and Split options

### Bug Fixes

- Fixed: Import and Export replace active panels and switch directly between each other without restoring the previous panel
- Fixed: the visible RAM region remains in place when opening a device through Alt+Enter
- Fixed: Tab and Shift+Tab navigation on selected RAM cells outside edit mode
- Fixed: spacing between the export target field and its dropdown matches other selectors
- Fixed: scroll hint fades no longer cover the Settings dialog border

## [2.1.1] - 2026-09-01

### Bug Fixes

- Fixed: export target labels update immediately when the language changes
- Fixed: printer property inputs align with selector controls
- Fixed: printer capabilities localize independently of the driver locale
- Fixed: printer paper controls align consistently and no longer use a redundant group frame
- Fixed: import target dropdown layout and highlighting remain stable
- Fixed: the opcode picker uses the shared compact RAM scrollbar
- Fixed: scrollbar hit areas are wider and rail clicks no longer reach list content
- Fixed: mouse-wheel scrolling works while the pointer is over a scrollbar thumb
- Fixed: the opcode picker keeps its selection visible and its scrollbar responsive
- Fixed: clicks on compact scrollbar rails scroll the corresponding lists
- Fixed: the full top area of detached windows can be used for dragging
- Fixed: detached device windows cannot be resized, maximized, or tiled with Windows Snap

## [2.1.0] - 2026-08-30

### Features

- Added: native file dialogs parented to their owning windows, bounded XLSX column autofit, and XDG dialog dependencies in Linux packages
- Updated: installer and uninstaller with shared Tokyo Night chrome, real removal stages, safe post-exit cleanup, and smooth progress reporting

### Bug Fixes

- Fixed: closing the opcode picker now preserves the selected RAM cell
- Fixed: the export dialog now spells out Microsoft Excel
- Fixed: minimum supported Rust version set to 1.88

### Build and Packaging

- Updated: reduced dependency feature graph
- Updated: Snap package to Snapcraft 9 and the core24 base
- Updated: consolidated icon assets inside the UI crate and removed the duplicate repository-root tree

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
