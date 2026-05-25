//! Runtime-side helpers for `DesktopApp`: command dispatch, snapshot
//! reconciliation, file dialogs, and follow-on submodules that group the
//! per-panel update logic.
//!
//! The actual editor methods are split by responsibility:
//!
//! - `register` — register name/value editing.
//! - `memory` — memory list, address spinner, inline editor, search.
//! - `focus` — Tab/Shift+Tab cycling.
//! - `focus_ops` — custom Focusable operation (post-click reconciliation).
//! - `parse` — small free helpers (hex parsing, saturating step).

mod focus;
mod focus_ops;
mod humanize_error;
mod memory;
mod parse;
mod register;
mod undo;

pub(crate) use focus_ops::{find_focusable_at, find_focused_optional, unfocus_except};

use crate::app::DesktopApp;
use k580_app::{AppCommand, AppEvent, AppSnapshot, Snapshot580Flavour};
use std::path::PathBuf;
use std::time::Duration;

use parse::parse_hex_u16;

/// Upper bound on how long a UI handler will wait for the worker thread
/// to publish the post-command snapshot. The emulator is single-stepping
/// or running ≤100k T-states per command, all of which complete in
/// well under a millisecond on contemporary hardware; 50 ms is generous
/// enough to absorb scheduler hiccups without making the UI feel
/// sluggish if the worker has actually died.
const SYNC_DISPATCH_TIMEOUT: Duration = Duration::from_millis(50);

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
        }
        self.pull_events();
    }

    /// Same as `dispatch`, but blocks until the worker thread has
    /// published a `StateChanged` for this command (or 50 ms have
    /// elapsed). Used by handlers that immediately read fields of
    /// `self.snapshot` to decide what to do next — e.g. the step button
    /// follows the new program counter in the memory list. Without this
    /// the snapshot read races the channel and the user has to click
    /// twice for the highlight to move.
    pub(crate) fn dispatch_sync(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
            return;
        }
        for event in self.handle.drain_until_state_change(SYNC_DISPATCH_TIMEOUT) {
            self.consume_event(event);
        }
    }

    /// Flips the visual run/pause state of the action panel's leftmost
    /// button and dispatches the matching CPU command. Pause is
    /// unconditional: with the run already armed, a click always
    /// sends `AppCommand::Stop`, regardless of where PC has walked
    /// to. Run-arming is gated — a halted CPU surfaces the
    /// reset-registers notice instead, and a blank page (the byte at
    /// `cpu.pc` is `0x00`) yields a status-bar hint with no worker
    /// activity. Together that keeps Stop reachable from any execution
    /// state while still preventing the play icon from going armed
    /// when there is nothing to actually execute.
    pub(crate) fn toggle_run(&mut self) {
        // Pause always wins. With the run already armed, a click on
        // the (currently red) pause glyph is unconditional: send
        // `AppCommand::Stop`, drop `self.running`, and let the worker
        // produce its `Stopped` event. The early-return matters
        // because the gates below would otherwise refuse the press —
        // a paced run advances PC into whatever bytes follow the
        // user's program, and once PC walks off the loaded code into
        // a stretch of `0x00` (the default RAM fill), the
        // `has_program` check below would mistake the running
        // program for an empty page and silently swallow the click.
        // The user reported this as «не могу остановить программу,
        // только сбросом регистров»: they typed `13` at 0x0000,
        // ran the program, PC walked through INX D + NOPs, and
        // pressing pause did nothing because the byte at the
        // current PC was zero. Putting the pause shortcut first
        // makes Stop reachable from any execution state.
        if self.running {
            self.running = false;
            self.dispatch(AppCommand::Stop);
            return;
        }

        // Halt-block latch: after the first run/step gesture refused
        // by HLT, every further execution attempt is a no-op until
        // the user explicitly resets registers or clears the halt
        // bit. The action panel buttons are already disabled by the
        // latch, so this branch only fires for the keyboard shortcut
        // (Ctrl+R) and the menu items, which iced does not gate on
        // their own. Re-raising the notice keeps the user from
        // staring at a dead button with no on-screen explanation
        // when the 8-second fade has already eaten the original
        // overlay.
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return;
        }

        // Run-arming path. Two gates protect against actions that
        // would either burn cycles for no observable effect or
        // stall on the very first instruction: a halted CPU has to
        // be reset before it can run again, and a blank page has
        // nothing to execute. Both branches are no-ops on the worker
        // (no `Run` is dispatched), so `self.running` stays false
        // and the play icon stays green.
        //
        // After HLT the CPU is wedged on the byte past HLT and a new
        // `Run` would just hit the same wall on the very first
        // `step_instruction`. Per docs/ui_app.md (Variant A: halt-blocked
        // controls), refuse to arm the run and leave the user a hint
        // about the only gesture that actually unblocks anything —
        // resetting the registers brings PC back to 0x0000.
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return;
        }
        let pc = self.snapshot.cpu.pc;
        let has_program = self.snapshot.cpu.memory.read(pc) != 0;
        if !has_program {
            // Empty memory at PC: refuse to arm the run at all. An
            // earlier iteration flipped the visual `self.running`
            // flag without sending `AppCommand::Run`, on the theory
            // that an empty page deserves a free cosmetic pause
            // icon. That left the door open to a desync: if the user
            // then imported a file, opened a snapshot, or even just
            // typed a byte through the inline editor, the action
            // panel kept flashing the red pause and `Message::Tick`
            // kept chasing PC even though no instruction had ever
            // been executed — the program *looked* like it was
            // running. Suppressing the flip here keeps the visible
            // state honest. Note this gate runs **after** the early
            // pause-return above, so it only applies to the
            // disarmed-to-armed transition.
            self.status = format!("Нет программы по адресу {pc:04X}");
            return;
        }
        self.running = true;
        self.dispatch(AppCommand::Run);
    }

    /// Resets the CPU registers/flags and re-runs the program from
    /// `0x0000`. Bound to the action panel's second button while the run
    /// state is armed: the icon swaps from `step-forward` to
    /// `refresh-ccw`, and pressing it acts as a "restart from the
    /// beginning" gesture mirroring the reference KR-580 emulator.
    /// Memory is preserved (only `ResetCpu` is sent, not `ResetRam`), so
    /// the loaded program survives the restart and starts executing again
    /// at PC = 0.
    pub(crate) fn restart_program(&mut self) {
        // Halt-block latch: same gating story as `toggle_run`. The
        // action-panel button is already rendered disabled by the
        // latch, but the second-button glyph swap (`refresh-ccw` ↔
        // `step-forward`) means a stray keyboard shortcut or menu
        // path could still land here, so we re-raise the notice and
        // bail out instead of sending ResetCpu + Run, which would
        // immediately re-trip the halt and surface the same notice
        // anyway after one wasted round trip through the worker.
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return;
        }
        // ResetCpu is what the user expects Ctrl+Z to roll back here:
        // restart wipes registers and PC, and reverting registers is
        // why the undo stack exists in the first place. The follow-up
        // `Run` is intentionally *not* in the undo entry — running is
        // not a state the timeline cares about, only the CPU snapshot
        // it was running against, and that snapshot is captured by
        // ResetCpu's pair already.
        self.dispatch_with_undo(AppCommand::ResetCpu);
        // Keep the run state armed and dispatch the run command directly,
        // so the visible play/pause toggle stays consistent with what the
        // CPU is actually doing.
        self.running = true;
        self.dispatch(AppCommand::Run);
    }

    /// Same as `dispatch_sync`, but also pushes a `Cpu` undo entry
    /// capturing the CPU state before and after the command. Used by
    /// every mutating dispatch site that the user might want to roll
    /// back with Ctrl+Z (`SetMemory`, `SetRegister`, `ResetCpu`,
    /// `ResetRam`, opcode picker writes). Read the `before` snapshot
    /// *before* sending the command so the rewind target is the state
    /// the user saw, then `dispatch_sync` blocks for the worker to
    /// publish the post-command snapshot, and the resulting
    /// `self.snapshot.cpu` is the `after` half. `push_cpu` itself
    /// drops no-op pairs, so a `SetMemory` that writes the same byte
    /// never clutters the stack.
    ///
    /// The blocking dispatch is non-negotiable here: an earlier
    /// async variant captured `after` before the worker had even
    /// received the command, so every undo entry was pushed with
    /// `before == after` and `push_cpu` silently dropped it as a
    /// no-op. The user reported it as "ввёл 12, нажал Enter,
    /// Ctrl+Z пишет «нечего отменять»" — exactly this race.
    pub(crate) fn dispatch_with_undo(&mut self, command: AppCommand) {
        let before = self.snapshot.cpu.clone();
        self.dispatch_sync(command);
        let after = self.snapshot.cpu.clone();
        // `push_cpu` drops no-op pairs (before == after), and that is
        // also the right gate for the dirty flag: a SetMemory that
        // re-writes the same byte should neither clutter the undo
        // stack nor mark the document as edited. Reading the result
        // of `push_cpu` would be cleaner but it currently returns
        // `()`; recomputing the equality here is cheap (Cpu8080State
        // implements PartialEq via deriving) and keeps the two
        // gates in lock-step without a wider refactor.
        if before != after {
            self.dirty = true;
        }
        self.undo_stack.push_cpu(before, after);
    }

    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            self.consume_event(event);
        }
    }

    fn consume_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
            AppEvent::InstructionBoundaryReached(outcome) => {
                self.status = format!("{} at {:04X}", outcome.mnemonic, outcome.pc_before)
            }
            AppEvent::TactAdvanced(outcome) => {
                if outcome.instruction_boundary {
                    self.last_tact_was_boundary = true;
                }
                self.status = format!("Такт {} цикл {}", outcome.tact_phase, outcome.cycle_count)
            }
            AppEvent::PortRead { port, value } => {
                self.status = format!("IN {port:02X} -> {value:02X}")
            }
            AppEvent::PortWritten { port, value } => {
                self.status = format!("OUT {port:02X} <- {value:02X}")
            }
            AppEvent::SnapshotFlavourLoaded(flavour) => {
                // Park the worker's verdict on which `.580` flavour
                // matched into the scratch slot on `DesktopApp`. The
                // helper that initiated the auto-detect load
                // (`load_snapshot_from_path`) reads it on the way out
                // to route `path` into the matching "current path"
                // field, then resets the slot. We deliberately do not
                // touch the status bar here — the load helper writes
                // a single human-readable line at the end of its run
                // and a flavour update mid-flight would just clobber
                // it.
                self.pending_snapshot_flavour = Some(flavour);
            }
            AppEvent::HaltStateChanged(_) => {
                // The worker has flipped its own `is_running` flag off
                // (the emulator pauses itself on halt). Mirror that on
                // the UI so the play/pause icon swaps back to green play
                // and the `Message::Tick` "follow PC" branch stops
                // chasing a frozen counter.
                self.running = false;
                // At high speed the worker can drain a burst of
                // `StateChanged` *and* the terminal `HaltStateChanged`
                // inside one 100 ms Tick. By the time the Tick branch
                // re-reads `self.running` the flag is already `false`,
                // so the closing `follow_pc_during_run` would not fire
                // and the highlight would stay on whichever row the
                // last per-instruction snapshot landed on. The pending
                // flag forces one more follow-pass on the next Tick so
                // the highlight reaches the HLT line before going idle.
                self.pending_follow_pc = true;
                self.status = "ЦП остановлен".to_owned();
            }
            AppEvent::ErrorRaised(error) => {
                // An error from the core or the bus also auto-pauses the
                // worker; keep UI state aligned for the same reason as
                // the halt branch above.
                self.running = false;
                self.pending_follow_pc = true;
                let raw = error.to_string();
                // Status bar carries the raw, untranslated text — it
                // is the developer-visible channel, the place we
                // attach to bug reports, and rewriting it to
                // Russian would erase information that English
                // logs / search engines / issue trackers expect. The
                // human-readable variant is the *user-facing*
                // overlay copy below.
                self.status = raw.clone();
                // Surface the same condition through the floating
                // overlay using a short Russian sentence. The
                // `humanize_error` module pattern-matches on the
                // English Display text and falls back to a generic
                // line plus the raw text in parens for anything it
                // doesn't recognise — see the module's doc comment
                // for why localization happens at this layer rather
                // than inside `AppError`.
                self.error_notice = Some(format!("Ошибка: {}", humanize_error::humanize(&raw)));
                // Arm the auto-dismiss timer: 8 seconds is the user-
                // requested window before the notice fades on its
                // own. Earlier the only exits were a click on the
                // overlay or Esc; without a timer the notice would
                // sit there forever if the user neither dismissed it
                // nor triggered another file gesture, which felt
                // sticky. `Message::Tick` polls this deadline.
                self.error_notice_dismiss_at =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
            }
            AppEvent::Stopped => {
                // Sent both for an explicit `AppCommand::Stop` and when
                // the worker auto-pauses (`MAX_INSTRUCTIONS_PER_RUN`
                // exhausted or halt/error path). Either way the run is
                // no longer armed; clear the flag so `toggle_run` and
                // the Tick handler agree with the worker.
                self.running = false;
                self.pending_follow_pc = true;
                self.status = "Остановлен".to_owned();
            }
        }
    }

    fn apply_snapshot(&mut self, snapshot: AppSnapshot) {
        let register_value_follows_snapshot =
            crate::app::parse_register_name(&self.register_name_input)
                == Some(self.selected_register)
                && self.register_value_input
                    == format!(
                        "{:02X}",
                        self.snapshot.cpu.registers.get(self.selected_register)
                    );
        let memory_address = parse_hex_u16(&self.memory_address_input).ok();
        let old_memory_value =
            memory_address.map(|address| format!("{:02X}", self.snapshot.cpu.memory.read(address)));
        let memory_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_value_input == *value);
        let inline_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_inline_value_input == *value);

        self.snapshot = snapshot;

        // The halt-blocked notice (top-center floating frame) lives
        // alongside the snapshot's halted bit: the moment the worker
        // publishes a fresh state where `halted == false`, the user
        // has unblocked themselves (typically via Сброс регистров,
        // the only gesture that clears the flag), so the notice
        // should disappear with no further bookkeeping at the
        // dispatch sites.
        if !self.snapshot.cpu.halted {
            self.clear_halt_notice();
            // Lift the UI latch in lockstep with the overlay: the
            // worker has confirmed the halt bit is down, so the
            // execution buttons must come back to life on the same
            // frame the notice fades. ResetCpu / ClearHalt already
            // flip this UI-side, but a snapshot path the user did
            // not initiate (Ctrl+Z that rewinds past the halting
            // instruction, a snapshot load whose cpu was never
            // halted) also reaches this branch and the latch must
            // come down with it — otherwise the buttons would stay
            // disabled even though the halt bit is no longer set.
            self.run_blocked_after_halt = false;
        }

        if register_value_follows_snapshot {
            self.register_value_input = format!(
                "{:02X}",
                self.snapshot.cpu.registers.get(self.selected_register)
            );
        }

        if let Some(address) = memory_address {
            let value = format!("{:02X}", self.snapshot.cpu.memory.read(address));
            if memory_value_follows_snapshot {
                self.memory_value_input = value.clone();
            }
            if inline_value_follows_snapshot {
                self.memory_inline_value_input = value;
            }
        }
    }

    pub(crate) fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
            .pick_file()
        {
            self.load_snapshot_from_path(path);
        }
    }

    /// Load a snapshot from a known path without opening a picker
    /// dialog. Used by both the File→Открыть menu (after the picker
    /// returns a path) and the `Message::LoadSnapshotFromPath` wired
    /// at startup when the OS hands us a `.580` file via `argv[1]`
    /// (double-click in Explorer with `k580.exe` as the default
    /// opener).
    ///
    /// Both `.580` flavours — modern K580 v1 (TLV) and legacy
    /// 65 549-byte flat dump — share the same extension, so this
    /// path cannot trust the file name and instead dispatches
    /// `LoadAnySnapshot`, which probes the bytes on the worker side
    /// and emits `SnapshotFlavourLoaded(flavour)` with its verdict.
    /// We read that verdict back from the scratch slot
    /// (`pending_snapshot_flavour`) and route `path` into the
    /// matching "current path" field so the next Ctrl+S /
    /// Ctrl+Alt+S writes back through the right serializer instead
    /// of silently re-encoding a legacy file as modern (or vice
    /// versa).
    ///
    /// The earlier implementation hard-wired `LoadSnapshot`, which
    /// bound double-click of legacy files to the modern decoder and
    /// rejected them with `InvalidMagic` — exactly the bug the user
    /// reported as "открытие 580 файлов через двойной клик по файлу
    /// старой версии выдает ошибку".
    ///
    /// Opening a file is *not* a Ctrl+Z-able gesture. The user's
    /// mental model is "this is a fresh starting point" — a freshly
    /// opened document has no history above it the same way a
    /// freshly launched editor does. Wrapping the load in
    /// `dispatch_with_undo` would technically work but produced the
    /// surprise the user reported: open file → Run → Ctrl+Z → blank
    /// buffer, with the now-loaded program lost. So we do the same
    /// thing the "Новый файл" handler does — wipe the timeline so
    /// the new document starts clean.
    pub(crate) fn load_snapshot_from_path(&mut self, path: PathBuf) {
        // A new file gesture earns a clean overlay: any error notice
        // from the previous failed gesture must clear so the user
        // can tell *this* attempt apart from the last one. If the
        // dispatch below itself errors, `consume_event` will refill
        // `error_notice` synchronously before we read it back. Goes
        // through `clear_error_notice` so the auto-dismiss deadline
        // is dropped in lockstep — see the helper for rationale.
        self.clear_error_notice();
        // Same reasoning for the info notice: any "Открыт старый
        // формат файла" hint from a previous legacy-open must clear
        // up front so the new attempt starts on a clean slate. The
        // legacy branch below will re-arm it through
        // `raise_info_notice` if (and only if) auto-detect picks
        // the legacy decoder for *this* file.
        self.clear_info_notice();
        let display = path.display().to_string();
        // Disarm the cosmetic run flag *before* the load goes out:
        // `toggle_run` on a blank page is purely visual (no
        // `AppCommand::Run` is dispatched when the byte at PC is
        // zero), so a stale `self.running == true` survives the
        // open and leaves the action panel flashing the red pause
        // icon even though the worker has not executed a single
        // instruction. With the load swapping the document under
        // the user's feet, any prior \"armed\" state is meaningless —
        // start the new file in the same neutral state a fresh
        // launch would.
        self.running = false;
        // Wipe any leftover flavour from a previous load so a
        // failed dispatch (where the worker never emits a fresh
        // `SnapshotFlavourLoaded`) cannot mistakenly claim the new
        // path against a stale flavour value.
        self.pending_snapshot_flavour = None;
        self.dispatch_sync(AppCommand::LoadAnySnapshot(path.clone()));
        // If the worker rejected the load (corrupt header, missing
        // file, IO error, neither flavour matched), `consume_event`
        // already wrote the user-visible notice. Bail out *before*
        // we mutate the document's identity — we don't want to
        // claim "current path = X" for a load that never happened,
        // otherwise the next Ctrl+S would silently overwrite the
        // unrelated file at X with the CPU state from before the
        // failed open.
        if self.error_notice.is_some() {
            return;
        }
        // Route `path` into the field that matches the flavour the
        // worker resolved. Modern → `current_snapshot_path` so plain
        // Ctrl+S overwrites in place; Legacy → `current_legacy_snapshot_path`
        // so Ctrl+Alt+S overwrites in place. The opposite slot is
        // wiped in each branch so a stray shortcut for the *other*
        // format prompts for a fresh location instead of silently
        // clobbering an unrelated file from before this gesture.
        match self.pending_snapshot_flavour.take() {
            Some(Snapshot580Flavour::Modern) => {
                self.current_snapshot_path = Some(path);
                self.current_legacy_snapshot_path = None;
                self.status = format!("Открыто {display}");
            }
            Some(Snapshot580Flavour::Legacy) => {
                self.current_snapshot_path = None;
                self.current_legacy_snapshot_path = Some(path);
                self.status = format!("Открыто {display} (старый формат)");
                // Yellow heads-up overlay: the status bar at 13 px
                // is too quiet a channel for "you opened a legacy
                // file" (the user reported missing the "(старый
                // формат)" status entirely on a small monitor),
                // and the next save routes through a different
                // serializer based on which "current path" slot we
                // populated above — the user deserves to know that
                // before they hit Ctrl+S. 5-second fade per the
                // explicit "5 секунд" ask.
                self.raise_info_notice("Открыт старый формат файла".to_owned());
            }
            None => {
                // The worker accepted the load but failed to publish
                // a flavour — should never happen given the dispatch
                // above, but if it ever does we conservatively treat
                // the file as a v1 origin so plain Ctrl+S still has
                // a meaningful destination. Flagging it through the
                // status bar makes the unexpected-state path visible
                // without forcing the user to abort.
                self.current_snapshot_path = Some(path);
                self.current_legacy_snapshot_path = None;
                self.status = format!("Открыто {display}");
            }
        }
        // Wipe undo/redo *after* the load — the worker has already
        // accepted the new document, and any history pointing at the
        // pre-load buffer would be misleading (Ctrl+Z would tear the
        // freshly opened program back out from under the user). See
        // the rationale in the doc-comment above.
        self.undo_stack.clear();
        // Open is a "fresh starting point" for the dirty flag too —
        // the loaded document by definition matches what is on disk,
        // so the next close/open gesture should run without a
        // confirmation modal until the user actually edits something.
        self.dirty = false;
        let pc = self.snapshot.cpu.pc;
        self.set_memory_address(pc);
    }

    /// Plain "Сохранить": overwrite the currently associated path if
    /// there is one, otherwise fall back to a save dialog (the very
    /// first save of an unnamed snapshot). Auto-flushes the pending
    /// inline-memory edit so a byte the user typed but did not press
    /// Enter on still makes it into the file — without this flush the
    /// saved snapshot was missing exactly the bytes the user thought
    /// they were saving.
    pub(crate) fn save_snapshot(&mut self) {
        self.commit_pending_inline_edit();
        // Same reset rationale as `load_snapshot_from_path`: a fresh
        // gesture earns a clean overlay. If the dispatch below itself
        // errors, `consume_event` will refill `error_notice`
        // synchronously before we read it back.
        self.clear_error_notice();
        let path = match &self.current_snapshot_path {
            Some(path) => path.clone(),
            None => {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("KR580 file", &["580"])
                    .save_file()
                else {
                    return;
                };
                self.current_snapshot_path = Some(path.clone());
                path
            }
        };
        let display = path.display().to_string();
        // dispatch_sync waits for the post-write `StateChanged` so any
        // pending `SetMemory` queued from `commit_pending_inline_edit`
        // is guaranteed to be applied *before* the snapshot is
        // serialized. Without the sync, the worker would see the writes
        // and the save command both arrive in order — but they would
        // execute against whatever state was in flight at command
        // receipt, which on a heavily edited buffer was racing the
        // update.
        self.dispatch_sync(AppCommand::SaveSnapshot(path));
        // If the worker rejected the write (permission denied, IO
        // error, disk full), `consume_event` already wrote the user-
        // visible notice. Bail out *before* we mutate the document's
        // state forward — the on-disk file does not reflect the live
        // state, so we must not clear `dirty` or claim the save
        // landed in the status bar.
        if self.error_notice.is_some() {
            return;
        }
        // Saving as v1 retargets the document's "primary" location to
        // a v1 path. Drop the legacy path so Ctrl+Alt+S after a v1
        // save prompts for a fresh location instead of writing to a
        // legacy file the user may not remember opening earlier.
        self.current_legacy_snapshot_path = None;
        // Save establishes a fresh "clean" baseline: the on-disk
        // file now matches what the user is editing, so an attempt
        // to close or open another document should no longer prompt
        // for confirmation. Cleared *after* the worker has
        // acknowledged the write so a failed dispatch (the status
        // bar will carry the error in that case) does not falsely
        // advertise a saved state.
        self.dirty = false;
        self.status = format!("Сохранено в {display}");
    }

    /// "Сохранить как": always opens a save dialog, replaces the
    /// remembered path on success. After this, plain "Сохранить" will
    /// overwrite the new path.
    pub(crate) fn save_snapshot_as(&mut self) {
        self.commit_pending_inline_edit();
        let mut dialog = rfd::FileDialog::new().add_filter("KR580 file", &["580"]);
        // Pre-seed the dialog with the current path so the user lands
        // in the same folder with the same filename — the standard
        // "save as" affordance every editor implements.
        if let Some(current) = &self.current_snapshot_path {
            if let Some(parent) = current.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(name) = current.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy().as_ref());
            }
        }
        let Some(path) = dialog.save_file() else {
            return;
        };
        // Same reset rationale as `save_snapshot`: clear any leftover
        // overlay before issuing the dispatch, so a fresh gesture is
        // visually distinguishable from a previous failed one.
        self.clear_error_notice();
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveSnapshot(path.clone()));
        // Same fail-safe pattern: if the write failed, do not retarget
        // the document's primary path or clear the dirty flag. The
        // user can retry, fix the underlying problem (read-only file,
        // missing folder, permissions), or pick a different location.
        if self.error_notice.is_some() {
            return;
        }
        self.current_snapshot_path = Some(path);
        // Same rationale as `save_snapshot`: the document's primary
        // origin is now this v1 path; drop the legacy path so a
        // later Ctrl+Alt+S prompts for a fresh location.
        self.current_legacy_snapshot_path = None;
        // See `save_snapshot` — same baseline-reset rationale: the
        // on-disk file at the new path now matches the live state.
        self.dirty = false;
        self.status = format!("Сохранено в {display}");
    }

    /// "Сохранить (старый формат)": writes the live CPU state out in
    /// the reference 65 549-byte layout — RAM + 13-byte trailer with
    /// PC, no registers / flags / SP / halt / cycles. Behaves like
    /// the regular "Сохранить": if a legacy path is remembered (from
    /// a prior `OpenLegacySnapshot` or a previous successful save),
    /// the file is overwritten in place; otherwise a save picker
    /// runs and the chosen path is stashed in
    /// `current_legacy_snapshot_path` so the next call writes to
    /// that location without prompting.
    ///
    /// The legacy path lives in its own field — separate from the
    /// K580 v1 `current_snapshot_path` — because the two formats
    /// must never share a remembered location: a Ctrl+S after an
    /// Open-legacy would otherwise silently rewrite the legacy file
    /// with K580 v1 bytes the reference emulator could not load
    /// back. By splitting the fields, Ctrl+S always means "save as
    /// v1 to the v1 path" and Ctrl+Alt+S always means "save as
    /// legacy to the legacy path", with no cross-talk.
    ///
    /// Clears `dirty` after the write succeeds so close / open /
    /// save-as guards stop pretending there is still pending work
    /// — earlier this helper deliberately left `dirty` set, which
    /// the user reported as a bug ("после Сохранить всё ещё спрашивает
    /// про несохранённые изменения"). Mirrors `save_snapshot` /
    /// `save_snapshot_as` in that respect.
    pub(crate) fn save_legacy_snapshot(&mut self) {
        self.commit_pending_inline_edit();
        // Already-known path: write straight through without bothering
        // the user with a picker. This is what every text editor does
        // for plain Save and what the user explicitly asked for —
        // "при попытке сохранить файл снова мне предлагает создать
        // новый файл, а не сохраняет в тот же файл".
        let (path, picked_now) = match &self.current_legacy_snapshot_path {
            Some(path) => (path.clone(), false),
            None => {
                let mut dialog = rfd::FileDialog::new().add_filter("KR580 legacy file", &["580"]);
                // Pre-seed the picker with the v1 snapshot's folder
                // when there is one, so the user lands in the
                // directory they were last working in. We do not
                // pre-seed the *file name* — legacy files are an
                // export-style sibling of the active document and
                // reusing the active document's name would make
                // accidental overwrite of the K580 v1 file too easy.
                if let Some(current) = &self.current_snapshot_path
                    && let Some(parent) = current.parent()
                {
                    dialog = dialog.set_directory(parent);
                }
                let Some(path) = dialog.save_file() else {
                    return;
                };
                (path, true)
            }
        };
        // Same reset rationale as `save_snapshot`: clear any leftover
        // overlay before issuing the dispatch.
        self.clear_error_notice();
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveLegacySnapshot(path.clone()));
        // Fail-safe: if the worker rejected the write, don't claim
        // the path (when freshly picked) and don't clear the dirty
        // flag. The legacy file on disk is unchanged; pretending the
        // path now points at it would mislead the next Ctrl+Alt+S
        // into silently overwriting whatever was actually there.
        if self.error_notice.is_some() {
            return;
        }
        if picked_now {
            // Remember the chosen path so the next "Сохранить
            // (старый формат)" goes straight to disk.
            self.current_legacy_snapshot_path = Some(path);
        }
        // The user explicitly chose "Сохранить (старый формат)", the
        // file is now on disk, the gesture has succeeded — clear the
        // dirty flag so the close/open/save-as guards stop pretending
        // there is still pending work. See the doc comment above.
        self.dirty = false;
        self.status = format!("Сохранено в {display} (старый формат)");
    }

    /// "Открыть (старый формат)": reads a 65 549-byte reference
    /// `.580` file produced by the original emulator, replaces RAM
    /// and PC with whatever the file carries, and resets every other
    /// CPU field to its default. Mirrors `load_snapshot_from_path`'s
    /// post-load housekeeping — wipe the run flag, the undo stack,
    /// and the dirty flag, then chase the spinner / inline editor to
    /// the recovered PC — so the user lands on the same idle, clean
    /// baseline a fresh launch would.
    ///
    /// Stashes the loaded file's path in
    /// `current_legacy_snapshot_path` so the next "Сохранить (старый
    /// формат)" overwrites it in place — this is what the user asked
    /// for ("сохраняет в тот же файл"). The K580 v1
    /// `current_snapshot_path` is wiped because the document's
    /// origin is now the legacy file: a subsequent plain Ctrl+S
    /// would otherwise overwrite a v1 file the user did not just
    /// touch and might not even remember.
    pub(crate) fn open_legacy_snapshot(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 legacy file", &["580"])
            .pick_file()
        else {
            return;
        };
        // Same reset rationale as `load_snapshot_from_path`: a fresh
        // gesture earns a clean overlay. If the dispatch below itself
        // errors, `consume_event` will refill `error_notice`
        // synchronously before we read it back.
        self.clear_error_notice();
        let display = path.display().to_string();
        self.running = false;
        self.dispatch_sync(AppCommand::LoadLegacySnapshot(path.clone()));
        // If the worker rejected the load (wrong length, missing FF
        // FF marker, missing file, IO error), `consume_event` already
        // wrote the user-visible notice. Bail out *before* we mutate
        // the document's identity — we don't want to claim "current
        // legacy path = X" for a load that never happened, otherwise
        // the next Ctrl+Alt+S would silently overwrite the unrelated
        // file at X with the live CPU state from before the failed
        // open.
        if self.error_notice.is_some() {
            return;
        }
        // Wipe undo/redo *after* the load — see
        // `load_snapshot_from_path` for the rationale; the legacy
        // load is the same kind of "fresh starting point" gesture.
        self.undo_stack.clear();
        // Document origin is now the legacy file: remember its path
        // so Ctrl+Alt+S overwrites it, and clear the v1 path so a
        // stray Ctrl+S does not clobber an unrelated v1 file the
        // user was working on earlier.
        self.current_snapshot_path = None;
        self.current_legacy_snapshot_path = Some(path);
        self.dirty = false;
        let pc = self.snapshot.cpu.pc;
        self.set_memory_address(pc);
        self.status = format!("Открыто {display} (старый формат)");
    }

    /// If the user has typed a hex byte into the inline memory editor
    /// but did not press Enter, push that pending value to the worker
    /// before saving. This is the fix for "сохранил файл, открыл — он
    /// пустой": the inline editor's `change_inline_memory_value` only
    /// updates the input buffer (`memory_inline_value_input`), the byte
    /// reaches the CPU only on `apply_inline_memory_value`. Saving
    /// without this commit would serialise stale RAM.
    ///
    /// The commit goes through `dispatch_with_undo` so the rolled-up
    /// edit lands on the undo stack just like an explicit Enter press
    /// would: the user typed a byte, walked away to save, and the
    /// flush is logically the same gesture as committing it. Without
    /// this Ctrl+Z would silently skip past the auto-flushed write
    /// even though the visible state changed underneath. Coalescing
    /// is broken first so the inline-text undo entries that piled up
    /// while typing don't fold into whatever the next text edit will
    /// emit after the save returns.
    fn commit_pending_inline_edit(&mut self) {
        let Ok(address) = parse_hex_u16(&self.memory_address_input) else {
            return;
        };
        let Ok(value) = u8::from_str_radix(self.memory_inline_value_input.trim(), 16) else {
            return;
        };
        if self.snapshot.cpu.memory.read(address) == value {
            return;
        }
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn export_file(&mut self) {
        // Make sure any pending inline edit is committed first; the
        // exporter reads the snapshot we are about to send out and
        // would otherwise drop whatever the user has typed but not
        // yet confirmed with Enter.
        self.commit_pending_inline_edit();
        // Two separate filters so the OS "Save as type" dropdown has
        // entries the user can pick between, and rfd's Windows backend
        // (`SetDefaultExtension` from the first filter) auto-appends the
        // right extension when the typed name has none. TXT is listed
        // first so it is the fallback default — XLSX requires the user
        // to actively pick the spreadsheet entry.
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 text export", &["txt"])
            .add_filter("KR580 spreadsheet export", &["xlsx"])
            .save_file()
        else {
            return;
        };
        // Final routing: only the actual extension on disk decides the
        // command, never the dropdown selection (rfd does not expose
        // it). Anything that is not `.xlsx` is normalised to `.txt`
        // with the `.txt` suffix appended — that way `mysnap.foo`
        // becomes `mysnap.foo.txt`, the user sees the rename in the
        // status bar, and the file content matches its extension. The
        // alternative — silently writing TXT bytes into a `.foo` file
        // — was the bug the user reported.
        let path = normalise_export_path(path);
        // Same reset rationale as the file-open helpers: clear any
        // leftover overlay before issuing the dispatch so the user
        // can tell *this* gesture's outcome apart from the previous
        // one.
        self.clear_error_notice();
        let display = path.display().to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        // Switched to `dispatch_sync` so a write failure surfaces
        // before we lie to the user via the status bar — the previous
        // fire-and-forget `dispatch` would let the success message
        // race the worker's error event and the user would see
        // "Экспорт в …" *and* the red overlay one tick later, in
        // that order, which read as "saved but also broken".
        match extension.as_deref() {
            Some("xlsx") => self.dispatch_sync(AppCommand::ExportXlsx(path)),
            _ => self.dispatch_sync(AppCommand::ExportTxt(path)),
        }
        if self.error_notice.is_some() {
            return;
        }
        self.status = format!("Экспорт в {display}");
    }

    pub(crate) fn import_file(&mut self) {
        // The first filter combines both extensions so the picker
        // surfaces TXT and XLSX side-by-side by default — the user
        // does not have to flip the dropdown to see a snapshot they
        // know they exported. The two narrow filters stay below for
        // when the user explicitly wants to scope the listing to one
        // format. Unlike export, import takes the file as-is — the
        // user is pointing at an existing file, so no extension
        // rewriting; we only look at the actual extension to pick the
        // parser. Falls through to TXT for a missing/unknown extension
        // because that is the more lenient parser and the user gets a
        // typed error in the status bar instead of a silent no-op.
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["txt", "xlsx"])
            .add_filter("KR580 txt file", &["txt"])
            .add_filter("KR580 spreadsheet file", &["xlsx"])
            .pick_file()
        else {
            return;
        };
        // Same reset rationale as `load_snapshot_from_path`: clear any
        // leftover overlay so a fresh import gesture is visually
        // distinguishable from a previous failed one.
        self.clear_error_notice();
        // Disarm the cosmetic run flag for the same reason
        // `load_snapshot_from_path` does: a prior `toggle_run` on a
        // blank page leaves `self.running == true` without any actual
        // worker activity, and the import then layers fresh bytes on
        // top of that stale armed state — the play/pause icon stays
        // red, `Message::Tick` keeps trying to follow PC, and the
        // user sees a program that *looks* like it is running while
        // the worker is in fact idle.
        self.running = false;
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        // Import is a "fresh starting point" gesture, same as
        // `load_snapshot_from_path` — the user wouldn't expect
        // Ctrl+Z to tear the freshly imported program back out and
        // restore whatever scratchpad they had loaded before. So
        // instead of pushing a `Cpu` undo pair, wipe the timeline
        // after the worker has applied the import. Sync dispatch
        // makes the wipe land on the post-import state rather than
        // a half-applied one.
        match extension.as_deref() {
            Some("xlsx") => self.dispatch_sync(AppCommand::ImportXlsx(path)),
            _ => self.dispatch_sync(AppCommand::ImportTxt(path)),
        }
        // If the worker rejected the import (parse error, missing
        // file, malformed cells), `consume_event` already wrote the
        // user-visible notice. Bail out *before* we wipe the undo
        // stack and clear `dirty` — the document is unchanged, so
        // pretending the timeline started fresh would silently
        // discard the user's prior edit history.
        if self.error_notice.is_some() {
            return;
        }
        self.undo_stack.clear();
        // Import is a "fresh starting point" too — see the
        // load_snapshot rationale. The imported document is now what
        // the user expects to see; until they edit it, prompting on
        // close/open would be noise. Note import does *not* set
        // `current_snapshot_path` (the imported file is not a `.580`
        // snapshot), so a subsequent Ctrl+S still asks the user
        // where to save — that part of the flow is unchanged. We
        // also drop `current_legacy_snapshot_path` for the same
        // reason: the document's origin is now a TXT/XLSX import,
        // not the legacy `.580` we may have had open before, so
        // Ctrl+Alt+S must prompt for a fresh location rather than
        // silently overwrite an unrelated legacy file.
        self.current_legacy_snapshot_path = None;
        self.dirty = false;
    }
}

