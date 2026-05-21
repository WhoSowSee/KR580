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

Settings are UTF-8 JSON with `settingsVersion: 1` and top-level `network`, `storage`, `export`, `ui`, and `recentFiles` fields.

## Exports

Exporters use direct generators and never scrape UI widgets:

- `.txt`: stable plain-text sections;
- `.xlsx`: `rust_xlsxwriter` workbook with stable `CPU` and `Memory` sheets.

The UI exposes both export actions, and `k580-app` routes them through the same `ExportModel` built from core state.

## Imports

`k580-persistence::Importers` round-trips the same two formats back into an `ExportModel`, and `ExportModel::apply_to(&mut Cpu8080State)` writes the parsed registers, flags, and memory cells into a CPU state. The XLSX reader uses `calamine`; the TXT reader parses the same `[Registers]/[Flags]/[Memory]` sections that the exporter emits, so a file written by `Exporters::write_txt` reloads byte-for-byte.
