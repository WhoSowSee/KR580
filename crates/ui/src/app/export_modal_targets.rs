use super::export_modal_state::parse_hex_u16_or;
use super::{DesktopApp, ExportModalFocus, ExportTab, ExportTargetSettings};
use crate::i18n::Key;
use crate::persistence::{ExportTextSection, ExportXlsxPage};

impl DesktopApp {
    pub(crate) fn export_target_input(&self) -> &str {
        match self.export_tab {
            ExportTab::Xlsx => &self.export_xlsx_page_input,
            ExportTab::Text => &self.export_text_section_input,
        }
    }

    pub(crate) fn export_target_options(&self) -> &[String] {
        match self.export_tab {
            ExportTab::Xlsx => &self.export_xlsx_pages,
            ExportTab::Text => &self.export_text_sections,
        }
    }

    pub(crate) fn ensure_export_targets(&mut self) {
        if self.export_xlsx_pages.is_empty() {
            self.export_xlsx_pages
                .push(self.lang.t(Key::ExportPageDefault).to_owned());
        }
        if self.export_text_sections.is_empty() {
            self.export_text_sections
                .push(self.lang.t(Key::ExportSectionDefault).to_owned());
        }
        self.ensure_xlsx_page_settings();
        self.ensure_text_section_settings();
        if self.export_xlsx_page_input.trim().is_empty() {
            self.export_xlsx_page_input = self.export_xlsx_pages[0].clone();
        }
        if self.export_text_section_input.trim().is_empty() {
            self.export_text_section_input = self.export_text_sections[0].clone();
        }
    }

    pub(crate) fn toggle_export_target_dropdown(&mut self) {
        self.export_target_dropdown_open = !self.export_target_dropdown_open;
        self.export_target_highlight = if self.export_target_dropdown_open {
            let input = self.export_target_input();
            self.export_target_options()
                .iter()
                .position(|option| option == input)
                .or(Some(0))
        } else {
            None
        };
    }

    pub(crate) fn select_export_target(&mut self, value: String) {
        self.sync_current_export_target_settings();
        *self.export_target_input_mut() = value;
        self.export_target_dropdown_open = false;
        self.export_target_highlight = None;
        self.load_current_export_target_settings();
        self.export_modal_focus = ExportModalFocus::Page;
    }

    pub(crate) fn set_export_target_input(&mut self, value: String) {
        *self.export_target_input_mut() = value;
    }

    pub(crate) fn add_export_target(&mut self) {
        self.sync_current_export_target_settings();
        let dropdown_was_open = self.export_target_dropdown_open;
        let typed = self.export_target_input().trim().to_owned();
        let value = if typed.is_empty() || self.export_target_options().iter().any(|v| v == &typed)
        {
            self.next_export_target_name()
        } else {
            typed
        };
        let highlight = {
            let options = self.export_target_options_mut();
            options.push(value.clone());
            options.len() - 1
        };
        let settings = self.current_export_target_settings();
        match self.export_tab {
            ExportTab::Xlsx => self.export_xlsx_page_settings.push(settings),
            ExportTab::Text => self.export_text_section_settings.push(settings),
        }
        *self.export_target_input_mut() = value;
        self.export_target_dropdown_open = dropdown_was_open;
        self.export_target_highlight = dropdown_was_open.then_some(highlight);
        self.export_modal_focus = ExportModalFocus::after_target_action(dropdown_was_open);
    }

