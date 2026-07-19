# Persistence

## `.580`

Snapshots are versioned little-endian binary files:

- magic `K580`;
- `u16` version (`1`);
- `u32` payload length;
- TLV payload using the tag registry from `prompt/04_file_formats.md`.

Unknown low-bit tags fail with `SnapshotError::UnsupportedTag`; high-bit extension tags are skipped.

Snapshot tests verify roundtrip fidelity, deterministic byte output, unsupported-version rejection, payload-length validation, and high-bit extension tag skipping.

## `.krs`

Subprograms are raw byte slices. The base address is supplied by the caller and is not hidden in the file.

`SubprogramSerializer::load_into_state` rejects memory overflows before mutating RAM. This keeps `.krs` deterministic and avoids adding a second headered format.

## Settings

Settings are UTF-8 JSON with `settingsVersion: 8` and top-level `network`, `storage`, `export`, `ui`, `general`, `shortcuts`, and `recentFiles` fields. Loading version 1 preserves non-network preferences but resets the legacy runtime-written client/server endpoints to `127.0.0.1:5800`; version 2 adds the default shortcut map; version 3 migrates the old `ui.theme: "dark"` value to `tokyoNight`; version 4 stores the selected `ColorScheme`; version 5 adds the printer dialog mode; version 6 adds the selected printer name. Version 7 adds `general.printerSettings`, containing the printer name, paper/source identifiers and labels, orientation, and the validated driver `DEVMODEW` bytes. Version 8 adds `general.printerPresets`; each `{ name, settings }` entry stores a complete validated configuration for one printer, including its driver-private `DEVMODEW`. A version 6 printer name is migrated to a name-only configuration and resolved against the current driver defaults when setup is opened, while version 7 migrates with an empty profile list. `general.printerSettings` is the runtime source of truth; `general.printerName` remains a synchronized compatibility mirror for older settings files. `tokyoNight` is the default theme, and the retired `monokai` value also falls back to `tokyoNight`. Customizable keyboard shortcut overrides live under `shortcuts.bindings`; an empty list means the built-in Ctrl/Shift/Alt layout is active, including the default `Alt+Enter` memory-cell action and `Shift+Alt+Enter` memory-cell return action.

## Exports

Exporters use direct generators and never scrape UI widgets:

- `.txt`: stable plain-text blocks; a single export keeps the
  `[Registers]`, `[Flags]`, and `[Memory]` sections, while text exports
  with several UI sections write one named block per section;
- `.xlsx`: a `rust_xlsxwriter` workbook whose worksheets contain the
  register table, flag table, and memory table for each selected page.

`ExportOptions` lets the UI pass a worksheet name, memory address range,
memory-table column toggles, register selection, flag selection, and
optional XLSX pages or text-export named sections. The memory
range, register selection, and flag selection are applied while building
`ExportModel`, so both TXT and XLSX exports use them. XLSX additionally
uses the page name and optional comment column and, when page entries
are present, writes each entry as a worksheet in one workbook. TXT
ignores the comment column and, when section entries are present, writes
each entry as `[Section name]` followed by its own `[Registers]`,
`[Flags]`, and `[Memory]` block.

The UI exposes both export actions, and the internal `kr580` backend routes them through the same `ExportModel` built from core state.

## Imports

`persistence::Importers` round-trips the same two formats back into an `ExportModel`, and `ExportModel::apply_to(&mut Cpu8080State)` writes the parsed registers, flags, and memory cells into a CPU state. The XLSX reader uses `calamine`; by default it imports the first worksheet, while `xlsx_sheet_names()` and `read_xlsx_sheet()` let the UI present and import a specific worksheet from a multi-page export. The TXT reader parses the same `[Registers]`, `[Flags]`, and `[Memory]` sections that the exporter emits. Plain TXT files still import as one model; multi-section text exports expose their named blocks through `txt_section_names()` and `read_txt_section()`.
