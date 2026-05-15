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
- `.xlsx`: `rust_xlsxwriter` workbook with stable `CPU` and `Memory` sheets;
- `.docx`: `docx-rs` document with stable sections.

The UI exposes all three export actions, and `k580-app` routes them through the same `ExportModel` built from core state.
