# Architecture

This workspace implements a layered KR580/Intel 8080 desktop emulator using only the `prompt/` documents as product source.

## Crates

- `k580-core`: public deterministic CPU state, memory, flags, opcode decode/execute, timing, interrupts, and the `PortBus` trait. Applications own their command/event contracts and compose them from these processor primitives. Opcode execution is split by instruction family under `ops/`.
- `kr580`: public installable desktop package. It contains the iced multi-window daemon, launcher, installer, uninstaller, platform shims, and internal `backend`, `devices`, and `persistence` modules. The internal modules own the emulator actor, `IoBus`, monitor, floppy, HDD, network, printer, snapshots, settings, and direct `.txt`/`.xlsx` import/export paths.

## Repository layout

- `crates/core/`: public `k580-core` library crate.
- `crates/ui/`: public `kr580` package with a private binary-side app and view, plus public library modules for backend, devices, persistence, launcher, and installer integration. The library modules are hidden from generated rustdoc where appropriate. Its `desktop_entry` helper centralizes freedesktop command quoting and XDG data-home resolution, `install_mode` resolves adjacent versus manifest-owned binaries, and `shell_quote` supplies the shared Unix single-argument encoder.
- `crates/ui/src/file_assoc/linux_default.rs`: owns the saved Linux default-handler state, `mimeapps.list` cleanup, and restoration of the handler that preceded KR580. `linux_files.rs` supplies process locking, atomic writes, and file backups used for rollback.
- `crates/ui/assets/linux/`: canonical freedesktop launcher, file-handler, package, and shared-MIME templates consumed by runtime registration and native packages.
- `crates/ui/assets/macos/Info.plist`: canonical macOS application metadata used by runtime integration and release packaging.
- `macos_launch_services`: narrow Core Services bridge for registering the installed application bundle and managing its document handlers.
- `platform/macos_open_documents`: Foundation event bridge that forwards Finder document activation into the normal application file-loading path.
- `prompt/`: the implementation source of truth.
- `docs/`: reference documentation (this directory).
- `crates/ui/assets/icons/`: canonical pre-rendered icon set consumed at build, run, and package time. The master `icon.png` lives next to the generated PNG fan-out and the multi-resolution `icon.ico`. See `docs/assets.md`.
- `scripts/`: developer helpers. `generate_icons.ps1` (Windows) and `generate_icons.sh` (Unix/macOS) regenerate `crates/ui/assets/icons/` from the master image. `build_installer.ps1` and `build_installer.sh` build standalone setup artifacts, while `build_macos_dmg.sh` packages the native GUI as a drag-and-drop application image under `dist/`. The verification scripts exercise Linux metadata, macOS images, and release artifact names; `version_release_artifacts.sh` adds the release tag only to unversioned setup packages.
- `target/`: cargo build artefacts (gitignored).

## Installation Layout

`kr580` builds `kr580`, `kr`, `k580-installer`, and `k580-uninstaller`. The
setup builder first builds `kr580` and `kr`, then builds `k580-uninstaller` with
the uninstall icon, then rebuilds `k580-installer` with the setup icon and
those binaries embedded so a new user can run the setup before any KR580 files
exist on the machine. The installer writes `install.json` at the install root,
keeps `kr580` under `app/`, keeps the installed maintenance binary as
`app/uninstaller`, keeps `kr` under `bin/`, and only adds `bin/` to PATH when
requested.
macOS releases instead contain the GUI executable directly in
`KR580.app/Contents/MacOS/kr580`; the surrounding DMG supplies an Applications
link and relies on normal drag-and-drop installation rather than the setup
state machine.
Portable installs default to the user's `KR580` folder and store settings under
`<install root>/data`; both install modes can optionally associate `.580`
snapshots and `.krs` subprograms with `app/kr580`. System installs use the platform config directory and
add OS integration: Start Menu/search launchers, optional desktop launchers,
and uninstall cleanup where the platform supports them. See
`docs/installer.md`.
Executables inside a macOS application bundle use Application Support even
without an installer manifest. Strict Snap executions use the writable,
revision-independent `SNAP_USER_COMMON` directory.
Nix wrappers mark the packaged GUI so it also uses the writable XDG config
directory and expose the wrapped GUI path to file-association code.
The Snap exposes `kr580` itself as its only app command; snapd owns its lifecycle
and exported desktop entry, so no nested installer, uninstaller, PATH writer,
or private desktop database is involved.

