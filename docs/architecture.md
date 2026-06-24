# Architecture

This workspace implements a layered KR580/Intel 8080 desktop emulator using only the `prompt/` documents as product source.

## Crates

- `k580-core`: public deterministic CPU state, memory, flags, opcode decode/execute, timing, interrupts, typed command/event contracts, and the `PortBus` trait. Opcode execution is split by instruction family under `ops/`.
- `kr580`: public installable desktop package. It contains the iced multi-window daemon, launcher, installer, uninstaller, platform shims, and internal `backend`, `devices`, and `persistence` modules. The internal modules own the emulator actor, `IoBus`, monitor, floppy, HDD, network, printer, snapshots, settings, and direct `.txt`/`.xlsx` import/export paths.

## Repository layout

- `crates/core/`: public `k580-core` library crate.
- `crates/ui/`: public `kr580` package with private app, device, persistence, UI, launcher, and installer modules.
- `prompt/`: the implementation source of truth.
- `docs/`: reference documentation (this directory).
- `assets/icons/`: pre-rendered icon set consumed at build and run time. The master `icon.png` lives next to the generated PNG fan-out and the multi-resolution `icon.ico`. See `docs/assets.md`.
- `assets/fonts/`: bundled fonts and their licenses. Roboto Mono is embedded into printer PDFs; the visible UI keeps the platform UI family and generic monospace selector, with `view::font_warmup` priming both slow Windows font paths during cloaked startup frames.
- `scripts/`: developer helpers. `generate_icons.ps1` (Windows) and `generate_icons.sh` (Unix/macOS) regenerate `assets/icons/` from the master image. `build_installer.ps1` and `build_installer.sh` build standalone setup artifacts under `dist/`.
- `target/`: cargo build artefacts (gitignored).

## Installation Layout

`kr580` builds `k580`, `kr`, `k580-installer`, and `k580-uninstaller`. The
setup builder first builds `k580` and `kr`, then builds `k580-uninstaller` with
the uninstall icon, then rebuilds `k580-installer` with the setup icon and
those binaries embedded so a new user can run the setup before any KR580 files
exist on the machine. The installer writes `install.json` at the install root,
keeps `k580` under `app/`, keeps the installed maintenance binary as
`app/uninstaller`, keeps `kr` under `bin/`, and only adds `bin/` to PATH when
requested.
Portable installs default to the user's `KR580` folder and store settings under
`<install root>/data`; both install modes can optionally associate `.580`
files with `app/k580`. System installs use the platform config directory and
add OS integration: Start Menu/search launchers, optional desktop launchers,
and uninstall cleanup where the platform supports them. See
`docs/installer.md`.

## Data flow

UI messages become `AppCommand` values. The internal backend actor owns `Cpu8080State` and `IoBus`, applies commands, and publishes typed `AppEvent` values. The UI stores only display/input state and can always re-render from `AppSnapshot`.

## Invariants

- `prompt/` is the source of truth for behavior, file formats, and quality gates.
- CPU state is owned by `k580-core` and the internal backend actor, never by UI widgets.
- Device state is owned by the internal `devices` module; `IN`/`OUT` route through `PortBus`.
- The internal `persistence` module reads from `Cpu8080State` or explicit export view models, never from UI labels or grids.
- `.krs` remains a raw byte slice with caller-provided base address; no secondary subprogram format is introduced.

## Runtime shape

`kr580` sends commands through a crossbeam channel to its internal backend emulator actor. The actor applies commands synchronously against the core and bus, then emits state snapshots and typed events. `Emulator` owns a Tokio runtime for storage, network, and printer PDF workers, so file, TCP, and PDF operations stay outside the UI thread. The actor polls network state and printer export completion every 50 ms and publishes a snapshot only when either state differs from the last published one, allowing received bytes, connection changes, and completed PDF jobs to reach an idle UI without causing continuous redraws. `AppCommand::ConfigureNetwork` cancels the previous TCP worker before starting the selected client connection or server listener; `AppCommand::ClearNetworkBuffers` clears only the visible RX buffer and last transmitted value while preserving the active endpoint, connection state, status, and error. When both are already empty, the command is a no-op and publishes no state event.

## Actor pacing loop

The worker thread in `backend::actor::run_worker` does not block on
`recv()`. Instead it uses `crossbeam_channel::select!` to wait
simultaneously on the command channel and a timer:

- **Paused (`!emulator.is_running()`)** – the timer arm is wired to
  `crossbeam_channel::never()`, so the `select!` degenerates to a plain
  command-channel `recv()`. The worker is fully idle until the UI sends
  the next command.
- **Running (`emulator.is_running()`)** – the timer arm is wired to
  `crossbeam_channel::after(deadline)`, where `deadline` depends on
  the active `RunMode`:
  - `RunMode::Paced` (Slow / Medium / High speed tiers in the UI) –
    the deadline is `emulator.step_interval()`. Each timer fire calls
    `emulator.tick()`, which advances exactly one instruction and
    emits `InstructionBoundaryReached`, `HaltStateChanged` (on halt),
    `Stopped` (on budget exhaustion or error), and a fresh
    `StateChanged`. The UI sees every step.
  - `RunMode::Burst { slice }` (Max speed tier in the UI) – the
    deadline is `slice` (16 ms by default). Each timer fire calls
    `emulator.tick()`, which now runs an inner loop that keeps
    stepping the CPU until `slice` wall-time elapses, the per-session
    budget is exhausted, the CPU halts, or an instruction errors.
    Only the **final** snapshot is published; per-instruction
    boundary events are deliberately suppressed so the UI side
    stops paying the per-step redraw cost. The slice doubles as
    the responsiveness floor for `Stop`: a press lands within at
    most one slice because the actor still re-checks the command
    channel between bursts.
  The run timer is scheduled as an absolute `Instant` and is not rebuilt
  from scratch when the 50 ms device poll arm fires. That keeps Slow
  (200 ms) and Medium (50 ms) execution from being starved or jittered by
  printer/network polling. Whichever `select!` arm fires first wins: a
  command interrupts the wait immediately and is applied without skipping
  a beat.

`AppCommand::Run` only flips `Emulator::running = true` and resets the
per-arming `instructions_since_run` counter. `AppCommand::Stop` clears
the flag. `AppCommand::SetStepInterval(duration)` overwrites
`Emulator::step_interval` (clamped to `MIN_STEP_INTERVAL = 1ms`);
`AppCommand::SetRunMode(mode)` overwrites `Emulator::run_mode`
(`slice` is clamped to a 1 ms floor too). The next `select!` iteration
re-arms the timer at the new pace and with the new dispatch shape.
Defaults: `DEFAULT_STEP_INTERVAL = 100ms` (10 instructions/sec),
`RunMode::Paced`, and `MAX_INSTRUCTIONS_PER_RUN = 100_000` instructions
per arming before the worker auto-pauses with `Stopped`.

This decoupling is what makes the UI animation visible at the paced
tiers: the previous `Run` implementation called
`cpu.run_until_halt(&mut bus, 100_000)` synchronously inside the
worker, which produced exactly one `StateChanged` after the whole
burst – the user only ever saw the final state. With the selector
loop and `RunMode::Paced` the UI receives one snapshot per
instruction, so the selected PC fields, registers, and status step
through the program live. `RunMode::Burst` is the explicit opt-out: the user
asks for "доведи программу до конца", and the worker collapses
thousands of instructions into a single snapshot per slice – *fewer*
snapshots than Paced, but each one is *farther apart* in program
state, which is what makes Burst measurably faster than the highest
paced tier even though both use a 1 ms-class deadline.
