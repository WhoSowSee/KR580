//! Direct text exporter.
//!
//! Per `prompt/04_file_formats.md` and `prompt/06_reimplementation_notes.md`:
//! exports must be derived from core state, never from UI controls.
//!
//! Only the `.txt` exporter is implemented in the first pass. `.xlsx` and
//! `.docx` writers can be added later without changing this contract; the
//! prompt requires that they exist eventually but does not require them in
//! the first implementation iteration. The view-model assembly is here so
//! all three exporters share the same source of truth.

use crate::error::ExportError;
use kr580_core::Cpu8080State;
use std::io::Write;
use std::path::Path;

/// Snapshot of CPU state suitable for exporting. Built from the core state.
#[derive(Debug, Clone)]
pub struct ExportView {
    /// Register dump as `(name, byte)` rows.
    pub registers: Vec<(&'static str, u8)>,
    /// Flags as `(name, value)`.
    pub flags: Vec<(&'static str, bool)>,
    /// PC.
    pub pc: u16,
    /// SP.
    pub sp: u16,
    /// Total cycle count since reset.
    pub cycle_count: u64,
    /// First N bytes of RAM, in 16-byte rows: `(addr, bytes)`.
    pub memory_preview: Vec<(u16, Vec<u8>)>,
}

impl ExportView {
    /// Build a view of the current core state. `mem_preview_bytes` controls
    /// how much RAM is exported (rounded up to a 16-byte boundary).
    pub fn from_state(state: &Cpu8080State, mem_preview_bytes: usize) -> Self {
        let registers = vec![
            ("A", state.a),
            ("B", state.b),
            ("C", state.c),
            ("D", state.d),
            ("E", state.e),
            ("H", state.h),
            ("L", state.l),
        ];
        let flags = vec![
            ("S", state.flags.s),
            ("Z", state.flags.z),
            ("AC", state.flags.ac),
            ("P", state.flags.p),
            ("CY", state.flags.cy),
        ];
        let preview_len = mem_preview_bytes.min(0x10000);
        let mut memory_preview = Vec::with_capacity(preview_len / 16);
        let ram = state.ram.as_slice();
        let mut addr = 0usize;
        while addr < preview_len {
            let end = (addr + 16).min(preview_len);
            memory_preview.push((addr as u16, ram[addr..end].to_vec()));
            addr = end;
        }
        Self {
            registers,
            flags,
            pc: state.pc,
            sp: state.sp,
            cycle_count: state.cycle_count,
            memory_preview,
        }
    }
}

/// Plain text exporter.
pub struct TxtExporter;

impl TxtExporter {
    /// Render `view` to a UTF-8 string with `\n` line endings.
    pub fn render(view: &ExportView) -> String {
        let mut s = String::new();
        s.push_str("KR580 / Intel 8080 state export\n");
        s.push_str("================================\n\n");
        s.push_str(&format!("PC = {:04X}\n", view.pc));
        s.push_str(&format!("SP = {:04X}\n", view.sp));
        s.push_str(&format!("Cycles = {}\n\n", view.cycle_count));
        s.push_str("Registers:\n");
        for (name, value) in &view.registers {
            s.push_str(&format!("  {name:<3} = {value:02X}\n"));
        }
        s.push('\n');
        s.push_str("Flags:\n");
        for (name, value) in &view.flags {
            s.push_str(&format!("  {name:<3} = {}\n", *value as u8));
        }
        s.push('\n');
        if !view.memory_preview.is_empty() {
            s.push_str("Memory:\n");
            for (addr, row) in &view.memory_preview {
                s.push_str(&format!("  {addr:04X}: "));
                for b in row {
                    s.push_str(&format!("{b:02X} "));
                }
                s.push('\n');
            }
        }
        s
    }

    /// Write rendered text to `path`.
    pub fn write(view: &ExportView, path: impl AsRef<Path>) -> Result<(), ExportError> {
        let text = Self::render(view);
        let mut f = std::fs::File::create(path).map_err(ExportError::Io)?;
        f.write_all(text.as_bytes()).map_err(ExportError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_register_lines() {
        let mut s = Cpu8080State::new();
        s.a = 0xAB;
        s.pc = 0x1234;
        let view = ExportView::from_state(&s, 32);
        let text = TxtExporter::render(&view);
        assert!(text.contains("PC = 1234"));
        assert!(text.contains("A   = AB"));
        assert!(text.contains("Memory:"));
    }
}
