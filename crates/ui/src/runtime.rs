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
            .add_filter("KR580 file", &["580"])
            .pick_file()
        {
            self.dispatch(AppCommand::LoadSnapshot(path));
        }
    }

    pub(crate) fn save_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 file", &["580"])
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

    /// Plain Enter handler for the register editor: applies the typed
    /// register value, cycles to the next/previous register, and keeps
    /// focus inside the register editor (whichever of the two fields the
    /// user was in). The memory list and the memory editor are not
    /// touched.
    pub(crate) fn apply_register_and_step(&mut self, backward: bool) -> Task<Message> {
        let stay_on_value = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        self.apply_register();
        let delta = if backward { -1 } else { 1 };
        self.step_register(delta);
        let target = if stay_on_value {
            REGISTER_VALUE_INPUT_ID
        } else {
            REGISTER_NAME_INPUT_ID
        };
        self.focused_input = Some(target);
        operation::focus(target)
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

        if self.memory_viewport_height <= 0.0 {
            // Viewport size unknown yet (no MemoryScrolled has fired). Skip
            // scrolling and leave the highlight where it is; iced will
            // report the viewport on the first scroll event.
            return Task::none();
        }

        let Some(target_offset) = self.scroll_offset_to_reveal(next) else {
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

    /// Writes the typed byte to the typed address. Does not scroll the
    /// memory list — callers that want the user moved to the new row must
    /// chain a scroll task themselves (see `apply_memory_and_jump`).
    pub(crate) fn apply_memory(&mut self) -> Task<Message> {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => {
                self.memory_inline_value_input = format!("{value:02X}");
                self.dispatch(AppCommand::SetMemory(address, value));
                Task::none()
            }
            (Err(error), _) | (_, Err(error)) => {
                self.status = error;
                Task::none()
            }
        }
    }

    /// Plain Enter handler for the memory cell editor: writes the byte,
    /// then advances/steps back the address input. The memory list is not
    /// scrolled — Alt+Enter is the explicit "jump to this row" shortcut.
    /// Focus stays on the value field so the user can keep typing the
    /// next byte without reaching for the mouse.
    pub(crate) fn apply_memory_and_step(&mut self, backward: bool) -> Task<Message> {
        let write = self.apply_memory();
        self.step_address_in_input(backward);
        self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
        write.chain(operation::focus(MEMORY_VALUE_INPUT_ID))
    }

    /// Alt+Enter handler for the memory value field: writes the byte and
    /// jumps the memory list to the same address.
    pub(crate) fn apply_memory_and_jump(&mut self) -> Task<Message> {
        let write = self.apply_memory();
        let jump = self.jump_memory_address();
        write.chain(jump)
    }

    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        match parse_hex_u16(&self.memory_address_input) {
            Ok(address) => {
                self.refresh_memory_value(address);
                // Only scroll if the target row is not already on screen,
                // so Alt+Enter on a visible address keeps the list still
                // instead of snapping the row to the top.
                if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
                    self.scroll_memory(target_offset);
                    return scroll_memory_to(target_offset);
                }
                Task::none()
            }
            Err(error) => {
                self.status = error;
                Task::none()
            }
        }
    }

    /// Returns the scroll offset that would bring the row containing
    /// `address` into the visible portion of the memory list, or `None`
    /// if the row is already on screen. Mirrors the visibility check used
    /// by `step_memory_address` for ArrowUp/Down navigation.
    fn scroll_offset_to_reveal(&self, address: u16) -> Option<f32> {
        let viewport = self.memory_viewport_height;
        if viewport <= 0.0 {
            // No layout has been measured yet — fall back to scrolling
            // unconditionally so the very first jump still lands on the
            // requested row.
            return Some(address as f32 * MEMORY_ROW_HEIGHT);
        }

        let row_top = address as f32 * MEMORY_ROW_HEIGHT;
        let row_bottom = row_top + MEMORY_ROW_HEIGHT;
        let view_top = self.memory_scroll_offset;
        let view_bottom = view_top + viewport;

        if row_top < view_top {
            Some(row_top)
        } else if row_bottom > view_bottom {
            Some((row_bottom - viewport).max(0.0))
        } else {
            None
        }
    }

    /// Steps the address shown in `memory_address_input` by one, wrapping
    /// around the 64 KiB window. Refreshes the memory value input for the
    /// new address, exits the search context, but **does not** scroll the
    /// memory list and does not touch the focus. Callers decide which
    /// input to leave focused.
    fn step_address_in_input(&mut self, backward: bool) {
        let current = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let delta = if backward { -1 } else { 1 };
        let next = ((current + delta).rem_euclid(total)) as u16;

        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        // Plain Enter exits the search context: the user is now manually
        // moving through addresses, not iterating over a pattern match.
        self.memory_search_pattern = None;
    }

    /// Plain Enter handler from the address field: step the address by one
    /// and keep the address input focused.
    pub(crate) fn advance_memory_address(&mut self, backward: bool) -> Task<Message> {
        self.step_address_in_input(backward);
        self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
        operation::focus(MEMORY_ADDRESS_INPUT_ID)
    }

    /// Walks the address space starting just after (or before) the
    /// currently selected cell and stops on the first address whose
    /// 4-digit hex form contains the cached search pattern. The pattern
    /// is captured from the address input on the very first invocation;
    /// subsequent calls reuse it so the user can iterate through every
    /// match (because each successful match rewrites the address input
    /// with a full 4-digit hex code, which would otherwise become the
    /// next search pattern). The search wraps around the 64 KiB window
    /// and always advances by at least one address in `backward`'s
    /// direction.
    pub(crate) fn find_next_memory_address_in_direction(
        &mut self,
        backward: bool,
    ) -> Task<Message> {
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

        let start = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let direction = if backward { -1 } else { 1 };

        let mut next_match = None;
        for step in 1..=total {
            let candidate = ((start + direction * step).rem_euclid(total)) as u16;
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
