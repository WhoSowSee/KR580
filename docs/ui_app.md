# App and UI

`k580-app` owns the emulator and exposes PascalCase commands such as
`ResetCpu`, `StepTact`, `RunForTStates`, `StepInstruction`, `Run`, `Stop`,
`SetStepInterval`, `SetRunMode`, `SetRegister`, `SetMemory`, `ReadPort`,
`WritePort`, `LoadSnapshot`, `SaveSnapshot`, `LoadSubprogram`, and direct
export commands.

`k580-ui` is an iced application shell. It renders an `AppSnapshot`, sends
`AppCommand` values to the actor, and drains `AppEvent` notifications.
Register and memory edits are parsed and validated before commands are
sent.

Native file dialogs use `rfd`. The UI exposes `.580` open/save and
`.txt`/`.xlsx` export and import actions. It does not serialize files,
run CPU instructions directly, or store emulator state in widgets.

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
`pause.svg` in red. **Pause is unconditional**: a click while the run
is armed always sends `AppCommand::Stop`, regardless of where PC has
walked to. This matters because a paced run carries PC through
whatever bytes follow the user's program — once it walks off the
loaded code into a stretch of `0x00` (the default RAM fill), any
gate that compares the *current* `cpu.memory.read(pc)` against zero
would mistake the running program for an empty page and silently
swallow the click. The user reported this as «не могу остановить
программу, только сбросом регистров»: typed `13` at 0x0000, ran it,
PC walked through INX D + NOPs, and pause did nothing because the
byte at the new PC was zero. The current handler returns Stop first
and only then reaches the run-arming gates, so Stop is reachable
from any execution state.

Run-arming is the gated half. With the toggle disarmed, the handler
checks both the halted bit and the byte at `cpu.pc`: a halted CPU
surfaces the reset-registers notice (Variant A — see below), and a
blank page (byte is `0x00`) yields a `No program at <PC>` hint with
no worker activity. Tying the visual flag to the same condition that
gates the dispatch prevents the desync the user reported earlier as
«программа выглядит будто работает, но ничего не выполняется»:
previously, an unconditional icon flip on an empty page survived a
subsequent `Import` / `OpenSnapshot`, leaving the panel painted red
while the worker sat idle. Two extra safeguards back this up —
`load_snapshot_from_path` and `import_file` both clear `self.running`
before they touch the worker, so any prior cosmetic state from
before the document changed is dropped. This also avoids the older
bug where every click on an empty RAM page burned ~100k T-states
inside `cpu.run_until_halt(&mut bus, 100_000)`.

`AppCommand::Run` no longer blocks the worker. It only flips an
`is_running` flag on the emulator; the actual instruction-by-instruction
advance is driven by the worker's `select!` loop (see
`docs/architecture.md`), which fires one `tick()` per
`step_interval`. Each `tick()` executes a single instruction, publishes
`InstructionBoundaryReached` + `StateChanged`, and decrements an internal
`MAX_INSTRUCTIONS_PER_RUN` budget so a runaway program eventually pauses
itself instead of burning the worker thread. `AppCommand::Stop` clears
the flag and emits `Stopped`. The combined effect: the highlighted memory
cell, register readouts, PC, and cycle counter all animate live as the
program executes — no more "click play, see only the final state".

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

## «Содержимое ячеек ОЗУ» follows PC during Run

While the run is armed, the highlight in the memory list (and the
address spinner / inline value buffer) tracks `cpu.pc` automatically.
Implementation:

- The actor publishes one `StateChanged` per executed instruction.
- The 100 ms `Message::Tick` subscription folds those snapshots into
  `DesktopApp` via `pull_events` → `apply_snapshot`.
- After draining the events, `Tick` calls `follow_pc_during_run` when
  `self.running` is true. The helper compares the spinner's current
  address with `cpu.pc`; if they differ it rewrites the spinner,
  refreshes `memory_value_input`, scrolls the viewport to keep the new
  row in view, and returns the same `scroll_memory_to` task the
  step-instruction button uses.

Two guards protect interactive editing:

- `follow_pc_during_run` does **not** call `sync_pc_to_cursor`. PC is
  the source of truth during a paced run; pushing `SetPc(pc)` back at
  the worker would be a useless round-trip and could race the next
  instruction step.
- The inline value buffer (`memory_inline_value_input`) is left alone
  whenever `focused_input == Some(MEMORY_INLINE_INPUT_ID)`, so a user
  who is mid-edit on a faraway cell does not see their typing wiped
  out. Only the spinner moves; the inline editor catches up the moment
  focus leaves it.

