use super::export_modal_state::{hex4_input, parse_hex_u16_or};
use super::{
    DesktopApp, ExportFlagSelection, ExportMemoryColumns, ExportModalFocus,
    ExportRegisterSelection, ExportTab, Message,
};
use crate::persistence::ExportOptions;
use iced::Task;
use iced::advanced::widget::{Id, operate};

impl DesktopApp {
    pub(crate) fn open_export_modal(&mut self) {
        self.export_modal_open = true;
        self.export_tab = ExportTab::Xlsx;
        self.export_modal_focus = ExportModalFocus::TabXlsx;
        self.ensure_export_targets();
        self.export_target_dropdown_open = false;
        self.export_target_highlight = None;
        self.export_memory_start_input = "0000".to_owned();
        self.export_memory_end_input = "FFFF".to_owned();
        self.export_memory_columns = ExportMemoryColumns::default();
        self.export_registers = ExportRegisterSelection::default();
        self.export_flags = ExportFlagSelection::default();
        self.open_menu = None;
        self.hide_opcode_dropdown();
    }

    pub(crate) fn close_export_modal(&mut self) {
        self.export_modal_open = false;
        self.export_modal_focus = ExportModalFocus::TabXlsx;
        self.export_target_dropdown_open = false;
        self.export_target_highlight = None;
    }

    pub(crate) fn route_export_modal_message(
        &mut self,
        message: &Message,
    ) -> Option<Task<Message>> {
        if !self.export_modal_open {
            return None;
        }

        match message {
            Message::Tick | Message::CursorMoved(_) | Message::ModifiersChanged(_) => None,
            Message::ExportTabSelected(tab) => {
                self.select_export_tab(*tab);
                Some(Task::none())
            }
            Message::ExportTargetChanged(value) => {
                self.set_export_target_input(value.clone());
                self.export_modal_focus = ExportModalFocus::Page;
                self.export_target_highlight = None;
                Some(Task::none())
            }
            Message::ExportTargetDropdownToggled => {
                self.toggle_export_target_dropdown();
                self.export_modal_focus = ExportModalFocus::TargetDropdown;
                Some(Task::none())
            }
            Message::ExportTargetSelected(value) => {
                self.select_export_target(value.clone());
                Some(Task::none())
            }
            Message::ExportTargetAdd => {
                self.add_export_target();
                Some(Task::none())
            }
            Message::ExportTargetDelete => {
                self.delete_export_target();
                Some(Task::none())
            }
            Message::ExportMemoryStartChanged(value) => {
                self.export_memory_start_input = hex4_input(value);
                self.sync_current_export_target_settings();
                self.export_modal_focus = ExportModalFocus::MemoryStart;
                Some(Task::none())
            }
            Message::ExportMemoryEndChanged(value) => {
                self.export_memory_end_input = hex4_input(value);
                self.sync_current_export_target_settings();
                self.export_modal_focus = ExportModalFocus::MemoryEnd;
                Some(Task::none())
            }
            Message::ToggleExportMemoryColumn(column) => {
                self.export_memory_columns.toggle(*column);
                self.sync_current_export_target_settings();
                self.export_modal_focus = ExportModalFocus::for_column(*column);
                Some(Task::none())
            }
            Message::ToggleExportRegister(register) => {
                self.export_registers.toggle(*register);
                self.sync_current_export_target_settings();
                self.export_modal_focus = ExportModalFocus::for_register(*register);
                Some(Task::none())
            }
            Message::ToggleExportFlag(flag) => {
                self.export_flags.toggle(*flag);
                self.sync_current_export_target_settings();
                self.export_modal_focus = ExportModalFocus::for_flag(*flag);
                Some(Task::none())
            }
            Message::ConfirmExport => Some(self.confirm_export()),
            Message::CancelExport => {
                self.close_export_modal();
                Some(Task::none())
            }
            Message::EscPressed => {
                if self.clear_export_value_focus() {
                    Some(blur_export_value_focus())
                } else {
                    self.close_export_modal();
                    Some(Task::none())
                }
            }
            Message::MousePressedIgnored => Some(self.clear_export_value_focus_task()),
            Message::FocusCycle { backward } => {
                self.cycle_export_modal_focus(*backward);
                Some(Task::none())
            }
            Message::ArrowKey(direction) if self.export_target_dropdown_open => {
                self.move_export_target_highlight(*direction);
                Some(Task::none())
            }
            Message::EnterPressed if self.export_target_dropdown_open => {
                self.submit_export_target_dropdown();
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.submit_export_modal_focus()),
            _ => Some(Task::none()),
        }
    }

