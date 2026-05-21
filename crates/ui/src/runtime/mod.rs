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

pub(crate) use focus_ops::{find_focusable_at, unfocus_except};

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
    /// button and conditionally dispatches the underlying CPU command.
    /// Mirrors the reference KR-580 emulator: clicking the play glyph on a
    /// page that has no program (the byte at `cpu.pc` is `0x00`) is purely
    /// cosmetic — the icon swaps to a red pause and back without burning
    /// any T-states. When a program *is* present the toggle still drives
    /// the real `Run` / `Stop` commands, so a user who has actually loaded
    /// code keeps the original behaviour.
    pub(crate) fn toggle_run(&mut self) {
        self.running = !self.running;
        let pc = self.snapshot.cpu.pc;
        let has_program = self.snapshot.cpu.memory.read(pc) != 0;
        if !has_program {
            // Empty memory at PC — only the icon flips. No CPU work, no
            // status churn beyond a hint so the user understands why
            // nothing is moving.
            self.status = if self.running {
                format!("No program at {pc:04X}")
            } else {
                "Stopped".to_owned()
            };
            return;
        }
        if self.running {
            self.dispatch(AppCommand::Run);
        } else {
            self.dispatch(AppCommand::Stop);
        }
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
        self.dispatch(AppCommand::ResetCpu);
        // Keep the run state armed and dispatch the run command directly,
        // so the visible play/pause toggle stays consistent with what the
        // CPU is actually doing.
        self.running = true;
        self.dispatch(AppCommand::Run);
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
            AppEvent::HaltStateChanged(_) => self.status = "CPU halted".to_owned(),
            AppEvent::ErrorRaised(error) => self.status = error.to_string(),
            AppEvent::Stopped => self.status = "Stopped".to_owned(),
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
    pub(crate) fn load_snapshot_from_path(&mut self, path: PathBuf) {
        self.current_snapshot_path = Some(path.clone());
        let display = path.display().to_string();
        self.dispatch_sync(AppCommand::LoadSnapshot(path));
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
        self.status = format!("Сохранено в {display}");
    }

    /// If the user has typed a hex byte into the inline memory editor
    /// but did not press Enter, push that pending value to the worker
    /// before saving. This is the fix for "сохранил файл, открыл — он
    /// пустой": the inline editor's `change_inline_memory_value` only
    /// updates the input buffer (`memory_inline_value_input`), the byte
    /// reaches the CPU only on `apply_inline_memory_value`. Saving
    /// without this commit would serialise stale RAM.
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
        self.dispatch_sync(AppCommand::SetMemory(address, value));
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
        // Two separate filters so the OS "Files of type" dropdown
        // shows TXT and XLSX as distinct entries the user can switch
        // between. Unlike export, import takes the file as-is — the
        // user is pointing at an existing file, so no extension
        // rewriting; we only look at the actual extension to pick the
        // parser. Falls through to TXT for a missing/unknown extension
        // because that is the more lenient parser and the user gets a
        // typed error in the status bar instead of a silent no-op.
        let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 text export", &["txt"])
            .add_filter("KR580 spreadsheet export", &["xlsx"])
            .pick_file()
        else {
            return;
        };
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        match extension.as_deref() {
            Some("xlsx") => self.dispatch(AppCommand::ImportXlsx(path)),
            _ => self.dispatch(AppCommand::ImportTxt(path)),
        }
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
