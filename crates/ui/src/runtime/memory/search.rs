use crate::app::{DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_ROW_HEIGHT, Message};
use iced::Task;
use iced::widget::operation;

use crate::runtime::parse::{parse_hex_u16, scroll_memory_to};

impl DesktopApp {
    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        match parse_hex_u16(&self.memory_address_input) {
            Ok(address) => {
                self.refresh_memory_value(address);
                if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
                    self.scroll_memory(target_offset);
                    return scroll_memory_to(target_offset);
                }
                Task::none()
            }
            Err(error) => {
                self.set_status_custom(error);
                Task::none()
            }
        }
    }

    /// Jumps the memory view to a resolved 16-bit address (e.g. the
    /// target decoded from an address operand). Mirrors
    /// `jump_memory_address` but takes a ready `u16` instead of
    /// parsing the address field, and never dispatches `SetPc` –
    /// this is a view relocation, not a program-counter move.
    pub(crate) fn jump_memory_to(&mut self, address: u16) -> Task<Message> {
        self.memory_address_input = format!("{address:04X}");
        self.refresh_memory_value(address);
        if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
            self.scroll_memory(target_offset);
            return scroll_memory_to(target_offset);
        }
        Task::none()
    }

    pub(crate) fn advance_memory_address(&mut self, backward: bool) -> Task<Message> {
        self.commit_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.step_address_in_input(backward);
        self.continue_replacement(MEMORY_ADDRESS_INPUT_ID);
        self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
        operation::focus(MEMORY_ADDRESS_INPUT_ID)
    }

    pub(super) fn step_address_in_input(&mut self, backward: bool) {
        let (view_start, view_count) = self.memory_view();
        let current = (parse_hex_u16(&self.memory_address_input)
            .unwrap_or(view_start)
            .saturating_sub(view_start)) as i32;
        let total = view_count as i32;
        let delta = if backward { -1 } else { 1 };
        let next = view_start + ((current + delta).rem_euclid(total)) as u16;

        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        self.memory_search_pattern = None;
    }

    /// The pattern is cached on the first call so subsequent presses
    /// iterate matches – every match overwrites the address input
    /// with a full hex code that would otherwise become the next
    /// pattern.
    pub(crate) fn find_next_memory_address_in_direction(
        &mut self,
        backward: bool,
    ) -> Task<Message> {
        if self.memory_search_pattern.is_none() {
            let pattern = self.memory_address_input.trim().to_ascii_uppercase();
            if pattern.is_empty() {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
            self.memory_search_pattern = Some(pattern);
        }

        let pattern = match self.memory_search_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => pattern.to_owned(),
            _ => {
                self.set_status(crate::app::StatusKind::EnterHexPattern);
                return Task::none();
            }
        };

        let (view_start, view_count) = self.memory_view();
        let start = parse_hex_u16(&self.memory_address_input).unwrap_or(view_start) as i32;
        let start_idx = (start - view_start as i32).rem_euclid(view_count as i32);
        let total = view_count as i32;
        let direction = if backward { -1 } else { 1 };

        let mut next_match = None;
        for step in 1..=total {
            let candidate_idx = (start_idx + direction * step).rem_euclid(total);
            let candidate = view_start + candidate_idx as u16;
            if format!("{candidate:04X}").contains(&pattern) {
                next_match = Some(candidate);
                break;
            }
        }

        match next_match {
            Some(address) => {
                self.memory_address_input = format!("{address:04X}");
                self.refresh_memory_value(address);
                self.set_status(crate::app::StatusKind::PatternFound { pattern, address });
                let target_offset = (address.saturating_sub(view_start) as f32) * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                scroll_memory_to(target_offset)
            }
            None => {
                self.set_status(crate::app::StatusKind::NoMatchesFor { pattern });
                Task::none()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopApp;
    use crate::app::Message;
    use iced::keyboard;

    fn load_lxi_b_d16(app: &mut DesktopApp) {
        app.snapshot.cpu.memory.write(0x0000, 0x01); // LXI B,d16
        app.snapshot.cpu.memory.write(0x0001, 0x34);
        app.snapshot.cpu.memory.write(0x0002, 0x12); // -> 0x1234
    }

    #[test]
    fn alt_enter_on_low_operand_byte_relocates_view_to_target() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_lxi_b_d16(&mut app);
        app.memory_address_input = "0001".to_owned();
        app.refresh_memory_value(0x0001);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert_eq!(app.memory_address_input, "1234");
    }

    #[test]
    fn alt_enter_on_high_operand_byte_relocates_view_to_same_target() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_lxi_b_d16(&mut app);
        app.memory_address_input = "0002".to_owned();
        app.refresh_memory_value(0x0002);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert_eq!(app.memory_address_input, "1234");
    }

    #[test]
    fn alt_enter_on_opcode_byte_keeps_current_address() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_lxi_b_d16(&mut app);
        app.memory_address_input = "0000".to_owned();
        app.refresh_memory_value(0x0000);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert_eq!(app.memory_address_input, "0000");
    }

    fn load_out_port(app: &mut DesktopApp, address: u16, port: u8) {
        app.snapshot.cpu.memory.write(address, 0xD3); // OUT
        app.snapshot.cpu.memory.write(address.wrapping_add(1), port);
    }

    #[test]
    fn alt_enter_on_out_port_opens_matching_device() {
        let cases = [
            (0x0000u16, 0x00u8, "monitor"),
            (0x0002, 0x01, "floppy"),
            (0x0004, 0x02, "hdd"),
            (0x0006, 0x03, "network"),
            (0x0008, 0x04, "printer"),
        ];
        for (start, port, label) in cases {
            let (mut app, _) = DesktopApp::with_initial_path(None);
            load_out_port(&mut app, start, port);
            let operand = start.wrapping_add(1);
            app.memory_address_input = format!("{:04X}", operand);
            app.refresh_memory_value(operand);
            app.keyboard_modifiers = keyboard::Modifiers::ALT;

            let _ = app.update(Message::EnterPressed);

            match label {
                "monitor" => assert!(app.monitor_open, "monitor not opened for port 0x{port:02X}"),
                "floppy" => assert!(app.floppy_open, "floppy not opened for port 0x{port:02X}"),
                "hdd" => assert!(app.hdd_open, "hdd not opened for port 0x{port:02X}"),
                "network" => {
                    assert!(app.network_open, "network not opened for port 0x{port:02X}")
                }
                "printer" => {
                    assert!(app.printer_open, "printer not opened for port 0x{port:02X}")
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn alt_enter_on_unknown_port_does_not_open_device() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_out_port(&mut app, 0x0000, 0x7F); // unmapped port
        app.memory_address_input = "0001".to_owned();
        app.refresh_memory_value(0x0001);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert!(!app.monitor_open);
        assert!(!app.floppy_open);
        assert!(!app.hdd_open);
        assert!(!app.network_open);
        assert!(!app.printer_open);
        assert_eq!(app.memory_address_input, "0001");
    }

    #[test]
    fn alt_enter_on_out_opcode_byte_does_not_open_device() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        load_out_port(&mut app, 0x0000, 0x04);
        app.memory_address_input = "0000".to_owned();
        app.refresh_memory_value(0x0000);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert!(!app.printer_open);
    }

    #[test]
    fn alt_enter_on_data_operand_still_falls_through() {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.snapshot.cpu.memory.write(0x0000, 0x06); // MVI B
        app.snapshot.cpu.memory.write(0x0001, 0x42);
        app.memory_address_input = "0001".to_owned();
        app.refresh_memory_value(0x0001);
        app.keyboard_modifiers = keyboard::Modifiers::ALT;

        let _ = app.update(Message::EnterPressed);

        assert!(!app.monitor_open);
        assert_eq!(app.memory_address_input, "0001");
    }
}
