use super::export_modal_state::parse_hex_u16_or;
use super::{DesktopApp, ExportModalFocus, ExportTab, ExportTarget, ExportTargetSettings};
use crate::i18n::Key;
use crate::persistence::{ExportTextSection, ExportXlsxPage};

impl DesktopApp {
    pub(crate) fn export_target_input(&self) -> &str {
        match self.export.export_tab {
            ExportTab::Xlsx => &self.export.export_xlsx_page_input,
            ExportTab::Text => &self.export.export_text_section_input,
        }
    }

    pub(crate) fn export_target_options(&self) -> &[ExportTarget] {
        match self.export.export_tab {
            ExportTab::Xlsx => &self.export.export_xlsx_pages,
            ExportTab::Text => &self.export.export_text_sections,
        }
    }

    pub(crate) fn ensure_export_targets(&mut self) {
        for (targets, input, key) in [
            (
                &mut self.export.export_xlsx_pages,
                &mut self.export.export_xlsx_page_input,
                Key::ExportPageDefault,
            ),
            (
                &mut self.export.export_text_sections,
                &mut self.export.export_text_section_input,
                Key::ExportSectionDefault,
            ),
        ] {
            if targets.is_empty() {
                targets.push(ExportTarget::named(self.preferences.lang.t(key).to_owned()));
            }
            if input.trim().is_empty() {
                *input = targets[0].name.clone();
            }
        }
    }

    pub(crate) fn toggle_export_target_dropdown(&mut self) {
        self.export
            .export_target_dropdown
            .set_open(!self.export.export_target_dropdown.is_open());
        self.export.export_target_dropdown.set_highlight(
            if self.export.export_target_dropdown.is_open() {
                self.export_target_options()
                    .iter()
                    .position(|target| target.name == self.export_target_input())
                    .or(Some(0))
            } else {
                None
            },
        );
    }

    pub(crate) fn select_export_target(&mut self, value: String) {
        self.sync_current_export_target_settings();
        *self.export_target_input_mut() = value;
        self.export.export_target_dropdown.set_open(false);
        self.load_current_export_target_settings();
        self.export.export_modal_focus = ExportModalFocus::Page;
    }

    pub(crate) fn set_export_target_input(&mut self, value: String) {
        *self.export_target_input_mut() = value;
    }

    pub(crate) fn add_export_target(&mut self) {
        self.sync_current_export_target_settings();
        let was_open = self.export.export_target_dropdown.is_open();
        let typed = self.export_target_input().trim().to_owned();
        let name = if typed.is_empty()
            || self
                .export_target_options()
                .iter()
                .any(|target| target.name == typed)
        {
            self.next_export_target_name()
        } else {
            typed
        };
        let target = ExportTarget {
            name: name.clone(),
            settings: self.current_export_target_settings(),
        };
        let targets = self.export_target_options_mut();
        targets.push(target);
        let highlight = targets.len() - 1;
        *self.export_target_input_mut() = name;
        self.export.export_target_dropdown.set_open(was_open);
        self.export
            .export_target_dropdown
            .set_highlight(was_open.then_some(highlight));
        self.export.export_modal_focus = ExportModalFocus::after_target_action(was_open);
    }

    pub(crate) fn delete_export_target(&mut self) {
        let was_open = self.export.export_target_dropdown.is_open();
        let name = self.export_target_input().trim().to_owned();
        let fallback = self
            .preferences
            .lang
            .t(match self.export.export_tab {
                ExportTab::Xlsx => Key::ExportPageDefault,
                ExportTab::Text => Key::ExportSectionDefault,
            })
            .to_owned();
        let targets = self.export_target_options_mut();
        let removed = targets.iter().position(|target| target.name == name);
        if let Some(index) = removed {
            targets.remove(index);
        }
        if targets.is_empty() {
            targets.push(ExportTarget::named(fallback));
        }
        let index = removed.unwrap_or(0).min(targets.len() - 1);
        let name = targets[index].name.clone();
        *self.export_target_input_mut() = name;
        self.load_current_export_target_settings();
        self.export.export_target_dropdown.set_open(was_open);
        self.export
            .export_target_dropdown
            .set_highlight(was_open.then_some(index));
        self.export.export_modal_focus = ExportModalFocus::after_target_action(was_open);
    }

