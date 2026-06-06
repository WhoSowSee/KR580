use super::{ExportModel, ExportOptions, Exporters, command_for, hex8, hex16};
use crate::ExportError;
use rust_xlsxwriter::{Workbook, Worksheet};
use std::path::Path;

impl Exporters {
    pub fn write_xlsx(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        Self::write_xlsx_with_options(path, model, &ExportOptions::default())
    }

    pub fn write_xlsx_with_options(
        path: impl AsRef<Path>,
        model: &ExportModel,
        options: &ExportOptions,
    ) -> Result<(), ExportError> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet
            .set_name(sheet_name(&options.page_name).as_str())
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        write_sheet(sheet, model, options)?;
        workbook
            .save(path)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))
    }

    pub fn write_xlsx_pages(
        path: impl AsRef<Path>,
        pages: &[(String, ExportModel, ExportOptions)],
    ) -> Result<(), ExportError> {
        let mut workbook = Workbook::new();
        let mut used_names = Vec::new();
        for (index, (name, model, options)) in pages.iter().enumerate() {
            let mut options = options.clone();
            options.page_name = name.clone();
            let sheet = workbook.add_worksheet();
            sheet
                .set_name(unique_sheet_name(name, index, &mut used_names).as_str())
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            write_sheet(sheet, model, &options)?;
        }
        workbook
            .save(path)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))
    }
}

fn write_sheet(
    sheet: &mut Worksheet,
    model: &ExportModel,
    options: &ExportOptions,
) -> Result<(), ExportError> {
    let mut row: u32 = 0;
    sheet
        .write_string(row, 0, "Field")
        .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    sheet
        .write_string(row, 1, "Value")
        .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    row += 1;
    for (name, value) in &model.registers {
        sheet
            .write_string(row, 0, name)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        sheet
            .write_string(row, 1, value)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        row += 1;
    }
    row += 1;
    sheet
        .write_string(row, 0, "Flag")
        .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    sheet
        .write_string(row, 1, "Set")
        .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    row += 1;
    for (name, set) in &model.flags {
        sheet
            .write_string(row, 0, name)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        sheet
            .write_string(row, 1, set.to_string())
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        row += 1;
    }
    row += 1;
    write_memory_headers(sheet, row, options)?;
    row += 1;
    for (address, value) in &model.memory {
        write_memory_row(sheet, row, *address, *value, options)?;
        row += 1;
    }
    Ok(())
}

fn write_memory_headers(
    sheet: &mut Worksheet,
    row: u32,
    options: &ExportOptions,
) -> Result<(), ExportError> {
    let mut column = 0;
    if options.include_memory_address {
        sheet
            .write_string(row, column, "Address")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_memory_value {
        sheet
            .write_string(row, column, "Value")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_memory_command {
        sheet
            .write_string(row, column, "Command")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_comment_column {
        sheet
            .write_string(row, column, "Comment")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    }
    Ok(())
}

fn write_memory_row(
    sheet: &mut Worksheet,
    row: u32,
    address: u16,
    value: u8,
    options: &ExportOptions,
) -> Result<(), ExportError> {
    let mut column = 0;
    if options.include_memory_address {
        sheet
            .write_string(row, column, hex16(address))
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_memory_value {
        sheet
            .write_string(row, column, hex8(value))
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_memory_command {
        sheet
            .write_string(row, column, command_for(value))
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        column += 1;
    }
    if options.include_comment_column {
        sheet
            .write_string(row, column, "")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
    }
    Ok(())
}

fn unique_sheet_name(name: &str, index: usize, used_names: &mut Vec<String>) -> String {
    let base = sheet_name(name);
    let mut candidate = base.clone();
    let mut suffix_index = index + 1;
    while used_names
        .iter()
        .any(|used| used.eq_ignore_ascii_case(&candidate))
    {
        let suffix = format!(" {suffix_index}");
        let base_len = 31usize.saturating_sub(suffix.chars().count());
        candidate = format!(
            "{}{}",
            base.chars().take(base_len).collect::<String>(),
            suffix
        );
        suffix_index += 1;
    }
    used_names.push(candidate.clone());
    candidate
}

fn sheet_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|ch| match ch {
            ':' | '\\' | '/' | '?' | '*' | '[' | ']' => '_',
            _ => ch,
        })
        .take(31)
        .collect();
    if cleaned.trim().is_empty() {
        "State".to_owned()
    } else {
        cleaned
    }
}