    pub(crate) fn select_export_tab(&mut self, tab: ExportTab) {
        self.sync_current_export_target_settings();
        self.export_tab = tab;
        self.export_target_dropdown_open = false;
        self.export_target_highlight = None;
        self.load_current_export_target_settings();
        self.export_modal_focus = match tab {
            ExportTab::Xlsx => ExportModalFocus::TabXlsx,
            ExportTab::Text => ExportModalFocus::TabText,
        };
    }

    pub(crate) fn cycle_export_modal_focus(&mut self, backward: bool) {
        self.export_modal_focus = if backward {
            self.export_modal_focus.previous_for_tab(self.export_tab)
        } else {
            self.export_modal_focus.next_for_tab(self.export_tab)
        };
    }

    pub(crate) fn submit_export_modal_focus(&mut self) -> Task<Message> {
        if let Some(tab) = self.export_modal_focus.tab() {
            self.select_export_tab(tab);
            return Task::none();
        }
        match self.export_modal_focus {
            ExportModalFocus::TargetDropdown => {
                self.toggle_export_target_dropdown();
                return Task::none();
            }
            ExportModalFocus::TargetAdd => {
                self.add_export_target();
                return Task::none();
            }
            ExportModalFocus::TargetDelete => {
                self.delete_export_target();
                return Task::none();
            }
            _ => {}
        }
        if let Some(column) = self.export_modal_focus.memory_column() {
            self.export_memory_columns.toggle(column);
            self.sync_current_export_target_settings();
            return Task::none();
        }
        if let Some(register) = self.export_modal_focus.register() {
            self.export_registers.toggle(register);
            self.sync_current_export_target_settings();
            return Task::none();
        }
        if let Some(flag) = self.export_modal_focus.flag() {
            self.export_flags.toggle(flag);
            self.sync_current_export_target_settings();
            return Task::none();
        }
        match self.export_modal_focus {
            ExportModalFocus::Cancel => {
                self.close_export_modal();
                Task::none()
            }
            ExportModalFocus::Confirm => self.confirm_export(),
            _ => Task::none(),
        }
    }

    pub(crate) fn confirm_export(&mut self) -> Task<Message> {
        self.sync_current_export_target_settings();
        let tab = self.export_tab;
        let options = self.export_options();
        self.close_export_modal();
        self.export_selected_file(tab, options);
        Task::none()
    }

    pub(crate) fn export_options(&self) -> ExportOptions {
        let mut start = parse_hex_u16_or(&self.export_memory_start_input, 0);
        let mut end = parse_hex_u16_or(&self.export_memory_end_input, u16::MAX);
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        let xlsx_pages = if self.export_tab == ExportTab::Xlsx {
            self.export_xlsx_page_options()
        } else {
            Vec::new()
        };
        let text_sections = if self.export_tab == ExportTab::Text {
            self.export_text_section_options()
        } else {
            Vec::new()
        };
        ExportOptions {
            page_name: self.export_target_input().trim().to_owned(),
            memory_start: start,
            memory_end: end,
            include_memory_address: self.export_memory_columns.address,
            include_memory_value: self.export_memory_columns.value,
            include_memory_command: self.export_memory_columns.command,
            include_comment_column: self.export_memory_columns.comment,
            registers: self.export_registers.selected(),
            flags: self.export_flags.selected(),
            xlsx_pages,
            text_sections,
        }
    }

    fn clear_export_value_focus_task(&mut self) -> Task<Message> {
        if self.clear_export_value_focus() {
            blur_export_value_focus()
        } else {
            Task::none()
        }
    }

    fn clear_export_value_focus(&mut self) -> bool {
        let should_clear =
            self.export_target_dropdown_open || self.export_modal_focus.clears_on_escape();
        if !should_clear {
            return false;
        }
        self.export_target_dropdown_open = false;
        self.export_target_highlight = None;
        self.focused_input = None;
        self.export_modal_focus = ExportModalFocus::None;
        true
    }
}

fn blur_export_value_focus() -> Task<Message> {
    operate(crate::runtime::unfocus_except(Id::new("export-modal-blur"))).discard()
}
