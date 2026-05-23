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
mod memory;
mod parse;
mod register;
mod undo;

pub(crate) use focus_ops::{find_focusable_at, find_focused_optional, unfocus_except};

use crate::app::DesktopApp;
use k580_app::{AppCommand, AppEvent, AppSnapshot};
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
            self.halt_notice =
                Some("Программа завершена. Сброс регистров для повторного запуска.".to_owned());
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
            self.status = format!("No program at {pc:04X}");
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
                self.status = format!("Tact {} cycle {}", outcome.tact_phase, outcome.cycle_count)
            }
            AppEvent::PortRead { port, value } => {
                self.status = format!("IN {port:02X} -> {value:02X}")
            }
            AppEvent::PortWritten { port, value } => {
                self.status = format!("OUT {port:02X} <- {value:02X}")
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
                self.status = "CPU halted".to_owned();
            }
            AppEvent::ErrorRaised(error) => {
                // An error from the core or the bus also auto-pauses the
                // worker; keep UI state aligned for the same reason as
                // the halt branch above.
                self.running = false;
                self.pending_follow_pc = true;
                self.status = error.to_string();
            }
            AppEvent::Stopped => {
                // Sent both for an explicit `AppCommand::Stop` and when
                // the worker auto-pauses (`MAX_INSTRUCTIONS_PER_RUN`
                // exhausted or halt/error path). Either way the run is
                // no longer armed; clear the flag so `toggle_run` and
                // the Tick handler agree with the worker.
                self.running = false;
                self.pending_follow_pc = true;
                self.status = "Stopped".to_owned();
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
            self.halt_notice = None;
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
    /// opener). Stores the path so subsequent "Сохранить" overwrites
    /// it in place, runs `dispatch_sync` so the worker has applied
    /// the load before we re-derive the spinner address, and finally
    /// pulls the new PC into the memory list / inline editor so the
    /// visible state matches what is actually in memory.
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
        self.current_snapshot_path = Some(path.clone());
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
        self.dispatch_sync(AppCommand::LoadSnapshot(path));
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
        self.status = format!("Открыто {display}");
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
        self.current_snapshot_path = Some(path.clone());
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::SaveSnapshot(path));
        // See `save_snapshot` — same baseline-reset rationale: the
        // on-disk file at the new path now matches the live state.
        self.dirty = false;
        self.status = format!("Сохранено в {display}");
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
        let display = path.display().to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        match extension.as_deref() {
            Some("xlsx") => self.dispatch(AppCommand::ExportXlsx(path)),
            _ => self.dispatch(AppCommand::ExportTxt(path)),
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
        self.undo_stack.clear();
        // Import is a "fresh starting point" too — see the
        // load_snapshot rationale. The imported document is now what
        // the user expects to see; until they edit it, prompting on
        // close/open would be noise. Note import does *not* set
        // `current_snapshot_path` (the imported file is not a `.580`
        // snapshot), so a subsequent Ctrl+S still asks the user
        // where to save — that part of the flow is unchanged.
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