When the worker auto-pauses — `HaltStateChanged`, `ErrorRaised`, or
`Stopped` — the `consume_event` handler clears `self.running`, so the
play/pause icon swaps back to green and the Tick branch stops chasing a
frozen counter.

### Final-tick follow-PC at high speed

At slow paces (e.g. the default 10 Hz) the worker delivers one
`StateChanged` per Tick, so the highlight inevitably catches up. At
high speed (e.g. 1000 Hz) it can deliver a long burst of `StateChanged`
followed by a terminal `HaltStateChanged` / `Stopped` inside the same
100 ms tick. By the time the Tick branch re-reads `self.running` the
flag is already `false`, so the closing `follow_pc_during_run` would
not fire and the highlight would be left on whichever row the last
per-instruction snapshot landed on — visibly mid-program even though
the CPU has actually halted.

`DesktopApp::pending_follow_pc` resolves this:

- `consume_event` sets `pending_follow_pc = true` in every auto-pause
  branch right after clearing `self.running`.
- The Tick handler treats `running || pending_follow_pc` as the
  condition for `follow_pc_during_run`, then consumes the flag
  (`pending_follow_pc = false`) so the helper runs exactly once after
  the run ends.
- When the auto-pause was a halt, `follow_pc_during_run` aims at
  `pc.wrapping_sub(1)` — PC sits one byte past the HLT opcode after
  the halt, but the user expects the highlight on the HLT row itself,
  which is what then gets the red row chrome.

### Halted-row highlight

When `cpu.halted` is true and `pc - 1` points at a `0x76` (HLT) byte,
that row in the memory list paints in red instead of the usual blue
selection: `view::styles::containers::memory_row_container_style`
takes a second `halted` argument and returns a red-tinted background
(`TOKYO_RED` at 0.22 alpha) with the same 6 px corner radius as the
regular selection — no extra border, so the highlight reads as a
peer of the blue selection rather than as competing chrome on top of
it. The address column on the same row also switches to `TOKYO_RED`
so the row reads as a single coherent "the program ended here"
banner. The byte check defends against corner cases where PC sits one
past an unrelated byte after a SetPc on a halted state — the halt
visual follows the actual opcode, not the counter alone.

### Halt-blocked controls (Variant A)

After HLT the action panel's run/pause toggle, `Выполнить команду`
(`step_instruction_and_advance`), and `Выполнить такт`
(`step_tact_and_maybe_advance`) all early-return without doing any CPU
work. Each refusal calls `DesktopApp::raise_halt_notice`, which fills
`halt_notice` with the canonical two-line body
(`Процессор остановлен командой HLT\nСбросьте регистры или флаг HLT`),
arms `halt_notice_dismiss_at = Instant::now() + 8s`, **and** sets
`run_blocked_after_halt = true` — all three in lockstep, from the
single chokepoint, so future halt-block sites cannot forget to arm
the latch. The view paints the string as a framed top-center floating
notice (`view::halt_notice_overlay`, `error_inset_style` body — the
same red-bordered chrome as the file-error overlay, so the user reads
"this is a blocking notice" from the frame alone). The text is
centred horizontally so the second (shorter) recommendation line
sits under the first instead of leaning against the left padding.
The body is wrapped in a `mouse_area` whose `on_press` emits
`Message::DismissHaltNotice`, the global Esc handler clears the
notice between `error_notice` and `pending_action`, and
`Message::Tick` polls `halt_notice_dismiss_at` to auto-dismiss
after 8 seconds — the same fade behaviour the user asked for the
file-error overlay to have, applied to its visual twin.

The notice sits above the menu bar's dropdown band — same `stack!`
pattern as the file menu — so it reads as a discrete UI element
instead of a status-bar line that blends into the dark schematic.
The first line is "what's wrong" (the CPU stopped on a HLT) and the
second line is "what unblocks it" (`Сброс регистров или флаг HLT`)
— diagnosis and recommendation on separate lines so the user can
read either half on its own. The recommendation lists both unblock
paths because the halt bit can be cleared either by `Сброс регистров`
(`AppCommand::ResetCpu`, which also rewinds PC to `0x0000`), by the
dedicated `Сбросить флаг HLT` entry at the bottom of the МП-Система
menu / `Ctrl+Shift+H` (`AppCommand::ClearHalt`, which flips *only*
the halt flip-flop and leaves PC, SP, registers, flags, RAM, and
`cycle_count` exactly where HLT left them), or by toggling the HLT
flag from the register editor. RAM is preserved by all three, so
the loaded program survives the unblock and runs again from
whatever PC it ends up at.

