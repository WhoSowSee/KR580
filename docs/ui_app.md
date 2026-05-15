# App and UI

`k580-app` owns the emulator and exposes PascalCase commands such as `ResetCpu`, `StepTact`, `RunForTStates`, `StepInstruction`, `SetRegister`, `SetMemory`, `ReadPort`, `WritePort`, `LoadSnapshot`, `SaveSnapshot`, `LoadSubprogram`, and direct export commands.

`k580-ui` is an iced application shell. It renders an `AppSnapshot`, sends `AppCommand` values to the actor, and drains `AppEvent` notifications. Register and memory edits are parsed and validated before commands are sent.

Native file dialogs use `rfd`. The UI exposes `.580` open/save and `.txt`, `.xlsx`, `.docx` export actions. It does not serialize files, run CPU instructions directly, or store emulator state in widgets.

## UI module split

- `main.rs` initializes tracing and starts iced.
- `app.rs` defines `DesktopApp`, message types, update routing, theme, and subscriptions.
- `runtime.rs` contains app-facing command dispatch, event draining, file dialogs, and input parsing.
- `view.rs` renders the current snapshot and input controls.

## Event handling

The actor publishes `StateChanged`, `InstructionBoundaryReached`, `TactAdvanced`, `PortRead`, `PortWritten`, `HaltStateChanged`, `ErrorRaised`, and `Stopped`. Events are notifications only; the latest `AppSnapshot` remains the authoritative render source.