    pub(crate) fn move_export_target_highlight(&mut self, direction: i32) {
        let len = self.export_target_options().len();
        if len == 0 {
            self.export.export_target_dropdown.set_highlight(None);
            return;
        }
        let next = self.export.export_target_dropdown.highlight().unwrap_or(0) as i32 - direction;
        if (0..len as i32).contains(&next) {
            self.export
                .export_target_dropdown
                .set_highlight(Some(next as usize));
        }
    }

    pub(crate) fn submit_export_target_dropdown(&mut self) {
        let Some(index) = self.export.export_target_dropdown.highlight() else {
            self.export.export_target_dropdown.set_open(false);
            return;
        };
        if let Some(target) = self.export_target_options().get(index) {
            self.select_export_target(target.name.clone());
        }
    }

    pub(crate) fn sync_current_export_target_settings(&mut self) {
        let name = self.export_target_input().trim().to_owned();
        let settings = self.current_export_target_settings();
        if let Some(target) = self
            .export_target_options_mut()
            .iter_mut()
            .find(|target| target.name.trim() == name)
        {
            target.settings = settings;
        }
    }

    pub(crate) fn load_current_export_target_settings(&mut self) {
        if let Some(target) = self
            .export_target_options()
            .iter()
            .find(|target| target.name.trim() == self.export_target_input().trim())
        {
            self.apply_export_target_settings(target.settings.clone());
        }
    }

    pub(crate) fn export_xlsx_page_options(&self) -> Vec<ExportXlsxPage> {
        let current = self.export.export_xlsx_page_input.trim();
        let settings = self.current_export_target_settings();
        let mut pages: Vec<_> = self
            .export
            .export_xlsx_pages
            .iter()
            .map(|target| {
                xlsx_page_from_settings(
                    target.name.clone(),
                    if target.name.trim() == current {
                        &settings
                    } else {
                        &target.settings
                    },
                )
            })
            .collect();
        if !current.is_empty()
            && !self
                .export
                .export_xlsx_pages
                .iter()
                .any(|target| target.name.trim() == current)
        {
            pages.push(xlsx_page_from_settings(current.to_owned(), &settings));
        }
        pages
    }

    pub(crate) fn export_text_section_options(&self) -> Vec<ExportTextSection> {
        let current = self.export.export_text_section_input.trim();
        let settings = self.current_export_target_settings();
        let mut sections: Vec<_> = self
            .export
            .export_text_sections
            .iter()
            .map(|target| {
                text_section_from_settings(
                    target.name.clone(),
                    if target.name.trim() == current {
                        &settings
                    } else {
                        &target.settings
                    },
                )
            })
            .collect();
        if !current.is_empty()
            && !self
                .export
                .export_text_sections
                .iter()
                .any(|target| target.name.trim() == current)
        {
            sections.push(text_section_from_settings(current.to_owned(), &settings));
        }
        sections
    }

    fn export_target_input_mut(&mut self) -> &mut String {
        match self.export.export_tab {
            ExportTab::Xlsx => &mut self.export.export_xlsx_page_input,
            ExportTab::Text => &mut self.export.export_text_section_input,
        }
    }

    fn export_target_options_mut(&mut self) -> &mut Vec<ExportTarget> {
        match self.export.export_tab {
            ExportTab::Xlsx => &mut self.export.export_xlsx_pages,
            ExportTab::Text => &mut self.export.export_text_sections,
        }
    }

    fn next_export_target_name(&self) -> String {
        let base = self.preferences.lang.t(match self.export.export_tab {
            ExportTab::Xlsx => Key::ExportPageNameBase,
            ExportTab::Text => Key::ExportSectionNameBase,
        });
        let mut index = self.export_target_options().len() + 1;
        loop {
            let candidate = format!("{base} {index}");
            if !self
                .export_target_options()
                .iter()
                .any(|target| target.name == candidate)
            {
                return candidate;
            }
            index += 1;
        }
    }

    fn current_export_target_settings(&self) -> ExportTargetSettings {
        ExportTargetSettings {
            memory_start_input: self.export.export_memory_start_input.clone(),
            memory_end_input: self.export.export_memory_end_input.clone(),
            columns: self.export.export_memory_columns,
            registers: self.export.export_registers,
            flags: self.export.export_flags,
        }
    }

    fn apply_export_target_settings(&mut self, settings: ExportTargetSettings) {
        self.export.export_memory_start_input = settings.memory_start_input;
        self.export.export_memory_end_input = settings.memory_end_input;
        self.export.export_memory_columns = settings.columns;
        self.export.export_registers = settings.registers;
        self.export.export_flags = settings.flags;
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
