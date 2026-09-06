use iced::Task;
use iced::advanced::widget::operation::focusable::unfocus;
use iced::advanced::widget::{Id, operate};
use iced::widget::operation;
use std::path::PathBuf;

use super::StatusKind;
use super::messages::Message;
use super::state::DesktopApp;
use crate::i18n::Key;
use crate::persistence::SubprogramSerializer;
use crate::runtime::parse::{bounded_hex_input, parse_hex_u16};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SubprogramDialogMode {
    Open,
    Save,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SubprogramDialogFocus {
    Start,
    End,
    Cancel,
    Confirm,
}

impl SubprogramDialogFocus {
    pub(crate) fn input_id(self) -> Option<&'static str> {
        match self {
            Self::Start => Some("subprogram-start"),
            Self::End => Some("subprogram-end"),
            Self::Cancel | Self::Confirm => None,
        }
    }

    fn next(self, mode: SubprogramDialogMode) -> Self {
        match (mode, self) {
            (SubprogramDialogMode::Open, Self::Start) => Self::Cancel,
            (SubprogramDialogMode::Open, Self::Cancel) => Self::Confirm,
            (SubprogramDialogMode::Open, Self::Confirm) => Self::Start,
            (SubprogramDialogMode::Save, Self::Start) => Self::End,
            (SubprogramDialogMode::Save, Self::End) => Self::Cancel,
            (SubprogramDialogMode::Save, Self::Cancel) => Self::Confirm,
            (SubprogramDialogMode::Save, Self::Confirm) => Self::Start,
            (SubprogramDialogMode::Open, Self::End) => Self::Cancel,
        }
    }

    fn previous(self, mode: SubprogramDialogMode) -> Self {
        match (mode, self) {
            (SubprogramDialogMode::Open, Self::Start) => Self::Confirm,
            (SubprogramDialogMode::Open, Self::Cancel) => Self::Start,
            (SubprogramDialogMode::Open, Self::Confirm) => Self::Cancel,
            (SubprogramDialogMode::Save, Self::Start) => Self::Confirm,
            (SubprogramDialogMode::Save, Self::End) => Self::Start,
            (SubprogramDialogMode::Save, Self::Cancel) => Self::End,
            (SubprogramDialogMode::Save, Self::Confirm) => Self::Cancel,
            (SubprogramDialogMode::Open, Self::End) => Self::Start,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SubprogramDialog {
    pub(crate) mode: SubprogramDialogMode,
    pub(crate) path: PathBuf,
    pub(crate) start_input: String,
    pub(crate) end_input: String,
    pub(crate) focus: SubprogramDialogFocus,
    pub(crate) keyboard_focus_visible: bool,
    pub(crate) error: Option<String>,
}

impl DesktopApp {
    pub(crate) fn open_subprogram_dialog(&mut self, path: PathBuf, mode: SubprogramDialogMode) {
        let start = parse_hex_u16(&self.memory_address_input)
            .or_else(|| self.selected_memory_address())
            .unwrap_or(0);
        self.close_top_menu();
        self.hide_opcode_dropdown();
        self.subprogram_dialog = Some(SubprogramDialog {
            mode,
            path,
            start_input: format!("{start:04X}"),
            end_input: "FFFF".to_owned(),
            focus: SubprogramDialogFocus::Cancel,
            keyboard_focus_visible: false,
            error: None,
        });
    }

    pub(crate) fn route_subprogram_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        let dialog = self.subprogram_dialog.as_mut()?;

        match message {
            Message::Tick | Message::CursorMoved(_) | Message::ModifiersChanged(_) => None,
            Message::SubprogramStartChanged(value) => {
                if let Some(value) = bounded_hex_input(value, 4) {
                    dialog.start_input = value;
                    dialog.focus = SubprogramDialogFocus::Start;
                    dialog.error = None;
                }
                Some(Task::none())
            }
            Message::SubprogramEndChanged(value) => {
                if let Some(value) = bounded_hex_input(value, 4) {
                    dialog.end_input = value;
                    dialog.focus = SubprogramDialogFocus::End;
                    dialog.error = None;
                }
                Some(Task::none())
            }
            Message::ConfirmSubprogram => {
                self.confirm_subprogram();
                Some(Task::none())
            }
            Message::CancelSubprogram | Message::EscPressed => {
                self.subprogram_dialog = None;
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                dialog.keyboard_focus_visible = true;
                dialog.focus = if *backward {
                    dialog.focus.previous(dialog.mode)
                } else {
                    dialog.focus.next(dialog.mode)
                };
                Some(match dialog.focus.input_id() {
                    Some(id) => operation::focus(id),
                    None => operate(unfocus()),
                })
            }
            Message::EnterPressed => {
                match dialog.focus {
                    SubprogramDialogFocus::Confirm => self.confirm_subprogram(),
                    SubprogramDialogFocus::Cancel => self.subprogram_dialog = None,
                    SubprogramDialogFocus::Start | SubprogramDialogFocus::End => {}
                }
                Some(Task::none())
            }
            Message::SubprogramFocusResolved(hit) => {
                let focus = [SubprogramDialogFocus::Start, SubprogramDialogFocus::End]
                    .into_iter()
                    .find(|focus| {
                        hit.as_ref().is_some_and(|id| {
                            focus.input_id().is_some_and(|name| *id == Id::new(name))
                        })
                    });
                if let (Some(focus), Some(id)) = (focus, hit) {
                    dialog.focus = focus;
                    Some(operate(crate::runtime::unfocus_except(id.clone())).discard())
                } else {
                    Some(operate(unfocus()))
                }
            }
            Message::MousePressed | Message::MousePressedIgnored => {
                dialog.keyboard_focus_visible = false;
                Some(
                    operate(crate::runtime::find_focusable_at(
                        self.latest_cursor_position,
                    ))
                    .map(Message::SubprogramFocusResolved),
                )
            }
            _ => Some(Task::none()),
        }
    }

    fn confirm_subprogram(&mut self) {
        let Some(dialog) = self.subprogram_dialog.take() else {
            return;
        };
        let start = match parse_hex_u16(&dialog.start_input) {
            Some(start) => start,
            None => return self.restore_subprogram_error(dialog, Key::StatusInvalidAddressHex),
        };
        let end = match dialog.mode {
            SubprogramDialogMode::Open => match SubprogramSerializer::file_end(&dialog.path, start)
            {
                Ok(end) => end,
                Err(error) => {
                    return self.restore_subprogram_error_text(
                        dialog,
                        crate::runtime::humanize_error::humanize(&error.to_string(), self.lang),
                    );
                }
            },
            SubprogramDialogMode::Save => match parse_hex_u16(&dialog.end_input) {
                Some(end) if start <= end => end,
                Some(_) => {
                    return self.restore_subprogram_error(dialog, Key::SubprogramRangeInvalid);
                }
                None => return self.restore_subprogram_error(dialog, Key::StatusInvalidAddressHex),
            },
        };

        self.clear_error_notice();
        let path = dialog.path.clone();
        let display = path.display().to_string();
        let command = match dialog.mode {
            SubprogramDialogMode::Open => crate::backend::AppCommand::LoadSubprogram {
                path: path.clone(),
                start,
            },
            SubprogramDialogMode::Save => crate::backend::AppCommand::SaveSubprogram {
                path: path.clone(),
                start,
                end,
            },
        };
        self.dispatch_sync(command);
        if let Some(error) = self.error_notice.take() {
            self.error_notice_dismiss_at = None;
            return self.restore_subprogram_error_text(dialog, error);
        }

        self.current_snapshot_path = Some(path);
        self.current_subprogram_range = Some((start, end));
        self.undo_stack.clear();
        match dialog.mode {
            SubprogramDialogMode::Open => self.mark_saved(),
            SubprogramDialogMode::Save => self.mark_subprogram_saved(start, end),
        }
        self.set_memory_address(start);
        self.set_status(match dialog.mode {
            SubprogramDialogMode::Open => StatusKind::Opened { display },
            SubprogramDialogMode::Save => StatusKind::SavedTo { display },
        });
    }

    fn restore_subprogram_error(&mut self, dialog: SubprogramDialog, key: Key) {
        self.restore_subprogram_error_text(dialog, self.lang.t(key).to_owned());
    }

    fn restore_subprogram_error_text(&mut self, mut dialog: SubprogramDialog, error: String) {
        dialog.error = Some(error);
        self.subprogram_dialog = Some(dialog);
    }
}

#[cfg(test)]
mod tests {
    use super::{SubprogramDialogFocus, SubprogramDialogMode};
    use crate::app::{DesktopApp, Message};

    #[test]
    fn subprogram_saves_preserve_unsaved_memory_and_registers() {
        use crate::backend::AppCommand;
        use k580_core::RegisterName;

        let dir = std::env::temp_dir().join(format!("kr580-partial-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("partial.krs");
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.dispatch_sync(AppCommand::ApplyCpuState(Box::default()));
        app.mark_saved();
        app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0x76));
        app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0x42));
        app.dispatch_with_undo(AppCommand::SetRegister(RegisterName::A, 0x12));
        app.open_subprogram_dialog(path.clone(), SubprogramDialogMode::Save);
        let dialog = app.subprogram_dialog.as_mut().unwrap();
        dialog.start_input = "0100".into();
        dialog.end_input = "0100".into();
        app.confirm_subprogram();
        assert!(app.error_notice.is_none());
        assert!(app.subprogram_dialog.is_none());
        assert_eq!(std::fs::read(&path).unwrap(), [0x76]);
        assert!(app.dirty);
        assert_eq!(app.saved_cpu.memory.read(0x0100), 0x76);
        assert_eq!(app.saved_cpu.memory.read(0x2000), 0);
        assert_eq!(app.saved_cpu.registers.a, 0);

        app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0));
        assert!(app.dirty);
        app.dispatch_with_undo(AppCommand::SetRegister(RegisterName::A, 0));
        app.dispatch_with_undo(AppCommand::SetPc(0));
        assert!(!app.dirty);
        app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0xC9));
        app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0x33));
        app.memory_inline_value_input.clear();
        let _ = app.save_program();
        assert_eq!(std::fs::read(&path).unwrap(), [0xC9]);
        assert!(app.dirty);
        app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0));
        assert!(!app.dirty);

        app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0xFF));
        let saved = app.saved_cpu.clone();
        app.current_snapshot_path = Some(dir.join("missing/partial.krs"));
        let _ = app.save_program();
        assert!(app.error_notice.is_some());
        assert!(app.dirty);
        assert_eq!(app.saved_cpu, saved);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn address_edits_accept_only_up_to_four_hex_digits() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.open_subprogram_dialog("unused.krs".into(), SubprogramDialogMode::Save);
        for (input, expected) in [
            ("1a2f", "1A2F"),
            ("1A2F5", "1A2F"),
            ("12g4", "1A2F"),
            ("", ""),
            ("ffff", "FFFF"),
        ] {
            let _ = app.update(Message::SubprogramStartChanged(input.into()));
            let _ = app.update(Message::SubprogramEndChanged(input.into()));
            let dialog = app.subprogram_dialog.as_ref().unwrap();
            assert_eq!(dialog.start_input, expected, "start: {input}");
            assert_eq!(dialog.end_input, expected, "end: {input}");
        }
    }

    #[test]
    fn dialogs_start_on_cancel_and_enter_closes_without_loading_or_saving() {
        for mode in [SubprogramDialogMode::Open, SubprogramDialogMode::Save] {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            app.open_subprogram_dialog("unused.krs".into(), mode);
            let dialog = app.subprogram_dialog.as_ref().unwrap();
            assert_eq!(dialog.focus, SubprogramDialogFocus::Cancel);
            assert!(!dialog.keyboard_focus_visible);
            let _ = app.update(Message::EnterPressed);
            assert!(app.subprogram_dialog.is_none());
            assert!(app.current_snapshot_path.is_none());
        }
    }

    #[test]
    fn tab_traverses_visible_controls_in_both_directions_after_mouse_focus() {
        use SubprogramDialogFocus::{Cancel, Confirm, End, Start};

        for mode in [SubprogramDialogMode::Open, SubprogramDialogMode::Save] {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            app.open_subprogram_dialog("unused.krs".into(), mode);
            let _ = app.update(Message::SubprogramFocusResolved(Some(
                iced::widget::Id::new(SubprogramDialogFocus::Start.input_id().unwrap()),
            )));
            let ring: &[_] = match mode {
                SubprogramDialogMode::Open => &[Start, Cancel, Confirm],
                SubprogramDialogMode::Save => &[Start, End, Cancel, Confirm],
            };
            for backward in [false, true] {
                for step in 1..=ring.len() {
                    let _ = app.update(Message::FocusCycle { backward });
                    let index = if backward {
                        ring.len() - step
                    } else {
                        step % ring.len()
                    };
                    let dialog = app.subprogram_dialog.as_ref().unwrap();
                    assert_eq!(dialog.focus, ring[index]);
                    assert!(dialog.keyboard_focus_visible);
                }
            }
        }
    }
}
