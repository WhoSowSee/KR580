use super::{DesktopApp, ImportFileFormat, ImportModalFocus, Message, StatusKind};
use crate::backend::AppCommand;
use crate::i18n::Key;
use crate::persistence::Importers;
use crate::runtime::file_dialog;
use iced::{Event, Task, window};
use std::path::PathBuf;

impl DesktopApp {
    pub(crate) fn open_import_modal(&mut self) {
        self.close_export_modal();
        self.import_modal_open = true;
        self.import_modal_focus = ImportModalFocus::Browse;
        self.import_modal_keyboard_focus_visible = false;
        self.import_file_drag_hovered = false;
        self.clear_import_file_selection();
        self.import_error = None;
        self.close_top_menu();
        self.hide_opcode_dropdown();
        self.close_open_device_panel();
    }

    pub(crate) fn close_import_modal(&mut self) {
        self.import_modal_open = false;
        self.import_modal_focus = ImportModalFocus::Browse;
        self.import_modal_keyboard_focus_visible = false;
        self.import_file_drag_hovered = false;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
    }

    pub(super) fn handle_import_file_drag_event(&mut self, event: &Event) -> bool {
        if !self.import_modal_open {
            return false;
        }
        match event {
            Event::Window(window::Event::FileHovered(_)) => {
                self.import_file_drag_hovered = true;
                true
            }
            Event::Window(window::Event::FileDropped(path)) => {
                self.import_file_drag_hovered = false;
                self.load_import_file(path.clone());
                true
            }
            Event::Window(window::Event::FilesHoveredLeft) => {
                self.import_file_drag_hovered = false;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn route_import_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        if !self.import_modal_open {
            return None;
        }

        if !matches!(
            message,
            Message::Tick
                | Message::CursorMoved(_)
                | Message::ModifiersChanged(_)
                | Message::FocusCycle { .. }
        ) {
            self.import_modal_keyboard_focus_visible = false;
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
                self.import_target_dropdown_open = false;
                self.import_target_highlight = None;
                self.import_modal_focus = ImportModalFocus::None;
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                self.cycle_import_modal_focus(*backward);
                self.import_modal_keyboard_focus_visible = true;
                Some(Task::none())
            }
            Message::ArrowKey(direction) if self.import_target_dropdown_open => {
                self.move_import_target_highlight(*direction);
                Some(Task::none())
            }
            Message::EnterPressed if self.import_target_dropdown_open => {
                self.submit_import_target_dropdown();
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.submit_import_modal_focus()),
            _ => Some(Task::none()),
        }
    }

    pub(crate) fn load_import_file(&mut self, path: PathBuf) {
        let Some(format) = ImportFileFormat::from_path(&path) else {
            self.clear_import_file_selection();
            self.import_modal_focus = ImportModalFocus::Browse;
            self.import_error = Some(self.lang.t(Key::ErrUnsupportedImportFile).to_owned());
            return;
        };
        let targets = match format {
            ImportFileFormat::Xlsx => Importers::xlsx_sheet_names(&path),
            ImportFileFormat::Text => Importers::txt_section_names(&path),
        };
        self.import_file_display = path.display().to_string();
        self.import_file_format = Some(format);
        self.import_file_path = Some(path);
        self.import_error = None;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;

        match targets {
            Ok(targets) => {
                self.import_target_options = targets;
                self.import_target_input = self
                    .import_target_options
                    .first()
                    .cloned()
                    .unwrap_or_default();
                self.import_modal_focus = if self.import_target_options.is_empty() {
                    ImportModalFocus::Confirm
                } else {
                    ImportModalFocus::Target
                };
            }
            Err(err) => {
                self.import_target_options.clear();
                self.import_target_input.clear();
                self.import_modal_focus = ImportModalFocus::Browse;
                self.import_error = Some(crate::runtime::humanize_error::humanize(
                    &err.to_string(),
                    self.lang,
                ));
            }
        }
    }

    pub(crate) fn confirm_import(&mut self) -> Task<Message> {
        let (Some(path), Some(format)) = (self.import_file_path.clone(), self.import_file_format)
        else {
            self.import_error = Some(self.lang.t(Key::ImportChooseFileRequired).to_owned());
            self.import_modal_focus = ImportModalFocus::Browse;
            return Task::none();
        };
        let display = self.import_file_display.clone();
        let target = self.import_target_input.trim().to_owned();
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
        self.running = false;
        self.dispatch_sync(command);
        if self.error_notice.is_some() {
            return Task::none();
        }
        self.undo_stack.clear();
        self.mark_saved();
        self.set_status(StatusKind::ImportFrom { display });
        Task::none()
    }

    fn clear_import_file_selection(&mut self) {
        self.import_file_path = None;
        self.import_file_display.clear();
        self.import_file_format = None;
        self.import_target_options.clear();
        self.import_target_input.clear();
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
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
        if self.import_target_options.is_empty() {
            return;
        }
        self.import_target_dropdown_open = !self.import_target_dropdown_open;
        self.import_target_highlight = if self.import_target_dropdown_open {
            self.import_target_options
                .iter()
                .position(|option| option == &self.import_target_input)
                .or(Some(0))
        } else {
            None
        };
        self.import_modal_focus = ImportModalFocus::Target;
    }

    fn select_import_target(&mut self, value: String) {
        self.import_target_input = value;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
        self.import_modal_focus = ImportModalFocus::Target;
    }

    fn cycle_import_modal_focus(&mut self, backward: bool) {
        let mut next = self.import_modal_focus;
        for _ in 0..4 {
            next = if backward {
                next.previous()
            } else {
                next.next()
            };
            let unavailable_target =
                self.import_target_options.is_empty() && next == ImportModalFocus::Target;
            let unavailable_confirm =
                self.import_file_path.is_none() && next == ImportModalFocus::Confirm;
            if !unavailable_target && !unavailable_confirm {
                break;
            }
        }
        self.import_modal_focus = next;
    }

    fn submit_import_modal_focus(&mut self) -> Task<Message> {
        match self.import_modal_focus {
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
        let len = self.import_target_options.len();
        if len == 0 {
            self.import_target_highlight = None;
            return;
        }
        let current = self.import_target_highlight.unwrap_or(0) as i32;
        let next = current - direction;
        if next < 0 || next >= len as i32 {
            return;
        }
        self.import_target_highlight = Some(next as usize);
    }

    fn submit_import_target_dropdown(&mut self) {
        let Some(index) = self.import_target_highlight else {
            self.import_target_dropdown_open = false;
            return;
        };
        if let Some(value) = self.import_target_options.get(index).cloned() {
            self.select_import_target(value);
        }
    }
}
