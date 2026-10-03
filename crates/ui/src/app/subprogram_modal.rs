use iced::Task;
use iced::advanced::widget::operation::focusable::unfocus;
use iced::advanced::widget::{Id, operate};
use iced::widget::operation;
use std::path::PathBuf;

use super::StatusKind;
use super::messages::Message;
use super::state::DesktopApp;
use crate::i18n::Key;
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
        self.clear_error_notice();
        if dialog.mode == SubprogramDialogMode::Open {
            self.dispatch_pending_request(
                crate::backend::AppCommand::LoadSubprogram {
                    path: dialog.path.clone(),
                    start,
                },
                super::pending::PendingRequest::LoadSubprogram { dialog, start },
            );
            return;
        }
        let end = match parse_hex_u16(&dialog.end_input) {
            Some(end) if start <= end => end,
            Some(_) => return self.restore_subprogram_error(dialog, Key::SubprogramRangeInvalid),
            None => return self.restore_subprogram_error(dialog, Key::StatusInvalidAddressHex),
        };
        let path = dialog.path;
        let display = path.display().to_string();
        self.dispatch_pending_request(
            crate::backend::AppCommand::SaveSubprogram {
                path: path.clone(),
                start,
                end,
            },
            super::pending::PendingRequest::SaveSubprogram {
                path,
                display,
                start,
                end,
                state: Box::new(self.snapshot.cpu.clone()),
            },
        );
    }

    pub(crate) fn finish_subprogram_load(&mut self, path: PathBuf, start: u16, end: u16) {
        let display = path.display().to_string();
        self.current_snapshot_path = Some(path);
        self.current_subprogram_range = Some((start, end));
        self.undo_stack.clear();
        self.mark_saved();
        self.set_memory_address(start);
        self.set_status(StatusKind::Opened { display });
    }

    fn restore_subprogram_error(&mut self, dialog: SubprogramDialog, key: Key) {
        self.restore_subprogram_error_text(dialog, self.lang.t(key).to_owned());
    }

    pub(crate) fn restore_subprogram_error_text(
        &mut self,
        mut dialog: SubprogramDialog,
        error: String,
    ) {
        dialog.error = Some(error);
        self.subprogram_dialog = Some(dialog);
    }
}

#[cfg(test)]
#[path = "subprogram_modal/tests.rs"]
mod tests;