The graphical uninstaller drives cleanup as three explicit tasks instead of a
timer-simulated monolith: system integration removal returns an uninstall plan,
link cleanup consumes that plan, and the file stage automatically starts a
platform helper that waits for the uninstaller PID to exit before deleting the
install root. `uninstaller.rs` owns this state machine; `uninstaller_view.rs`
renders it. Real task completion advances target milestones, while a separate
presentation-only timer moves the displayed progress toward those targets. The
final Close action does not initiate cleanup and becomes available only after
the successful animation reaches 100%.

## Data flow

UI messages become `AppCommand` values. The internal backend actor owns `Cpu8080State` and `IoBus`, applies commands, and publishes typed `AppEvent` values. The UI stores only display/input state and can always re-render from `AppSnapshot`.

After `.krs` loading, the actor applies the bytes, publishes `StateChanged`,
and completes the matching request with the installed end address. The UI uses
that completion range for subsequent subprogram saves.

## Invariants

- `prompt/` is the source of truth for behavior, file formats, and quality gates.
- CPU state is owned by `k580-core` and the internal backend actor, never by UI widgets.
- Device state is owned by the internal `devices` module; `IN`/`OUT` route through `PortBus`.
- The internal `persistence` module reads from `Cpu8080State` or explicit export view models, never from UI labels or grids.
- `.krs` remains a raw byte slice with caller-provided base address; no secondary subprogram format is introduced.
- `persistence/subprogram/atomic_save.rs` stages `.krs` writes beside the destination and replaces it only after the complete file has been flushed and closed.

## Runtime shape

Persistence dispatch and execution live in `backend/emulator/io/{dispatch,jobs}.rs`;
`io/mod.rs` owns completion reconciliation. Every job carries the document
generation. New CPU documents and subsequent program/subprogram loads supersede
older completions, which finish as `CommandResult::Superseded` without changing
paths, notices or CPU state. Imports return validated patches and `.krs` loads
return their actual byte blocks; the actor applies them to its current CPU so
unrelated edits made while reading remain intact. Save/export workers retain an
independent captured state or export model.

File-association changes use an iced task backed by Tokio's blocking pool.
`DesktopApp.file_association_pending` survives closing Settings and prevents
overlapping operations; completion messages bypass modal routing. Only Windows
polls handler registration, while Settings is open and no operation is pending.
The dialog keeps that Windows status for both rendering and keyboard actions;
the pending flag has a single owner in `DesktopApp`. Unix has no unused
registration-status query or Launch Services default-handler lookup.

`kr580` sends commands through a crossbeam channel to its internal backend emulator actor. The actor owns CPU and device state and emits typed events. Persistence requests capture actor state and run file work on a dedicated worker, then return a matching `RequestId` completion before the UI changes paths or dirty state. The event path coalesces full snapshots into a latest-state mailbox while keeping completion and error events separate. `Emulator` owns a Tokio runtime for storage, network, and native printer workers, so file, TCP, GDI printing, and printer-driver calls stay outside the UI thread. The printer settings layer keeps the Windows PrintTicket provider lifecycle inside one blocking MTA task, while the iced state stores only parsed capabilities and the validated `DEVMODEW`. The actor polls network, storage, and printer completion every 50 ms and publishes a snapshot only when device state differs from the last published one. `AppCommand::ConfigureNetwork` cancels the previous TCP worker before starting the selected client connection or server listener; `AppCommand::ClearNetworkBuffers` clears only the visible RX buffer and last transmitted value while preserving the active endpoint, connection state, status, and error. When both are already empty, the command is a no-op and publishes no state event.

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
    `StateChanged`; the UI consumes the latest available snapshot.
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
