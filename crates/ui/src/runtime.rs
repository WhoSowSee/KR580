use crate::app::{
    DesktopApp, MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID,
    MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS, MEMORY_VALUE_INPUT_ID,
    Message, REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID, parse_register_name,
    register_name,
};
use iced::Task;
use iced::widget::operation;
use k580_app::{AppCommand, AppEvent, AppSnapshot};
use k580_core::RegisterName;

impl DesktopApp {
    pub(crate) fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
        }
        self.pull_events();
    }

    pub(crate) fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            match event {
                AppEvent::StateChanged(snapshot) => self.apply_snapshot(*snapshot),
                AppEvent::InstructionBoundaryReached(outcome) => {
                    self.status = format!("{} at {:04X}", outcome.mnemonic, outcome.pc_before)
                }
                AppEvent::TactAdvanced(outcome) => {
                    self.status =
                        format!("Tact {} cycle {}", outcome.tact_phase, outcome.cycle_count)
                }
                AppEvent::PortRead { port, value } => {
                    self.status = format!("IN {port:02X} -> {value:02X}")
                }
                AppEvent::PortWritten { port, value } => {
                    self.status = format!("OUT {port:02X} <- {value:02X}")
                }
                AppEvent::HaltStateChanged(_) => self.status = "CPU halted".to_owned(),
                AppEvent::ErrorRaised(error) => self.status = error.to_string(),
                AppEvent::Stopped => self.status = "Stopped".to_owned(),
            }
        }
    }

    fn apply_snapshot(&mut self, snapshot: AppSnapshot) {
        let register_value_follows_snapshot = parse_register_name(&self.register_name_input)
            == Some(self.selected_register)
            && self.register_value_input
                == format!(
                    "{:02X}",
                    self.snapshot.cpu.registers.get(self.selected_register)
                );
        let memory_address = parse_hex_u16(&self.memory_address_input).ok();
        let old_memory_value =
            memory_address.map(|address| format!("{:02X}", self.snapshot.cpu.memory.read(address)));
        let memory_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_value_input == *value);
        let inline_value_follows_snapshot = old_memory_value
            .as_ref()
            .is_some_and(|value| self.memory_inline_value_input == *value);

        self.snapshot = snapshot;

        if register_value_follows_snapshot {
            self.register_value_input = format!(
                "{:02X}",
                self.snapshot.cpu.registers.get(self.selected_register)
            );
        }

        if let Some(address) = memory_address {
            let value = format!("{:02X}", self.snapshot.cpu.memory.read(address));
            if memory_value_follows_snapshot {
                self.memory_value_input = value.clone();
            }
            if inline_value_follows_snapshot {
                self.memory_inline_value_input = value;
            }
        }
    }

    pub(crate) fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
            .pick_file()
        {
            self.dispatch(AppCommand::LoadSnapshot(path));
        }
    }

    pub(crate) fn save_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
            .save_file()
        {
            self.dispatch(AppCommand::SaveSnapshot(path));
        }
    }

    pub(crate) fn export_txt(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportTxt(path));
        }
    }

    pub(crate) fn export_xlsx(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Spreadsheet", &["xlsx"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportXlsx(path));
        }
    }

    pub(crate) fn export_docx(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Document", &["docx"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportDocx(path));
        }
    }

    pub(crate) fn select_register(&mut self, register: RegisterName) {
        self.selected_register = register;
        self.register_name_input = register_name(register).to_owned();
        self.register_value_input = format!("{:02X}", self.snapshot.cpu.registers.get(register));
    }

    pub(crate) fn change_register_name(&mut self, value: String) {
        let Some(value) = bounded_register_input(&value) else {
            return;
        };

        self.register_name_input = value;
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
            self.register_value_input =
                format!("{:02X}", self.snapshot.cpu.registers.get(register));
        }
    }

    pub(crate) fn change_register_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            self.register_value_input = value;
        }
    }

    pub(crate) fn step_register(&mut self, delta: i32) {
        let index = register_index(self.selected_register);
        let len = REGISTER_ORDER.len() as i32;
        let next = (index as i32 + delta).rem_euclid(len) as usize;
        self.select_register(REGISTER_ORDER[next]);
    }

    pub(crate) fn apply_register(&mut self) {
        if let Some(register) = parse_register_name(&self.register_name_input) {
            self.selected_register = register;
        } else {
            self.register_name_input = register_name(self.selected_register).to_owned();
        }

        match parse_hex_u8(&self.register_value_input) {
            Ok(value) => self.dispatch(AppCommand::SetRegister(self.selected_register, value)),
            Err(error) => self.status = error,
        }
    }

    pub(crate) fn select_memory(&mut self, address: u16) {
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.set_memory_address(address);
    }

    pub(crate) fn step_memory_address(&mut self, delta: i32) -> Task<Message> {
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = if delta.is_negative() {
            address.saturating_sub((-delta) as u16)
        } else {
            address.saturating_add(delta as u16)
        };
        self.select_memory(next);

        let row_height = MEMORY_ROW_HEIGHT;
        let viewport = self.memory_viewport_height;
        let scroll = self.memory_scroll_offset;
        if viewport <= 0.0 {
            // Viewport size unknown yet (no MemoryScrolled has fired). Skip scrolling
            // and leave the highlight where it is; iced will report viewport on the
            // first scroll event.
            return Task::none();
        }

        let row_top = next as f32 * row_height;
        let row_bottom = row_top + row_height;
        let view_top = scroll;
        let view_bottom = scroll + viewport;

        let target_offset = if row_top < view_top {
            row_top
        } else if row_bottom > view_bottom {
            (row_bottom - viewport).max(0.0)
        } else {
            return Task::none();
        };

        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    pub(crate) fn scroll_memory(&mut self, offset: f32) {
        self.memory_scroll_offset = offset.max(0.0);
        self.memory_scroll_first_row = (self.memory_scroll_offset / MEMORY_ROW_HEIGHT)
            .floor()
            .clamp(0.0, u16::MAX as f32) as u16;
    }

    pub(crate) fn change_memory_address(&mut self, value: String) {
        let Some(value) = bounded_hex_input(&value, 4) else {
            return;
        };

        // The user is editing the address inline; the previous Ctrl+Enter
        // search context is no longer relevant.
        self.memory_search_pattern = None;
        self.memory_address_input = value;
        if let Ok(address) = parse_hex_u16(&self.memory_address_input) {
            self.refresh_memory_value(address);
        }
    }

    pub(crate) fn change_memory_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            self.memory_value_input = value;
        }
    }

    pub(crate) fn change_inline_memory_value(&mut self, address: u16, value: String) {
        let Some(value) = bounded_hex_input(&value, 2) else {
            return;
        };

        self.memory_address_input = format!("{address:04X}");
        self.memory_inline_value_input = value;
    }

    pub(crate) fn apply_inline_memory_value(&mut self, address: u16) {
        match parse_hex_u8(&self.memory_inline_value_input) {
            Ok(value) => {
                self.memory_address_input = format!("{address:04X}");
                self.memory_value_input = format!("{value:02X}");
                self.memory_inline_value_input = self.memory_value_input.clone();
                self.dispatch(AppCommand::SetMemory(address, value));
            }
            Err(error) => self.status = error,
        }
    }

    pub(crate) fn toggle_opcode_dropdown(&mut self, address: u16) {
        if self.opcode_dropdown_address == Some(address) {
            self.opcode_dropdown_address = None;
            self.opcode_search_input.clear();
            return;
        }

        self.set_memory_address(address);
        self.opcode_dropdown_address = Some(address);
    }

    pub(crate) fn select_opcode(&mut self, address: u16, value: u8) {
        self.memory_address_input = format!("{address:04X}");
        self.memory_value_input = format!("{value:02X}");
        self.memory_inline_value_input = self.memory_value_input.clone();
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.dispatch(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn hide_opcode_dropdown(&mut self) {
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
    }

    pub(crate) fn apply_memory(&mut self) -> Task<Message> {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => {
                self.memory_inline_value_input = format!("{value:02X}");
                let target_offset = address as f32 * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                self.dispatch(AppCommand::SetMemory(address, value));
                scroll_memory_to(target_offset)
            }
            (Err(error), _) | (_, Err(error)) => {
                self.status = error;
                Task::none()
            }
        }
    }

    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        match parse_hex_u16(&self.memory_address_input) {
            Ok(address) => {
                self.refresh_memory_value(address);
                let target_offset = address as f32 * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                scroll_memory_to(target_offset)
            }
            Err(error) => {
                self.status = error;
                Task::none()
            }
        }
    }

    /// Walks the address space starting just after the currently selected
    /// cell and stops on the first address whose 4-digit hex form contains
    /// the cached search pattern. The pattern is captured from the address
    /// input on the very first invocation; subsequent calls reuse it so the
    /// user can iterate through every match (because each successful match
    /// rewrites the address input with a full 4-digit hex code, which would
    /// otherwise become the next search pattern). The search wraps around
    /// the 64 KiB window and always advances by at least one address.
    pub(crate) fn find_next_memory_address(&mut self) -> Task<Message> {
        if self.memory_search_pattern.is_none() {
            let pattern = self.memory_address_input.trim().to_ascii_uppercase();
            if pattern.is_empty() {
                self.status = "Enter a hex pattern to search for".to_owned();
                return Task::none();
            }
            self.memory_search_pattern = Some(pattern);
        }

        let pattern = match self.memory_search_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => pattern.to_owned(),
            _ => {
                self.status = "Enter a hex pattern to search for".to_owned();
                return Task::none();
            }
        };

        let start = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let total = MEMORY_ADDRESS_COUNT as u32;

        let mut next_match = None;
        for step in 1..=total {
            let candidate = ((start as u32 + step) % total) as u16;
            if format!("{candidate:04X}").contains(&pattern) {
                next_match = Some(candidate);
                break;
            }
        }

        match next_match {
            Some(address) => {
                self.memory_address_input = format!("{address:04X}");
                self.refresh_memory_value(address);
                self.status = format!("Found pattern {pattern} at {address:04X}");
                let target_offset = address as f32 * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                scroll_memory_to(target_offset)
            }
            None => {
                self.status = format!("No addresses match {pattern}");
                Task::none()
            }
        }
    }

    /// Resolves Tab/Shift+Tab inside one of the panels. Each focus group is a
    /// closed cycle so the user cannot accidentally tab from the memory
    /// editor into the register editor or into the inline list.
    pub(crate) fn cycle_focus(
        &mut self,
        focused: iced::widget::Id,
        backward: bool,
    ) -> Task<Message> {
        // Two-element rings: the panels each have exactly two inputs, so Tab
        // and Shift+Tab both simply swap the pair. That matches the user's
        // expectation of "go to the other field" without surprises.
        if focused == iced::widget::Id::new(MEMORY_ADDRESS_INPUT_ID) {
            self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            return operation::focus(MEMORY_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_VALUE_INPUT_ID) {
            self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            return operation::focus(MEMORY_ADDRESS_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_NAME_INPUT_ID) {
            self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            return operation::focus(REGISTER_VALUE_INPUT_ID);
        }
        if focused == iced::widget::Id::new(REGISTER_VALUE_INPUT_ID) {
            self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            return operation::focus(REGISTER_NAME_INPUT_ID);
        }
        if focused == iced::widget::Id::new(MEMORY_INLINE_INPUT_ID) {
            // The inline editor lives on whatever address is currently
            // selected. Tab moves the selection to the next/previous
            // address; the same id is then rendered for the new row, so
            // refocusing it keeps the user typing without grabbing the
            // mouse. Reuse `step_memory_address` to keep highlight, scroll,
            // and search-pattern bookkeeping consistent with arrow keys.
            self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            let step = if backward { -1 } else { 1 };
            let scroll_task = self.step_memory_address(step);
            return scroll_task.chain(operation::focus(MEMORY_INLINE_INPUT_ID));
        }
        Task::none()
    }

    fn set_memory_address(&mut self, address: u16) {
        self.memory_address_input = format!("{address:04X}");
        self.refresh_memory_value(address);
    }

    fn refresh_memory_value(&mut self, address: u16) {
        self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(address));
        self.memory_inline_value_input = self.memory_value_input.clone();
    }
}

