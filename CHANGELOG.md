# Changelog

## [1.0.0] - 2026-06-23

### Added

- Desktop KR580 emulator with deterministic CPU state, 64 KiB RAM, interrupts, halt state, cycle counters, and tact-level execution
- Native iced GUI with RAM editing, register editing, status register view, instruction stepping, tact stepping, paced run, and burst run modes
- External device windows for monitor, floppy, HDD, network adapter, and printer through typed `IoBus` ports
- Versioned `.580` snapshots, raw `.krs` subprogram loading, TXT/XLSX import and export, and printer PDF export
- Graphical installer, graphical uninstaller, terminal launcher, optional `.580` file association, and portable/system install modes
- Release packaging pipeline for installer artifacts, Linux Debian packages, Snap packages, GitHub Actions artifacts, and tag-based GitHub releases
