# Persistence

## `.580`

Snapshots use an exact 65,549-byte layout without magic or version fields:

| Offset | Length | Value |
|---|---|---|
| 0–65,535 | 65,536 | RAM |
| 65,536–65,544 | 9 | A, B, C, D, E, H, L, W, Z |
| 65,545–65,546 | 2 | Little-endian PC |
| 65,547–65,548 | 2 | Little-endian SP |

All SP values are valid, including `FFFFh`; the last bytes are not a signature.
Loading creates a default CPU and restores only these fields. Flags, IRQ, halt
and timing retain their CPU defaults. Devices and application preferences are
not serialized by this format.

Snapshot/subprogram path checks are case-insensitive. Empty, short or wrong-size
inputs report the format's typed error. Loaders share a bounded reader: at most
65,550 bytes for snapshots and `65,536 - start + 1` for subprograms. The extra
byte detects overflow.

Metadata rejects known oversized files before reading, but validation still
uses actual bytes because a file can grow after metadata. A failed input never
partially changes CPU state.

## `.krs`

Subprograms contain raw RAM bytes without an embedded base address. Open supplies
a start address; Save supplies an inclusive start/end range.
`load_into_state` returns the actual inclusive installed end address and rejects
overflow before mutating RAM. The UI records that result from its matching
completion rather than estimating from metadata.

A successful `.krs` save updates the dirty baseline only for the saved range.
Register edits and RAM outside it remain unsaved. See
[document flows](ui_app.md#unsaved-changes-modal).

## Atomic saves

Snapshots, subprograms and settings share staged replacement. The writer creates
a unique sibling temporary file, writes and synchronizes it, closes it and
replaces the destination. Write/flush/replacement failure preserves the previous
destination and attempts temporary-file cleanup.

Existing symbolic links are resolved before staging; dangling links are rejected.
Existing permissions are copied and read-only targets are rejected. Replacement
changes file identity, so hard links retain the previous content.

This protects against ordinary save failures, not every filesystem/power-loss
scenario. Abrupt process exit can leave a temporary file.

`persistence::write_file_atomic(path, bytes)` exposes the same contract for raw
outputs. Monitor image and storage-buffer exports run it on the UI file worker
and report Saved after successful replacement. Encoded images do not return
through the completion channel.

## Settings

Settings are UTF-8 JSON with `settingsVersion: 11` and top-level `network`,
`storage`, `export`, `ui`, `general`, `shortcuts` and `recentFiles` fields.
Filesystem locations depend on [installation mode](installer.md#modes).

The loader migrates supported versions 1–10; other versions return
`SettingsError::UnsupportedVersion`. Compatibility rules include:

- Version 1 network endpoints reset to defaults, preserving non-network settings.
- Versions 1–4 discard the obsolete printer name; versions 1–5 use the default
  custom setup-dialog mode.
- A remaining name-only printer becomes a configuration resolved against current
  driver defaults when setup opens. Full configurations retain validated DEVMODE
  bytes; presets retain complete named configurations.
- `general.printerSettings` is authoritative; `printerName` is a synchronized
  compatibility mirror.
- Legacy `dark` and `monokai` themes resolve to Tokyo Night.
- Older shortcut maps preserve custom bindings rather than enabling new default
  RAM chords over conflicting actions. Overrides are normalized after migration.

Absent file-name and monitor-split preferences default to false. An empty
`shortcuts.bindings` uses built-in defaults, including context-specific RAM
search/replacement. User-facing preference behavior belongs to
[Settings](ui_settings.md).

Settings writes return `SettingsError`. Directory creation or replacement
failure must be handled before reporting success. Background operations read
the current file and apply their owned change in FIFO order. Confirmed cache
and draft/cancel semantics are defined in
[live preview and saving](ui_settings.md#live-preview-and-saving).

## Exports

Exporters consume an `ExportModel`, independent of UI widgets. Each target
selects registers, flags and an inclusive RAM range. Every selected address is
included, even when its byte is zero.

| Format | Representation |
|---|---|
| TXT | `[Registers]`, `[Flags]` and `[Memory]`; memory uses `address=value` lines |
| Multi-section TXT | A named header followed by each target's three data blocks |
| XLSX | One worksheet per page, with selected register/flag/memory tables |

XLSX column options control address, value, command and comment columns. TXT
keeps its fixed text representation and does not use those column switches.
XLSX autofit samples at most the first 512 rows and caps width at 300 pixels,
preventing long tables or strings from producing impractical worksheets.

UI and TXT export share `text_section_names`: trim outer whitespace, replace
each CR/LF with a space, use `Section N` for blank names and append ` (section)`
to reserved `Memory`, `Registers` or `Flags`. Collisions after normalization gain
numeric suffixes from ` (2)`. Existing ordinary names and header syntax remain
compatible.

With address/value data present, reimport restores the complete selected range
and leaves outside RAM unchanged. Older sparse files remain readable; omitted
addresses retain their current values.

## Imports

Importers produce an `ExportModel` from TXT or XLSX. The default XLSX path reads
the first worksheet; sheet-name and per-sheet APIs support explicit selection.
TXT supports one plain model or named blocks through section-name/per-section
APIs.

Before CPU mutation, import validates register names, values and flags into an
owned typed patch. Applying a valid patch cannot fail; invalid models leave CPU
unchanged. Subprogram loads likewise validate their actual byte range first.

Workers return the patch/block to the actor, which applies it to the current
CPU rather than an old pre-read copy. A changed document generation rejects an
older completion and its error before it can change data or document state.
