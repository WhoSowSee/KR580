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
- `app/` defines `DesktopApp`, message routing, theme, and the keyboard
  subscription:
  - `app/mod.rs` — state container, `update`, `subscription`, and
    `handle_arrow_key`, the router that maps a raw ArrowUp/ArrowDown
    press to the editor that currently owns focus.
  - `app/messages.rs` — the `Message` enum.
  - `app/constants.rs` — widget identifiers, register order, and the
    name lookup helpers. Re-exported from `crate::app::*` so the rest
    of the crate keeps importing them by short path.
- `runtime/` contains app-facing command dispatch, event draining, file
  dialogs, and the per-panel update logic. The methods all hang off
  `impl DesktopApp` and are grouped by responsibility:
  - `runtime/mod.rs` — `dispatch`, `pull_events`, `apply_snapshot`,
    file-dialog helpers.
  - `runtime/register.rs` — register name/value editing, including
    `step_register_value_input` for ArrowUp/ArrowDown ±1 stepping.
  - `runtime/memory.rs` — memory list, address spinner, inline editor,
    Ctrl+Enter pattern search, and the matching value-step helpers
    (`step_memory_value_input`, `step_inline_memory_value_input`).
  - `runtime/focus.rs` — Tab/Shift+Tab cycling between fields.
  - `runtime/parse.rs` — small free helpers (hex parsing, normalization,
    `saturating_step_u8`).
- `view.rs` renders the current snapshot, lays out every panel, and owns
  every widget style.
- `platform.rs` is a Windows-only helper used by `app/mod.rs` for DWM
  cloaking during launch (see "Launch flash mitigation"). On non-Windows
  targets it compiles down to a no-op.

## Event handling

The actor publishes `StateChanged`, `InstructionBoundaryReached`,
`TactAdvanced`, `PortRead`, `PortWritten`, `HaltStateChanged`,
`ErrorRaised`, and `Stopped`. Events are notifications only; the latest
`AppSnapshot` remains the authoritative render source.

## Side panel layout

The right-hand 330 px column stacks four legend-framed panels in this
order, top to bottom:

1. **«Список ячеек ОЗУ»** — virtualised memory list with the inline
   value editor and the opcode dropdown.
2. **«Ячейка ОЗУ и ее значение»** — address spinner + value field +
   `↵` apply button.
3. **«Регистр и его значение»** — register name spinner + value field +
   `↵` apply button.
4. **«Управление»** — action button strip described below.

### «Управление» action panel

A single horizontal strip of square 38×38 SVG icon buttons that mirror
the toolbar of the reference KR-580 emulator. The execution group sits
on the left, then a thin vertical divider, then the memory-state
(reset) group on the right (the divider colour matches the surrounding
panel border so it reads as a piece of the frame):

| Group | Icon | Message | Accent | Tooltip |
|---|---|---|---|---|
| run  | `play.svg` / `pause.svg` | `Message::ToggleRun`       | green / red | Выполнить программу / Пауза |
| run  | `step-forward.svg` / `refresh-ccw.svg` | `Message::StepInstruction` / `Message::RestartProgram` | blue | Выполнить команду / Перезапустить программу |
| run  | `redo-dot.svg`        | `Message::StepTact`        | yellow  | Выполнить такт |
| reset | `reset-ram.svg`       | `Message::ResetRam`        | red     | Сброс ОЗУ |
| reset | `reset-registers.svg` | `Message::ResetCpu`        | magenta | Сброс регистров |

The first two buttons are tumblers driven by `DesktopApp::running`.

The leftmost (run/pause) button mirrors the reference KR-580 emulator.
At rest it paints `play.svg` in green; once armed it swaps to
`pause.svg` in red. Toggling the icon is decoupled from dispatching
`AppCommand::Run`: when the byte at `cpu.pc` is `0x00` the press is
purely cosmetic — the icon flips and the status line reads
`No program at <PC>` / `Stopped`, but no T-states are consumed. With a
program loaded at the current PC the toggle drives the real
`Run` / `Stop` commands, so users who actually loaded code keep the
original behaviour. This avoids the prior bug where every click on an
empty RAM page burned ~100k T-states inside
`cpu.run_until_halt(&mut bus, 100_000)`.

The second (step / restart) button is `step-forward` at rest and
`refresh-ccw` while running. At rest it sends
`Message::StepInstruction`, which dispatches a single
`AppCommand::StepInstruction` and then jumps the memory list / address
spinner to the new program counter so the highlighted cell follows the
CPU as the user steps through code. While running it sends
`Message::RestartProgram`, which dispatches `AppCommand::ResetCpu`
followed by `AppCommand::Run`: the registers and flags are wiped, the
program counter goes back to `0x0000`, the run state stays armed, and
the program executes again from the beginning. Memory is preserved
(no `ResetRam`), so the loaded program survives the restart.

The SVG sources live under `assets/icons/actions/` and are embedded at
build time with `include_bytes!`; `crates/ui/src/view/icons.rs` holds
one `LazyLock<svg::Handle>` per icon and exposes a thin getter for each.
The `play` and `step-forward` icons come from the Lucide set; the
`redo-dot` icon is also Lucide and reads as "step one tick around the
loop"; `reset-ram` and `reset-registers` are custom KR-580 glyphs (a
DIMM silhouette and a stacked-register block, each annotated with a
circular reset arrow). All five files declare
`stroke="currentColor"`, so the iced `svg` widget tints them at
runtime via `svg::Style { color: Some(accent) }` — the accent is the
glyph colour at rest and the border colour on hover/press, while the
surface stays on the neutral `TOKYO_BG` / `TOKYO_BORDER` palette of the
editor `↵` button. Tooltip bodies use `inset_style` so they belong to
the same chrome family as the rest of the side panel. The same actions
remain available from the top menu bar — this panel is a discoverable
in-context surface for the same commands; no new `AppCommand` or
`Message` variants were added. iced's `svg` Cargo feature is enabled
in `crates/ui/Cargo.toml` so the renderer pulls in the resvg backend.

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
| ArrowUp / ArrowDown (in address field) | Step the highlighted address by one. |
| ArrowUp / ArrowDown (in value field) | Bump the byte in the value field by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to memory until Enter; ArrowUp on `FF` and ArrowDown on `00` are no-ops. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

### Register editor (name + value pair)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the typed register. |
| ArrowUp / ArrowDown (in name field) | Cycle to the previous/next register in `A B C D E H L`. |
| ArrowUp / ArrowDown (in value field) | Bump the byte in the value field by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to the register until Enter; ArrowUp on `FF` and ArrowDown on `00` are no-ops. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

### Memory list (the inline value cell of the selected row)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the selected address. |
| Tab / Shift+Tab | Move the selection to the next/previous address and refocus the inline editor for the new row. |
| Esc | Hide the opcode dropdown if it is open. |
| ArrowUp / ArrowDown (inline editor focused) | Bump the byte in the inline editor by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to memory until Enter. |
| ArrowUp / ArrowDown (no editor focused) | Move the highlighted address by one. |
| PageUp / PageDown | Move the highlighted address by 16. |

### Global

| Shortcut | Effect |
|---|---|
| Esc | Hide the opcode dropdown if it is open. |
| ArrowUp / ArrowDown | Routed by `DesktopApp::handle_arrow_key` to the editor that currently owns focus (see the panel-specific tables above). With nothing tracked focused they fall back to memory list navigation. |
| PageUp / PageDown | Move the highlighted address by 16, regardless of focus. |

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