    pub(crate) fn delete_export_target(&mut self) {
        let dropdown_was_open = self.export_target_dropdown_open;
        let value = self.export_target_input().trim().to_owned();
        let fallback = match self.export_tab {
            ExportTab::Xlsx => self.lang.t(Key::ExportPageDefault).to_owned(),
            ExportTab::Text => self.lang.t(Key::ExportSectionDefault).to_owned(),
        };
        let removed_at = {
            let options = self.export_target_options_mut();
            let removed_at = options.iter().position(|option| option == &value);
            if let Some(index) = removed_at {
                options.remove(index);
            }
            if options.is_empty() {
                options.push(fallback);
            }
            removed_at
        };
        match self.export_tab {
            ExportTab::Xlsx => {
                if let Some(index) = removed_at {
                    self.export_xlsx_page_settings.remove(index);
                }
                self.ensure_xlsx_page_settings();
            }
            ExportTab::Text => {
                if let Some(index) = removed_at {
                    self.export_text_section_settings.remove(index);
                }
                self.ensure_text_section_settings();
            }
        }
        let next = removed_at
            .unwrap_or(0)
            .min(self.export_target_options().len() - 1);
        let next_value = self.export_target_options()[next].clone();
        *self.export_target_input_mut() = next_value;
        self.load_current_export_target_settings();
        self.export_target_dropdown_open = dropdown_was_open;
        self.export_target_highlight = dropdown_was_open.then_some(next);
        self.export_modal_focus = ExportModalFocus::after_target_action(dropdown_was_open);
    }

    pub(crate) fn move_export_target_highlight(&mut self, direction: i32) {
        let len = self.export_target_options().len();
        if len == 0 {
            self.export_target_highlight = None;
            return;
        }
        let current = self.export_target_highlight.unwrap_or(0) as i32;
        let next = current - direction;
        if next < 0 || next >= len as i32 {
            return;
        }
        self.export_target_highlight = Some(next as usize);
    }

    pub(crate) fn submit_export_target_dropdown(&mut self) {
        let Some(index) = self.export_target_highlight else {
            self.export_target_dropdown_open = false;
            return;
        };
        if let Some(value) = self.export_target_options().get(index).cloned() {
            self.select_export_target(value);
        }
    }

    pub(crate) fn sync_current_export_target_settings(&mut self) {
        let input = self.export_target_input().trim();
        let Some(index) = self
            .export_target_options()
            .iter()
            .position(|name| name.trim() == input)
        else {
            return;
        };
        let settings = self.current_export_target_settings();
        match self.export_tab {
            ExportTab::Xlsx => {
                self.ensure_xlsx_page_settings();
                self.export_xlsx_page_settings[index] = settings;
            }
            ExportTab::Text => {
                self.ensure_text_section_settings();
                self.export_text_section_settings[index] = settings;
            }
        }
    }

    pub(crate) fn load_current_export_target_settings(&mut self) {
        let input = self.export_target_input().trim();
        let Some(index) = self
            .export_target_options()
            .iter()
            .position(|name| name.trim() == input)
        else {
            return;
        };
        let settings = match self.export_tab {
            ExportTab::Xlsx => self
                .export_xlsx_page_settings
                .get(index)
                .cloned()
                .unwrap_or_default(),
            ExportTab::Text => self
                .export_text_section_settings
                .get(index)
                .cloned()
                .unwrap_or_default(),
        };
        self.apply_export_target_settings(settings);
    }

