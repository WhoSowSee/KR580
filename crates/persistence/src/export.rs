use crate::ExportError;
use k580_core::Cpu8080State;
use rust_xlsxwriter::Workbook;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportModel {
    pub registers: Vec<(String, String)>,
    pub flags: Vec<(String, bool)>,
    pub memory: Vec<(u16, u8)>,
}

pub struct Exporters;

impl ExportModel {
    /// Builds an export model from the live CPU state. Memory is emitted
    /// **sparsely**: only cells that differ from the default `0x00` are
    /// included, scanning the full 64 KiB address space. The emulator
    /// boots with a zeroed RAM, so `value != 0` is exactly the set of
    /// bytes the user has actually touched (via `SetMemory`, snapshot
    /// load, or import) — anything else is implicit and gets restored to
    /// zero on import via `ExportModel::apply_to`.
    ///
    /// This replaces an earlier dense range (`0x0000..=0x00FF`) which
    /// dumped the first 256 bytes regardless of content. The user
    /// reported the dump was illogical — most rows were `0000=00` noise
    /// that obscured the handful of actually-edited cells, and bytes
    /// past `0x00FF` were silently lost. Scanning all 64 KiB with a
    /// non-zero filter solves both problems at once.
    pub fn from_cpu(state: &Cpu8080State) -> Self {
        let registers = vec![
            ("A".to_owned(), hex8(state.registers.a)),
            ("B".to_owned(), hex8(state.registers.b)),
            ("C".to_owned(), hex8(state.registers.c)),
            ("D".to_owned(), hex8(state.registers.d)),
            ("E".to_owned(), hex8(state.registers.e)),
            ("H".to_owned(), hex8(state.registers.h)),
            ("L".to_owned(), hex8(state.registers.l)),
            ("PC".to_owned(), hex16(state.pc)),
            ("SP".to_owned(), hex16(state.sp)),
            ("cycles".to_owned(), state.cycle_count.to_string()),
        ];
        let flags = vec![
            ("S".to_owned(), state.flags.sign),
            ("Z".to_owned(), state.flags.zero),
            ("AC".to_owned(), state.flags.auxiliary_carry),
            ("P".to_owned(), state.flags.parity),
            ("CY".to_owned(), state.flags.carry),
        ];
        let memory = (0u16..=u16::MAX)
            .filter_map(|address| {
                let value = state.memory.read(address);
                (value != 0).then_some((address, value))
            })
            .collect();
        Self {
            registers,
            flags,
            memory,
        }
    }
}

impl Exporters {
    pub fn write_txt(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        std::fs::write(path, Self::to_text(model))?;
        Ok(())
    }

    pub fn write_xlsx(path: impl AsRef<Path>, model: &ExportModel) -> Result<(), ExportError> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet
            .set_name("State")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        // One sheet, three sections stacked vertically: registers, then
        // flags, then memory. Each section starts with its own header row
        // (`Field/Value`, `Flag/Set`, `Address/Value`) and is separated
        // from the next by a single blank row. The importer keys on the
        // header row strings to switch sections, mirroring how
        // `parse_txt` keys on `[Registers]` / `[Flags]` / `[Memory]`.
        // Splitting into two sheets earlier was over-engineered: every
        // user opens the file, sees one tab, scrolls. The second tab was
        // pure friction.
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
        row += 1; // blank separator
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
        row += 1; // blank separator
        sheet
            .write_string(row, 0, "Address")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        sheet
            .write_string(row, 1, "Value")
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
        row += 1;
        for (address, value) in &model.memory {
            sheet
                .write_string(row, 0, hex16(*address))
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            sheet
                .write_string(row, 1, hex8(*value))
                .map_err(|e| ExportError::Spreadsheet(e.to_string()))?;
            row += 1;
        }
        workbook
            .save(path)
            .map_err(|e| ExportError::Spreadsheet(e.to_string()))
    }

    /// Serialises the model as the human-readable TXT format. The output
    /// is three sections — `[Registers]`, `[Flags]`, `[Memory]` — each
    /// listing `key=value` pairs and separated by a blank line. No
    /// banner, no version line: the format has a single shape and the
    /// importer is the source of truth for what's parseable.
    pub fn to_text(model: &ExportModel) -> String {
        let mut out = String::new();
        out.push_str("[Registers]\n");
        for (name, value) in &model.registers {
            out.push_str(&format!("{name}={value}\n"));
        }
        out.push_str("\n[Flags]\n");
        for (name, set) in &model.flags {
            out.push_str(&format!("{name}={set}\n"));
        }
        out.push_str("\n[Memory]\n");
        for (address, value) in &model.memory {
            out.push_str(&format!("{}={}\n", hex16(*address), hex8(*value)));
        }
        out
    }
}

pub(crate) fn hex8(value: u8) -> String {
    format!("{value:02X}")
}

pub(crate) fn hex16(value: u16) -> String {
    format!("{value:04X}")
}
