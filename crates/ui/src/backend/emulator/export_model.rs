use super::Emulator;
use crate::persistence::{ExportModel, ExportOptions};

impl Emulator {
    pub(super) fn export_model(&self) -> ExportModel {
        ExportModel::from_cpu(&self.cpu)
    }

    pub(super) fn export_model_with_options(&self, options: &ExportOptions) -> ExportModel {
        ExportModel::from_cpu_with_options(&self.cpu, options)
    }

    pub(super) fn export_text_models(&self, options: &ExportOptions) -> Vec<(String, ExportModel)> {
        options
            .text_sections
            .iter()
            .map(|section| {
                (
                    section.name.clone(),
                    ExportModel::from_cpu_with_options(&self.cpu, &section.to_options()),
                )
            })
            .collect()
    }

    pub(super) fn export_xlsx_models(
        &self,
        options: &ExportOptions,
    ) -> Vec<(String, ExportModel, ExportOptions)> {
        options
            .xlsx_pages
            .iter()
            .map(|page| {
                let page_options = page.to_options();
                (
                    page.name.clone(),
                    ExportModel::from_cpu_with_options(&self.cpu, &page_options),
                    page_options,
                )
            })
            .collect()
    }
}