    pub(crate) fn export_xlsx_page_options(&self) -> Vec<ExportXlsxPage> {
        let current = self.export_xlsx_page_input.trim();
        let current_settings = self.current_export_target_settings();
        let mut pages: Vec<_> = self
            .export_xlsx_pages
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let settings = if name.trim() == current {
                    &current_settings
                } else {
                    self.export_xlsx_page_settings
                        .get(index)
                        .unwrap_or(&current_settings)
                };
                xlsx_page_from_settings(name.clone(), settings)
            })
            .collect();
        if !current.is_empty()
            && !self
                .export_xlsx_pages
                .iter()
                .any(|name| name.trim() == current)
        {
            pages.push(xlsx_page_from_settings(
                current.to_owned(),
                &current_settings,
            ));
        }
        pages
    }

    pub(crate) fn export_text_section_options(&self) -> Vec<ExportTextSection> {
        let current = self.export_text_section_input.trim();
        let current_settings = self.current_export_target_settings();
        let mut sections: Vec<_> = self
            .export_text_sections
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let settings = if name.trim() == current {
                    &current_settings
                } else {
                    self.export_text_section_settings
                        .get(index)
                        .unwrap_or(&current_settings)
                };
                text_section_from_settings(name.clone(), settings)
            })
            .collect();
        if !current.is_empty()
            && !self
                .export_text_sections
                .iter()
                .any(|name| name.trim() == current)
        {
            sections.push(text_section_from_settings(
                current.to_owned(),
                &current_settings,
            ));
        }
        sections
    }

    fn export_target_input_mut(&mut self) -> &mut String {
        match self.export_tab {
            ExportTab::Xlsx => &mut self.export_xlsx_page_input,
            ExportTab::Text => &mut self.export_text_section_input,
        }
    }

    fn export_target_options_mut(&mut self) -> &mut Vec<String> {
        match self.export_tab {
            ExportTab::Xlsx => &mut self.export_xlsx_pages,
            ExportTab::Text => &mut self.export_text_sections,
        }
    }

    fn ensure_text_section_settings(&mut self) {
        while self.export_text_section_settings.len() < self.export_text_sections.len() {
            self.export_text_section_settings
                .push(ExportTargetSettings::default());
        }
        self.export_text_section_settings
            .truncate(self.export_text_sections.len());
    }

    fn ensure_xlsx_page_settings(&mut self) {
        while self.export_xlsx_page_settings.len() < self.export_xlsx_pages.len() {
            self.export_xlsx_page_settings
                .push(ExportTargetSettings::default());
        }
        self.export_xlsx_page_settings
            .truncate(self.export_xlsx_pages.len());
    }

    fn next_export_target_name(&self) -> String {
        let base = match self.export_tab {
            ExportTab::Xlsx => self.lang.t(Key::ExportPageNameBase),
            ExportTab::Text => self.lang.t(Key::ExportSectionNameBase),
        };
        let options = self.export_target_options();
        let mut index = options.len() + 1;
        loop {
            let candidate = format!("{base} {index}");
            if !options.iter().any(|option| option == &candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    fn current_export_target_settings(&self) -> ExportTargetSettings {
        ExportTargetSettings {
            memory_start_input: self.export_memory_start_input.clone(),
            memory_end_input: self.export_memory_end_input.clone(),
            columns: self.export_memory_columns,
            registers: self.export_registers,
            flags: self.export_flags,
        }
    }

    fn apply_export_target_settings(&mut self, settings: ExportTargetSettings) {
        self.export_memory_start_input = settings.memory_start_input;
        self.export_memory_end_input = settings.memory_end_input;
        self.export_memory_columns = settings.columns;
        self.export_registers = settings.registers;
        self.export_flags = settings.flags;
    }
}

fn xlsx_page_from_settings(name: String, settings: &ExportTargetSettings) -> ExportXlsxPage {
    let mut start = parse_hex_u16_or(&settings.memory_start_input, 0);
    let mut end = parse_hex_u16_or(&settings.memory_end_input, u16::MAX);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    ExportXlsxPage {
        name,
        memory_start: start,
        memory_end: end,
        include_memory_address: settings.columns.address,
        include_memory_value: settings.columns.value,
        include_memory_command: settings.columns.command,
        include_comment_column: settings.columns.comment,
        registers: settings.registers.selected(),
        flags: settings.flags.selected(),
    }
}

fn text_section_from_settings(name: String, settings: &ExportTargetSettings) -> ExportTextSection {
    let mut start = parse_hex_u16_or(&settings.memory_start_input, 0);
    let mut end = parse_hex_u16_or(&settings.memory_end_input, u16::MAX);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    ExportTextSection {
        name,
        memory_start: start,
        memory_end: end,
        include_memory_address: settings.columns.address,
        include_memory_value: settings.columns.value,
        include_memory_command: settings.columns.command,
        registers: settings.registers.selected(),
        flags: settings.flags.selected(),
    }
}