`run_blocked_after_halt` is the second half of the contract. The
first time the user attempts a run/step gesture against a halted
CPU they get the explanatory overlay; from that moment on, every
execution chip in the action panel (`Выполнить программу /
Пауза`, `Выполнить команду / Перезапустить программу`, `Выполнить
такт`) renders disabled — `editors::actions_panel` calls
`icon_action_button` with `None` for those four messages, and
iced 0.14 paints the button without hover and ignores clicks when
no `on_press` is attached. The two reset chips (`Сброс ОЗУ`,
`Сброс регистров`) keep their `Some(...)` because the resets are
the way *out* of the latch. The latch outlives the 8-second halt
notice on purpose: the user's contract was "до тех пор пока не
сброшу флаг или регистры" — a fade is not an unblock. Repeat
attempts (e.g. through the menu) re-raise the notice via the same
chokepoint so the explanation comes back even if the original
overlay already faded.

The latch is cleared along three independent edges, mirroring the
notice itself: `apply_snapshot` clears both `halt_notice` and
`run_blocked_after_halt` whenever the new snapshot has
`cpu.halted == false`, so any gesture that flips the halt bit off
(snapshot load, register-editor HLT toggle, etc.) re-enables the
chips automatically; the explicit `Message::ResetCpu` arm clears
the latch before dispatching `AppCommand::ResetCpu`; and the new
`Message::ClearHalt` arm clears the latch before dispatching
`AppCommand::ClearHalt`. The first two paths land through the
worker as a non-halted snapshot anyway — clearing the latch
synchronously in the message arm just makes the next view tick
paint live chips before the round-trip completes, instead of one
frame of stale-disabled chrome.

The notice itself is cleared along the same edges plus the user's
Esc / click and the 8-second deadline; see `clear_halt_notice` in
`app/mod.rs`. The two pieces of state are *armed* together in
`raise_halt_notice` and *cleared* together everywhere except the
fade timer (which clears the notice but leaves the latch armed —
that is the whole point of the latch).

`runtime::memory::sync_pc_to_cursor` also early-returns on halt. The
function normally mirrors a freshly-clicked memory cell into `cpu.pc`
so a subsequent step runs against that byte, but on a halted CPU
there is no next step (the gestures are blocked), and the
`SetPc` round-trip is actively harmful: `dispatch_sync` waits for a
`StateChanged`, which on a halted CPU is the same post-halt snapshot
the worker keeps republishing (`pc = halt_pc + 1`); `apply_snapshot`
then reads that PC back into the spinner and the visible address
jumps one cell forward on every click. Skipping the dispatch lets
the user browse memory freely after HLT — the next reset reattaches
PC to whatever address they end up clicking on.

### Esc reverts unsaved inline memory edits

Inside the inline `Значение` editor, Esc discards a byte the user has
typed but not yet committed with Enter, restoring the buffer to the
byte that actually lives in memory at the highlighted address. The
spinner-side `memory_value_input` is restored alongside the inline
buffer so the side panel stays in sync, and focus stays on the inline
editor so the user can keep typing.

The keyboard subscription emits `Message::EscPressed`; the `update`
router checks `self.focused_input` and calls
`cancel_inline_memory_edit` when the inline editor owns focus,
falling back to `hide_opcode_dropdown` (the legacy Esc binding)
otherwise. Routing in `update` rather than the `Fn` event listener
keeps the listener stateless. With the inline buffer already matching
storage the handler is a no-op so a stray Esc does not snap the caret
to the end of the field.

## Speed switch (left schematic panel)

The schematic board on the left edge of the window carries a paced-run
control next to the Cycle/Tick readout. It is a four-position
segmented switch framed by a `schematic_block_style` capsule, with a
live caption `Скорость: N шаг/сек` that mirrors the resolved Hz of the
active tier. It sits in the `low_control` row inside
`crates/ui/src/view/schematic.rs`, between the cycle/tick block and the
placeholder Sync &amp; Control readout.