fn scroll_memory_to(offset: f32) -> Task<Message> {
    operation::scroll_to(
        MEMORY_SCROLL_ID,
        operation::AbsoluteOffset {
            x: None,
            y: Some(offset),
        },
    )
}

fn parse_hex_u8(input: &str) -> Result<u8, String> {
    u8::from_str_radix(hex_digits(input), 16).map_err(|_| format!("Invalid byte hex: {input}"))
}

fn parse_hex_u16(input: &str) -> Result<u16, String> {
    u16::from_str_radix(hex_digits(input), 16).map_err(|_| format!("Invalid address hex: {input}"))
}

fn hex_digits(input: &str) -> &str {
    input
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
}

fn bounded_hex_input(input: &str, max_len: usize) -> Option<String> {
    let input = hex_digits(input);
    if input.len() > max_len || !input.chars().all(|char| char.is_ascii_hexdigit()) {
        return None;
    }

    Some(input.to_ascii_uppercase())
}

fn bounded_register_input(input: &str) -> Option<String> {
    let input = input.trim();
    if input.len() > 1 {
        return None;
    }

    Some(input.to_ascii_uppercase())
}

fn register_index(register: RegisterName) -> usize {
    REGISTER_ORDER
        .iter()
        .position(|candidate| *candidate == register)
        .unwrap_or(0)
}
