# Devices (`kr580-devices`)

Routing IoBus and asynchronous peripheral workers.

## Port map (per `prompt/03_peripherals.md`)

| Port | Device          |
| ----:| --------------- |
| 00h  | Monitor         |
| 01h  | Floppy storage  |
| 02h  | HDD storage     |
| 03h  | Network adapter |
| 04h  | Printer         |

Unmapped ports return `0xFF` (open bus); writes are dropped.

## Files

| File          | Responsibility                                   |
| ------------- | ------------------------------------------------ |
| `error.rs`    | `DeviceError` (NotReady, Busy, Timeout, Disconnected, …) |
| `bus.rs`      | `DeviceBus` impl of `IoBus`, `DeviceStatus`      |
| `monitor.rs`  | Synchronous monitor state machine                |
| `storage.rs`  | Tokio-backed floppy / HDD writer                 |
| `network.rs`  | Tokio-backed TCP `client` / `server` device      |
| `printer.rs`  | In-memory spooled printer                        |

## Hot path

`DeviceBus::write(port, value)` and `DeviceBus::read(port)` are O(1) and
non-blocking. Heavy work (disk, network) happens inside dedicated Tokio
tasks; the synchronous bus methods only enqueue / drain.

## Error surfacing

Per `prompt/08_peripheral_edge_cases.md`, errors are device state — never
panics, never modal dialogs. Each device exposes a `snapshot_status()`
method that returns a serde-friendly status struct, which the UI renders
and which exporters can dump into a textual report.

* `StorageStatus { path, bytes_written, tail_buffer, last_error, worker_alive }`
* `NetworkStatus { mode, address, connected, rx_pending, rx_total, tx_total, last_error }`
* `PrinterStatus { bytes_buffered, spool_text, last_error }`

## Network

* explicit mode at construction (`NetworkMode::Client` or `NetworkMode::Server`);
* never auto-detected;
* read returns `0xFF` if no byte is queued (non-fatal no-data);
* disconnect / refused / timeout are written to `last_error` and `connected`.

## Storage

* writes are queued through `tokio::mpsc`;
* tail buffer is a snapshot, not the source of truth;
* the host file is the canonical destination;
* file open errors map to typed `DeviceError` variants.

## Tests

Each device has at least one async-aware test. `storage` and `network`
tests use `#[tokio::test]` to drive the worker loops on the test runtime.