The switch replaces an earlier freeform slider (`MIN_STEP_HZ=1` …
`MAX_STEP_HZ=1000`). The slider was honest about the underlying
`SetStepInterval` knob but dishonest about the user-visible result:
above the monitor's refresh rate the same row would still appear
highlighted for one frame regardless of how fast the worker stepped,
so dragging the slider past ~60 Hz only made the program *finish*
faster, not *animate* faster. Named tiers communicate the real
trade-off — pacing for reading, pacing for "just run it", and the
explicit "don't bother animating" opt-in — without inviting the user
to chase a sweet spot that doesn't exist.

| Tier | Label | Resolved Hz | Use |
|---|---|---|---|
| `SpeedTier::Slow`   | Медленно | `SLOW_TIER_HZ = 5`        | One step every 200 ms — read every memory row as PC walks across it. |
| `SpeedTier::Medium` | Средне   | `MEDIUM_TIER_HZ = 20`     | Default. Visibly "the program is running" while the eye still keeps up with each PC update. |
| `SpeedTier::High`   | Высоко   | primary monitor's refresh rate, fallback `HIGH_TIER_FALLBACK_HZ = 60`, capped at `HIGH_TIER_CEILING_HZ = 240` | One instruction per painted frame — finishes as fast as the screen can paint without skipping rows. |
| `SpeedTier::Max`    | Максимум | `MAX_TIER_HZ = 1000`      | Switches the worker to **burst mode** (`RunMode::Burst`). The CPU runs instructions in a tight inner loop bounded by a 16 ms wall-time slice and the per-session budget; the UI sees one coalesced snapshot per slice instead of one per instruction. The opt-in for "доведи программу до конца, мне не нужно смотреть на каждый шаг" — and unlike the earlier "fast slider" attempt, this one is *actually* faster than High, because it stops paying the per-instruction timer + crossbeam + redraw round-trip. |

| Property | Value |
|---|---|
| Storage | `DesktopApp::speed_tier: SpeedTier` |
| Default | `DEFAULT_SPEED_TIER = SpeedTier::Medium` |
| Emit | `Message::SpeedTierChanged(SpeedTier)` |
| Resolve | `tier_hz(tier) -> u32` (in `app/mod.rs`) |
| Dispatch | `AppCommand::SetStepInterval(Duration::from_micros(1_000_000 / hz))` followed by `AppCommand::SetRunMode(...)` |

Switching to a tier emits **two** worker commands: the
`SetStepInterval` keeps the per-instruction delay honest for the
paced tiers, and the `SetRunMode` toggles between
`RunMode::Paced` (Slow / Medium / High — one instruction per tick,
one snapshot per instruction) and `RunMode::Burst { slice: 16 ms }`
(Max — tight inner loop bounded by wall-time, one coalesced
snapshot per slice). The two-command shape lets the actor pick its
deadline based on `run_mode()` without inferring it from the
interval value, which keeps the worker honest if the UI ever wants
to mix burst with a non-default slice.

`tier_hz` encapsulates the platform query for the High tier:
`platform::primary_monitor_refresh_hz()` reads `dmDisplayFrequency`
from `EnumDisplaySettingsW` on Windows; on non-Windows it is a stub
returning `None`. Either way callers fall through to
`HIGH_TIER_FALLBACK_HZ = 60`, then clamp into
`[HIGH_TIER_FALLBACK_HZ, HIGH_TIER_CEILING_HZ]` so a virtual / headless
display reporting `0` or `1` Hz cannot make the worker run at one
instruction per second, and a stuck driver claiming, say, 999 Hz
cannot make it burn cycles on UI ticks the panel can't actually
display. The Max tier sidesteps this resolver entirely and returns
`MAX_TIER_HZ` directly — its semantics are "saturate the worker, ignore
the monitor".

Each tier button is a `button` styled with `mux_button_style`, the
same selected/unselected paint the multiplexer panel uses for its
register cells. The active tier reads as the magenta-tinted square in
the row of four; the others stay on the neutral blue accent. Pressing
a button emits `Message::SpeedTierChanged(tier)`; the handler in
`app/mod.rs` stashes the tier on `DesktopApp::speed_tier`, resolves
it through `tier_hz`, and ships `SetStepInterval` + `SetRunMode` to
the worker. The worker clamps once more at `MIN_STEP_INTERVAL = 1ms`
(re-exported from `k580_app::actor`) before it overwrites
`Emulator::step_interval`. The next `select!` iteration re-arms the
timer with the new interval (paced) or slice (burst), so clicking a
tier while a program is running adjusts the visible animation rate
immediately — without restarting the program or losing the run-armed
state.

