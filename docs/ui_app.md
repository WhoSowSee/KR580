# App and UI

`k580-app` owns the emulator and exposes PascalCase commands such as
`ResetCpu`, `StepTact`, `RunForTStates`, `StepInstruction`, `SetRegister`,
`SetMemory`, `ReadPort`, `WritePort`, `LoadSnapshot`, `SaveSnapshot`,
`LoadSubprogram`, and direct export commands.

`k580-ui` is an iced application shell. It renders an `AppSnapshot`, sends
`AppCommand` values to the actor, and drains `AppEvent` notifications.
Register and memory edits are parsed and validated before commands are
sent.

Native file dialogs use `rfd`. The UI exposes `.580` open/save and `.txt`,
`.xlsx`, `.docx` export actions. It does not serialize files, run CPU
instructions directly, or store emulator state in widgets.

## UI module split

- `main.rs` initializes tracing, declares the iced subsystem and window
  options, sets the app-level theme/style, and wires the embedded window
  icon. It also pins the Windows subsystem to GUI on release builds (see
  "Console window suppression").
- `app.rs` defines `DesktopApp`, message types, update routing, theme, the
  subscription, and the constants used to identify focusable inputs and
  scrollables.
- `runtime.rs` contains app-facing command dispatch, event draining, file
  dialogs, input parsing, the address-pattern search, and the focus-cycle
  resolver.
- `view.rs` renders the current snapshot, lays out every panel, and owns
  every widget style.
- `platform.rs` is a Windows-only helper used by `app.rs` for DWM cloaking
  during launch (see "Launch flash mitigation"). On non-Windows targets it
  compiles down to a no-op.

## Event handling

The actor publishes `StateChanged`, `InstructionBoundaryReached`,
`TactAdvanced`, `PortRead`, `PortWritten`, `HaltStateChanged`,
`ErrorRaised`, and `Stopped`. Events are notifications only; the latest
`AppSnapshot` remains the authoritative render source.

## Keyboard shortcuts

The UI exposes the following shortcuts. Modifier names follow iced's
`Modifiers::command()` convention: Ctrl on Windows/Linux, ⌘ on macOS.

### Memory cell editor (address + value pair)

| Shortcut | Effect |
|---|---|
| Enter (in address field) | Jump to the typed address; remembers the typed substring as the search pattern. |
| Enter (in value field) | Write the typed byte into the currently selected address. |
| Ctrl+Enter | Find the next address whose 4-digit hex form contains the cached search pattern, advancing past the current cell and wrapping around 64 KiB. The pattern is captured before the first plain Enter so iterating after an initial jump uses the original short hex (`FF`) rather than the matched address (`00FF`). The pattern is reset whenever the user edits the address field by hand. |
| Alt+Enter | Step to the next sequential address (same as ArrowDown). Never writes memory, never touches the search pattern cache. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

### Register editor (name + value pair)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the typed register. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

### Memory list (the inline value cell of the selected row)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the selected address. |
| Tab / Shift+Tab | Move the selection to the next/previous address and refocus the inline editor for the new row. |
| Esc | Hide the opcode dropdown if it is open. |
| ArrowUp / ArrowDown | Move the highlighted address by one. |
| PageUp / PageDown | Move the highlighted address by 16. |

### Global

| Shortcut | Effect |
|---|---|
| Esc | Hide the opcode dropdown if it is open. |

## Focus rings and styling

The two paired panels each form a closed two-element focus ring that Tab
and Shift+Tab simply swap. Focus is moved via
`iced::widget::operation::focus(id)`, and the destination is decided in
`runtime::cycle_focus` based on the id reported by
`iced::advanced::widget::operation::focusable::find_focused()`. The
inline-editor case handles Tab as "advance the selected address and
refocus the same id", which iced re-renders against the new row.

The address spinner and the register spinner are wrapped in `mouse_area`
to surface hover events. Focus state is not exposed by `text_input` in
iced 0.14, so `DesktopApp::focused_input` is a best-effort cosmetic
marker: it is updated whenever the user types into a known input, when
they explicitly Tab to one, or when they click an inline memory row. This
drives the same blue/cyan/border colour scheme that iced applies to the
plain right-hand text input, so both visual styles match.

## Launch flash mitigation (Windows)

On Windows the desktop window manager paints the client area with the
default white system brush between window creation and the first
GPU-presented frame. To suppress that flash:

1. The window starts with `visible: false` (winit/iced).
2. On `Event::Window::Opened` the app cloaks the HWND through DWM
   (`DWMWA_CLOAK = TRUE`) and then asks iced to switch the window to
   windowed mode. The window now exists and is laid out, but the
   compositor does not show it.
3. After the second `RedrawRequested` the app uncloaks the HWND. By that
   point the wgpu swapchain has presented at least one frame of real
   content.

Cross-platform fall-backs:

- The application root style explicitly paints the iced background with
  `#121320` (`TOKYO_BOARD`) so any frame the OS shows before our wgpu
  surface presents is already in-theme.
- On non-Windows targets `platform::cloak_window` is a no-op.

## Console window suppression

Release builds set `windows_subsystem = "windows"` (via a top-of-file
`#![cfg_attr(...)]`) so launching `k580.exe` from Explorer does not spawn
a stray console window. Debug builds keep the default console subsystem
so `tracing` output stays visible during `cargo run`.

## Window icon

The runtime icon is loaded from `assets/icons/icon-64.png` via
`include_bytes!` and `iced::window::icon::from_file_data`. The full icon
fan-out (`16/32/48/64/128/256` PNGs and a multi-resolution `icon.ico`)
lives in `assets/icons/` and is regenerated by the scripts in
`scripts/`. On Windows, `crates/ui/build.rs` additionally embeds
`icon.ico` into the PE resource section so the `.exe`, Explorer, the
taskbar, and the Start menu all show the icon. See `docs/assets.md` for
the asset pipeline.

## Crate dependencies relevant to the UI

`crates/ui/Cargo.toml`:

- `iced` with the `image` feature (icon decoding) and the `advanced`
  feature (`find_focused` and the `operate` task wrapper) enabled.
- `rfd` for native file dialogs.
- `tracing` and `tracing-subscriber` for diagnostic logging.
- `windows-sys` (Windows only) with `Win32_Foundation` and
  `Win32_Graphics_Dwm` features for DWM cloaking.
- `winresource` (Windows-only build dependency) for the PE icon resource.
