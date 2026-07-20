use super::{
    DesktopApp, ImportFileFormat, ImportModalFocus, MEMORY_SCROLL_VISIBLE_TICKS, Message,
    StatusKind,
};
use crate::backend::AppCommand;
use crate::i18n::Key;
use crate::persistence::Importers;
use iced::Task;
use std::path::PathBuf;

impl DesktopApp {
    pub(crate) fn open_import_modal(&mut self) {
        self.import_modal_open = true;
        self.import_modal_focus = ImportModalFocus::Browse;
        self.import_file_path = None;
        self.import_file_display.clear();
        self.import_file_format = None;
        self.import_target_options.clear();
        self.import_target_input.clear();
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
        self.import_target_scroll_visible_ticks = 0;
        self.import_error = None;
        self.close_top_menu();
        self.hide_opcode_dropdown();
    }

    pub(crate) fn close_import_modal(&mut self) {
        self.import_modal_open = false;
        self.import_modal_focus = ImportModalFocus::Browse;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
        self.import_target_scroll_visible_ticks = 0;
    }

    pub(crate) fn route_import_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        if !self.import_modal_open {
            return None;
        }

        match message {
            Message::Tick | Message::CursorMoved(_) | Message::ModifiersChanged(_) => None,
            Message::ImportFileBrowse => {
                self.choose_import_file();
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
            Message::ImportTargetScrolled => {
                self.import_target_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
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
                self.import_target_scroll_visible_ticks = 0;
                self.import_modal_focus = ImportModalFocus::None;
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                self.cycle_import_modal_focus(*backward);
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
        self.import_file_display = path.display().to_string();
        self.import_file_format = Some(ImportFileFormat::from_path(&path));
        self.import_file_path = Some(path.clone());
        self.import_error = None;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
        self.import_target_scroll_visible_ticks = 0;

        let targets = match self.import_file_format {
            Some(ImportFileFormat::Xlsx) => Importers::xlsx_sheet_names(&path),
            Some(ImportFileFormat::Text) => Importers::txt_section_names(&path),
            None => Ok(Vec::new()),
        };

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
        let Some(path) = self.import_file_path.clone() else {
            self.import_error = Some(self.lang.t(Key::ImportChooseFileRequired).to_owned());
            self.import_modal_focus = ImportModalFocus::Browse;
            return Task::none();
        };
        let display = self.import_file_display.clone();
        let target = self.import_target_input.trim().to_owned();
        let command = match self
            .import_file_format
            .unwrap_or_else(|| ImportFileFormat::from_path(&path))
        {
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

    fn choose_import_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["txt", "xlsx"])
            .add_filter("KR580 txt file", &["txt"])
            .add_filter("KR580 spreadsheet file", &["xlsx"])
            .pick_file()
        {
            self.load_import_file(path);
        }
    }

    fn toggle_import_target_dropdown(&mut self) {
        if self.import_target_options.is_empty() {
            return;
        }
        self.import_target_dropdown_open = !self.import_target_dropdown_open;
        self.import_target_highlight = if self.import_target_dropdown_open {
            self.import_target_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            self.import_target_options
                .iter()
                .position(|option| option == &self.import_target_input)
                .or(Some(0))
        } else {
            self.import_target_scroll_visible_ticks = 0;
            None
        };
        self.import_modal_focus = ImportModalFocus::Target;
    }

    fn select_import_target(&mut self, value: String) {
        self.import_target_input = value;
        self.import_target_dropdown_open = false;
        self.import_target_highlight = None;
        self.import_target_scroll_visible_ticks = 0;
        self.import_modal_focus = ImportModalFocus::Target;
    }

    fn cycle_import_modal_focus(&mut self, backward: bool) {
        let mut next = if backward {
            self.import_modal_focus.previous()
        } else {
            self.import_modal_focus.next()
        };
        if self.import_target_options.is_empty() && next == ImportModalFocus::Target {
            next = if backward {
                ImportModalFocus::Browse
            } else {
                ImportModalFocus::Cancel
            };
        }
        self.import_modal_focus = next;
    }

    fn submit_import_modal_focus(&mut self) -> Task<Message> {
        match self.import_modal_focus {
            ImportModalFocus::Browse => {
                self.choose_import_file();
                Task::none()
            }
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
