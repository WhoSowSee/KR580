# App and UI

`k580-app` owns the emulator and exposes PascalCase commands such as `ResetCpu`, `StepInstruction`, `SetRegister`, `SetMemory`, `LoadSnapshot`, and `SaveSnapshot`.

`k580-ui` is an iced application shell. It renders an `AppSnapshot`, sends `AppCommand` values to the actor, and drains `AppEvent` notifications. Register and memory edits are parsed and validated before commands are sent.

Native file dialogs use `rfd`. The UI does not serialize files, run CPU instructions directly, or store emulator state in widgets.
