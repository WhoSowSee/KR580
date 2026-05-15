# Persistence (`kr580-persistence`)

Versioned, deterministic persistence: snapshots, subprograms, settings,
and exporters. All readers / writers are typed; no string-typed errors.

## Files

| File              | Responsibility                                  |
| ----------------- | ----------------------------------------------- |
| `error.rs`        | `PersistenceError`, `SnapshotError`, `ExportError` |
| `snapshot.rs`     | `.580` versioned binary TLV snapshot            |
| `subprogram.rs`   | `.krs` subprogram (raw bytes + base address)    |
| `settings.rs`     | UTF-8 JSON settings with schema migration       |
| `export.rs`       | Plain-text exporter via a shared `ExportView`   |

## `.580` snapshot

```
magic    : "K580"   (4 bytes)
version  : u16 LE
length   : u32 LE
payload  : sequence of TLV blocks (tag u8, len u32 LE, value bytes)
```

Required tags (per `prompt/04_file_formats.md`):

| Tag    | Name            | Value                                                       |
| ------ | --------------- | ----------------------------------------------------------- |
| `0x01` | RAM             | 65536 bytes                                                 |
| `0x02` | registers       | A B C D E H L (7 bytes)                                     |
| `0x03` | flags           | one PSW byte                                                |
| `0x04` | PC              | u16 LE                                                      |
| `0x05` | SP              | u16 LE                                                      |
| `0x06` | interrupt state | `ie`, `ie_pending`, `irq_pending`, `has_vector`, `vector`   |
| `0x07` | halt state      | one byte                                                    |
| `0x08` | timing          | `cycle_count` u64 LE + optional `tact_phase` u8             |

* Unknown low-bit tags fail with `SnapshotError::UnsupportedTag`.
* Unknown high-bit tags (`tag & 0x80 != 0`) are skipped silently.
* Bad magic, wrong version, missing required tag, or truncated input return
  the matching typed error.

`SnapshotError::MissingTag(0x07)` etc. is the canonical migration signal:
add a migration step here when a future tag becomes mandatory.

## `.krs` subprogram

Tiny TLV-free header so we can recover the exact length and base the user
chose at save time:

```
"KRS1"   (4)
base     u16 LE
length   u32 LE
payload  length bytes
```

`SubprogramSerializer` is the only reader/writer. The base address is
provided by the caller; there is no implicit GUI metadata.

## Settings

UTF-8 JSON with `settingsVersion: 1`. Loader:

* missing version → migrated to current,
* future version → typed error,
* unknown fields are accepted (forward compatible).

Network mode is *explicit* (`client` / `server`); there is no auto mode.
Storage paths come from settings, not from the process working directory.

## Exporters

The first iteration ships only a plain-text exporter (`TxtExporter`). It
reads `ExportView`, which is built from `Cpu8080State` directly — never
from UI controls. Adding an `.xlsx` or `.docx` writer is a matter of
adding a sibling exporter; the source-of-truth contract is already in
place.