The UI subscription tick interval is also coupled to the resolved Hz
(see `subscription` in `app/mod.rs`) with a 16 ms floor. Slow / Medium
/ High deliver one snapshot per tick because the worker is paced and
matches the subscription cadence one-to-one. Max keeps the same 16 ms
subscription, but the worker is no longer paced — each Tick drains a
single coalesced snapshot that already represents *thousands* of
executed instructions. That visible jumpiness across memory rows is
the honest signal that "Максимум" gave up frame fidelity to gain wall
time. A `Stop` press still lands within one slice (≤ 16 ms) because
the actor's `select!` re-arms its deadline to `slice` rather than
parking inside the inner loop.

### Run modes (`RunMode`)

`k580_app::RunMode` controls how the worker dispatches CPU work
during a paced `Run`. Two variants:

- `RunMode::Paced` — the default, used by the Slow / Medium / High
  speed tiers. `Emulator::tick()` executes exactly one instruction,
  publishes one `InstructionBoundaryReached` and one `StateChanged`,
  and the actor re-arms its `after(step_interval)` deadline. The UI
  sees every step. This is the path that lets the highlighted memory
  row walk one cell at a time.
- `RunMode::Burst { slice }` — used by the Max speed tier. `tick()`
  enters a tight inner loop that keeps stepping until any of:
  - `slice` wall-time has elapsed (re-checked every 64 instructions
    so `Instant::now()` doesn't dominate the hot loop),
  - the per-session `MAX_INSTRUCTIONS_PER_RUN` budget is exhausted,
  - the CPU halts,
  - or an instruction errors.
  Only the **final** snapshot is published; the per-instruction
  `InstructionBoundaryReached` events are deliberately suppressed so
  the iced side stops paying the per-step redraw cost. The actor
  re-arms its `after(slice)` deadline, which doubles as the
  responsiveness floor for `Stop`: a press lands within one slice.

`AppCommand::SetRunMode(mode)` switches between them. The emulator
stores the mode in `Emulator::run_mode` and exposes it through
`run_mode()` so the actor can pick the deadline (`step_interval` for
paced, `slice` for burst) on the next `select!` iteration. The
worker also floors `slice` at 1 ms to mirror the `SetStepInterval`
floor — out-of-range zero would degenerate the timer arm into a busy
loop.

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
| Esc | Discard the unsaved byte typed into the inline editor and restore it to the value currently in memory. With no pending edit, falls through to closing the opcode dropdown. |
| ArrowUp / ArrowDown (inline editor focused) | Bump the byte in the inline editor by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to memory until Enter. |
| ArrowUp / ArrowDown (no editor focused) | Move the highlighted address by one. |
| PageUp / PageDown | Move the highlighted address by 16. |

### Global

| Shortcut | Effect |
|---|---|
| Esc | Routed by `Message::EscPressed`: reverts an unsaved inline-edit byte when the inline editor owns focus, otherwise hides the opcode dropdown if it is open. |
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

Two gestures clear the caret without going through any of the
acquire-side write paths and so leave `focused_input` stale on their
own: Esc (iced consumes it by clearing `state.is_focused` on the
focused text_input) and a left-click in dead space (every text_input
that does not contain the click runs the same clearing branch in
`text_input::update`). Both paths chain a `find_focused_optional()`
operation onto whatever else they do — Esc fires it from
`Message::EscPressed`, dead-space clicks fire it from the
`FocusReconciled(None)` branch — and the `Message::ResolveFocusedTracker`
handler clears `focused_input` iff iced reports no focusable still owns
the caret. The `_optional` variant lives in `runtime::focus_ops` because
the built-in `iced::advanced::widget::operation::focusable::find_focused`
returns `Outcome::None` when nothing is focused, which would silently
drop the message exactly when we need it most. Wrapping the answer in
`Option<Id>` and returning `Outcome::Some(option)` makes the report
unconditional.

The `FocusReconciled(Some(_))` branch never needs the poll: the
two-pass click reconciler in `runtime::focus_ops` already chained
`unfocus_except(hit)` and updated `focused_input` to the resolved id
on the same frame.

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
- `windows-sys` (Windows only) with `Win32_Foundation`,
  `Win32_Graphics_Dwm`, and `Win32_Graphics_Gdi` features — the first
  two for DWM cloaking and the rounded-corner attribute, the third for
  `EnumDisplaySettingsW` (used by the High speed tier to read the
  primary monitor's refresh rate).
- `winresource` (Windows-only build dependency) for the PE icon resource.
