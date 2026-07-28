use iced::Task;
use std::path::PathBuf;

use super::StatusKind;
use super::messages::Message;
use super::state::DesktopApp;
use crate::i18n::Key;
use crate::persistence::SubprogramSerializer;
use crate::runtime::parse::parse_hex_u16;

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
            focus: SubprogramDialogFocus::Start,
            keyboard_focus_visible: false,
            error: None,
        });
    }

    pub(crate) fn route_subprogram_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        self.subprogram_dialog.as_ref()?;

        match message {
            Message::Tick | Message::CursorMoved(_) | Message::ModifiersChanged(_) => None,
            Message::SubprogramStartChanged(value) => {
                if let Some(dialog) = self.subprogram_dialog.as_mut() {
                    dialog.start_input = value.clone();
                    dialog.error = None;
                }
                Some(Task::none())
            }
            Message::SubprogramEndChanged(value) => {
                if let Some(dialog) = self.subprogram_dialog.as_mut() {
                    dialog.end_input = value.clone();
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
                self.cycle_subprogram_focus(*backward);
                Some(Task::none())
            }
            Message::EnterPressed => {
                if self
                    .subprogram_dialog
                    .as_ref()
                    .is_some_and(|dialog| dialog.focus == SubprogramDialogFocus::Confirm)
                {
                    self.confirm_subprogram();
                }
                Some(Task::none())
            }
            Message::MousePressed | Message::MousePressedIgnored => {
                if let Some(dialog) = self.subprogram_dialog.as_mut() {
                    dialog.keyboard_focus_visible = false;
                }
                Some(Task::none())
            }
            _ => Some(Task::none()),
        }
    }

    fn cycle_subprogram_focus(&mut self, backward: bool) {
        if let Some(dialog) = self.subprogram_dialog.as_mut() {
            dialog.keyboard_focus_visible = true;
            dialog.focus = if backward {
                dialog.focus.previous(dialog.mode)
            } else {
                dialog.focus.next(dialog.mode)
            };
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
        self.mark_saved();
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

    #[test]
    fn open_dialog_skips_end_address() {
        assert_eq!(
            SubprogramDialogFocus::Start.next(SubprogramDialogMode::Open),
            SubprogramDialogFocus::Cancel
        );
        assert_eq!(
            SubprogramDialogFocus::Cancel.previous(SubprogramDialogMode::Open),
            SubprogramDialogFocus::Start
        );
    }

    #[test]
    fn save_dialog_cycles_through_range_and_actions() {
        assert_eq!(
            SubprogramDialogFocus::Start.next(SubprogramDialogMode::Save),
            SubprogramDialogFocus::End
        );
        assert_eq!(
            SubprogramDialogFocus::Confirm.next(SubprogramDialogMode::Save),
            SubprogramDialogFocus::Start
        );
    }
}