/// Forces the export path to end in `.txt` or `.xlsx`. Anything else
/// (no extension, `.foo`, `.png`, …) is treated as TXT and gets `.txt`
/// appended to the existing name — `mysnap.foo` becomes
/// `mysnap.foo.txt`, `mysnap` becomes `mysnap.txt`. Without this the
/// dispatcher silently wrote TXT bytes into files with arbitrary
/// extensions, which made downstream tools choke.
fn normalise_export_path(path: PathBuf) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    match extension.as_deref() {
        Some("txt") | Some("xlsx") => path,
        _ => {
            // `with_extension` would *replace* `.foo` with `.txt`,
            // discarding whatever the user typed. Appending instead
            // keeps the user's name visible (`mysnap.foo.txt`) so the
            // status-bar message and the file in Explorer line up.
            let mut as_string = path.into_os_string();
            as_string.push(".txt");
            PathBuf::from(as_string)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalise_export_path;
    use std::path::PathBuf;

    /// Paths that already end in a supported export extension are
    /// returned untouched — uppercase variants normalise too, since
    /// the routing match in `export_file` lowercases the extension
    /// before comparing.
    #[test]
    fn keeps_supported_extensions_intact() {
        for already_ok in ["a.txt", "a.xlsx", "a.TXT", "a.XLSX", "deep/path/file.txt"] {
            assert_eq!(
                normalise_export_path(PathBuf::from(already_ok)),
                PathBuf::from(already_ok),
                "{already_ok} should be left as-is"
            );
        }
    }

    /// The exact bug the user hit: typing `.foo` in the dialog used
    /// to silently write TXT bytes into a `.foo` file. Now we append
    /// `.txt` instead of replacing, so the original name stays
    /// visible to the user.
    #[test]
    fn appends_txt_to_unknown_extension() {
        assert_eq!(
            normalise_export_path(PathBuf::from("mysnap.foo")),
            PathBuf::from("mysnap.foo.txt"),
        );
        assert_eq!(
            normalise_export_path(PathBuf::from("dump.png")),
            PathBuf::from("dump.png.txt"),
        );
    }

    /// A bare name with no extension is the canonical "user did not
    /// pick a format" case — TXT is the documented default fallback.
    #[test]
    fn appends_txt_when_no_extension() {
        assert_eq!(
            normalise_export_path(PathBuf::from("plain")),
            PathBuf::from("plain.txt"),
        );
    }

    /// Names that start with a dot but have no extension proper
    /// (`.bashrc`, `.gitignore`) are treated by `Path::extension` as
    /// having no extension — that means the suffix gets appended,
    /// producing `.bashrc.txt`. This is intentional: we never strip
    /// leading dots, so the dotfile stays a dotfile and just gains a
    /// real extension.
    #[test]
    fn dotfiles_get_txt_appended() {
        assert_eq!(
            normalise_export_path(PathBuf::from(".bashrc")),
            PathBuf::from(".bashrc.txt"),
        );
    }
}
