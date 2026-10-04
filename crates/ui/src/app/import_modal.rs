mod loading;
use super::{DesktopApp, ImportFileFormat, ImportModalFocus, Message};
use crate::backend::AppCommand;
use crate::i18n::Key;
use crate::runtime::file_dialog;
use iced::{Event, Task, window};

impl DesktopApp {
    pub(crate) fn open_import_modal(&mut self) {
        self.close_export_modal();
        self.import.import_modal_open = true;
        self.import.import_modal_focus = ImportModalFocus::Browse;
        self.import.import_modal_keyboard_focus_visible = false;
        self.import.import_file_drag_hovered = false;
        self.clear_import_file_selection();
        self.import.import_error = None;
        self.close_top_menu();
        self.hide_opcode_dropdown();
        self.close_open_device_panel();
    }

    pub(crate) fn close_import_modal(&mut self) {
        self.import.import_modal_open = false;
        self.import.generation = self.import.generation.wrapping_add(1);
        self.import.loading = false;
        self.import.import_modal_focus = ImportModalFocus::Browse;
        self.import.import_modal_keyboard_focus_visible = false;
        self.import.import_file_drag_hovered = false;
        self.import.import_target_dropdown.set_open(false);
    }

    pub(super) fn handle_import_file_drag_event(&mut self, event: &Event) -> bool {
        if !self.import.import_modal_open {
            return false;
        }
        match event {
            Event::Window(window::Event::FileHovered(_)) => {
                self.import.import_file_drag_hovered = true;
                true
            }
            Event::Window(window::Event::FileDropped(path)) => {
                self.import.import_file_drag_hovered = false;
                self.load_import_file(path.clone());
                true
            }
            Event::Window(window::Event::FilesHoveredLeft) => {
                self.import.import_file_drag_hovered = false;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn route_import_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        if !self.import.import_modal_open {
            return None;
        }

        if !matches!(
            message,
            Message::Tick
                | Message::CursorMoved(_)
                | Message::ModifiersChanged(_)
                | Message::FocusCycle { .. }
        ) {
            self.import.import_modal_keyboard_focus_visible = false;
        }

        match message {
            Message::Export => None,
            Message::Tick | Message::CursorMoved(_) | Message::ModifiersChanged(_) => None,
            Message::ImportFileBrowse => Some(self.choose_import_file()),
            Message::ImportFileSelected(path) => {
                self.load_import_file(path.clone());
                Some(Task::none())
            }
            Message::ImportTargetDropdownToggled => {
                self.toggle_import_target_dropdown();
                Some(Task::none())
            }
            Message::ImportTargetSelected(value) => {
                self.select_import_target(value.clone());
                Some(Task::none())
            }
            Message::ConfirmImport => Some(self.confirm_import()),
            Message::CancelImport => {
                self.close_import_modal();
                Some(Task::none())
            }
            Message::EscPressed => {
                self.close_import_modal();
                Some(Task::none())
            }
            Message::MousePressedIgnored => {
                self.import.import_target_dropdown.set_open(false);
                self.import.import_modal_focus = ImportModalFocus::None;
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                self.cycle_import_modal_focus(*backward);
                self.import.import_modal_keyboard_focus_visible = true;
                Some(Task::none())
            }
            Message::ArrowKey(direction) if self.import.import_target_dropdown.is_open() => {
                self.move_import_target_highlight(*direction);
                Some(Task::none())
            }
            Message::EnterPressed if self.import.import_target_dropdown.is_open() => {
                self.submit_import_target_dropdown();
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.submit_import_modal_focus()),
            _ => Some(Task::none()),
        }
    }

    pub(crate) fn confirm_import(&mut self) -> Task<Message> {
        if self.import.loading || self.import.import_error.is_some() {
            return Task::none();
        }
        let (Some(path), Some(format)) = (
            self.import.import_file_path.clone(),
            self.import.import_file_format,
        ) else {
            self.import.import_error = Some(
                self.preferences
                    .lang
                    .t(Key::ImportChooseFileRequired)
                    .to_owned(),
            );
            self.import.import_modal_focus = ImportModalFocus::Browse;
            return Task::none();
        };
        let display = self.import.import_file_display.clone();
        let target = self.import.import_target_input.trim().to_owned();
        let command = match format {
            ImportFileFormat::Xlsx if !target.is_empty() => {
                AppCommand::ImportXlsxSheet(path, target)
            }
            ImportFileFormat::Xlsx => AppCommand::ImportXlsx(path),
            ImportFileFormat::Text if !target.is_empty() => {
                AppCommand::ImportTxtSection(path, target)
            }
            ImportFileFormat::Text => AppCommand::ImportTxt(path),
        };

        self.close_import_modal();
        self.clear_error_notice();
        self.execution.running = false;
        self.dispatch_pending_request(
            command,
            super::pending::PendingRequest::Import {
                display,
                edit_epoch: self.document.edit_epoch,
            },
        );
        Task::none()
    }

    fn clear_import_file_selection(&mut self) {
        self.import.generation = self.import.generation.wrapping_add(1);
        self.import.loading = false;
        self.import.import_file_path = None;
        self.import.import_file_display.clear();
        self.import.import_file_format = None;
        self.import.import_target_options.clear();
        self.import.import_target_input.clear();
        self.import.import_target_dropdown.set_open(false);
    }

    fn choose_import_file(&self) -> Task<Message> {
        let dialog = rfd::FileDialog::new()
            .add_filter("KR580 file", &["txt", "xlsx"])
            .add_filter("KR580 txt file", &["txt"])
            .add_filter("KR580 spreadsheet file", &["xlsx"]);
        file_dialog::run(
            self.dialog_parent(None),
            dialog,
            rfd::FileDialog::pick_file,
            Message::ImportFileSelected,
        )
    }

    fn toggle_import_target_dropdown(&mut self) {
        if self.import.import_target_options.is_empty() {
            return;
        }
        self.import
            .import_target_dropdown
            .set_open(!self.import.import_target_dropdown.is_open());
        self.import.import_target_dropdown.set_highlight(
            if self.import.import_target_dropdown.is_open() {
                self.import
                    .import_target_options
                    .iter()
                    .position(|option| option == &self.import.import_target_input)
                    .or(Some(0))
            } else {
                None
            },
        );
        self.import.import_modal_focus = ImportModalFocus::Target;
    }

    fn select_import_target(&mut self, value: String) {
        self.import.import_target_input = value;
        self.import.import_target_dropdown.set_open(false);
        self.import.import_modal_focus = ImportModalFocus::Target;
    }

    fn cycle_import_modal_focus(&mut self, backward: bool) {
        let mut next = self.import.import_modal_focus;
        for _ in 0..4 {
            next = if backward {
                next.previous()
            } else {
                next.next()
            };
            let unavailable_target =
                self.import.import_target_options.is_empty() && next == ImportModalFocus::Target;
            let unavailable_confirm = (self.import.import_file_path.is_none()
                || self.import.loading
                || self.import.import_error.is_some())
                && next == ImportModalFocus::Confirm;
            if !unavailable_target && !unavailable_confirm {
                break;
            }
        }
        self.import.import_modal_focus = next;
    }

    fn submit_import_modal_focus(&mut self) -> Task<Message> {
        match self.import.import_modal_focus {
            ImportModalFocus::Browse => self.choose_import_file(),
            ImportModalFocus::Target => {
                self.toggle_import_target_dropdown();
                Task::none()
            }
            ImportModalFocus::Cancel => {
                self.close_import_modal();
                Task::none()
            }
            ImportModalFocus::Confirm => self.confirm_import(),
            ImportModalFocus::None => Task::none(),
        }
    }

    fn move_import_target_highlight(&mut self, direction: i32) {
        let len = self.import.import_target_options.len();
        if len == 0 {
            self.import.import_target_dropdown.set_highlight(None);
            return;
        }
        let current = self.import.import_target_dropdown.highlight().unwrap_or(0) as i32;
        let next = current - direction;
        if next < 0 || next >= len as i32 {
            return;
        }
        self.import
            .import_target_dropdown
            .set_highlight(Some(next as usize));
    }

    fn submit_import_target_dropdown(&mut self) {
        let Some(index) = self.import.import_target_dropdown.highlight() else {
            self.import.import_target_dropdown.set_open(false);
            return;
        };
        if let Some(value) = self.import.import_target_options.get(index).cloned() {
            self.select_import_target(value);
        }
    }
}
