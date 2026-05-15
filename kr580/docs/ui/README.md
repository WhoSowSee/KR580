# UI (`kr580-ui`)

iced-based view layer with a separate runtime / actor module.

## Files

| File         | Responsibility                                              |
| ------------ | ----------------------------------------------------------- |
| `runtime.rs` | Spawns the deterministic core actor on its own thread; defines `UiCommand`, `UiEvent`, and the `StateView` snapshot. |
| `view.rs`    | `EmulatorApp` (iced `Application`), all widgets, all message routing. |

## Contract

The UI **never** owns emulator state. It holds:

* the latest `Arc<StateView>` snapshot received from the core actor,
* user input drafts (register hex strings, memory address text),
* the `RuntimeHandles { commands, events }` to send commands and receive
  events.

## Commands

| UI button / menu | `UiCommand`              |
| ---------------- | ------------------------ |
| Step             | `StepInstruction`        |
| Run / Stop       | `Run`, `Stop`            |
| Reset CPU        | `ResetCpu`               |
| Reset Regs       | `ResetRegisters`         |
| Reset RAM        | `ResetRam`               |
| Save / Load .580 | `SaveSnapshot`, `LoadSnapshot(bytes)` |
| Reg edit (Set)   | `SetRegister(reg, byte)` |
| Memory edit      | `SetMemory(addr, byte)`  |
| Port write (devs)| `WritePort(port, byte)`  |

Hex-edit fields go through validation in the UI; invalid values never reach
the core. `MemCommit` only fires `SetMemory` when both fields parse.

## Subscription

iced subscribes to a 33 ms `time::every` tick. On each tick we drain
the event channel (non-blocking) and emit:

1. `Message::StateArrived(s)` if any state snapshot was queued (priority);
2. else `Message::EventArrived(evt)` for the latest non-state event;
3. else `Message::Tick`.

This keeps the UI thread from blocking and ensures it always sees the most
recent authoritative state.

## File dialogs

`rfd::AsyncFileDialog` is used for `.580` save / load. The dialog runs on
the executor returned by iced; it never blocks the UI thread.
