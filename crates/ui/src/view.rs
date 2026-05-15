use crate::app::{DesktopApp, Message};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

impl DesktopApp {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;
        let registers = column![
            text("Registers").size(24),
            text(format!(
                "A {:02X}   B {:02X}   C {:02X}",
                cpu.registers.a, cpu.registers.b, cpu.registers.c
            )),
            text(format!(
                "D {:02X}   E {:02X}   H {:02X}   L {:02X}",
                cpu.registers.d, cpu.registers.e, cpu.registers.h, cpu.registers.l
            )),
            text(format!(
                "PC {:04X}   SP {:04X}   cycles {}",
                cpu.pc, cpu.sp, cpu.cycle_count
            )),
            text(format!(
                "Flags S={} Z={} AC={} P={} CY={}",
                cpu.flags.sign,
                cpu.flags.zero,
                cpu.flags.auxiliary_carry,
                cpu.flags.parity,
                cpu.flags.carry
            )),
            row![
                text_input("A hex", &self.register_a_input)
                    .on_input(Message::RegisterAChanged)
                    .width(Length::Fixed(90.0)),
                button("Set A").on_press(Message::ApplyRegisterA),
            ]
            .spacing(8),
        ]
        .spacing(8);

        let controls = row![
            button("Step instruction").on_press(Message::StepInstruction),
            button("Step tact").on_press(Message::StepTact),
            button("Run until halt").on_press(Message::Run),
            button("Reset CPU").on_press(Message::ResetCpu),
            button("Reset RAM").on_press(Message::ResetRam),
        ]
        .spacing(8);

        let files = row![
            button("Open .580").on_press(Message::OpenSnapshot),
            button("Save .580").on_press(Message::SaveSnapshot),
            button("Export .txt").on_press(Message::ExportTxt),
            button("Export .xlsx").on_press(Message::ExportXlsx),
            button("Export .docx").on_press(Message::ExportDocx),
        ]
        .spacing(8);

        let memory_editor = row![
            text_input("addr", &self.memory_address_input)
                .on_input(Message::MemoryAddressChanged)
                .width(Length::Fixed(90.0)),
            text_input("value", &self.memory_value_input)
                .on_input(Message::MemoryValueChanged)
                .width(Length::Fixed(80.0)),
            button("Set memory").on_press(Message::ApplyMemory),
        ]
        .spacing(8);

        let memory = column![
            text("RAM 0000h..003Fh").size(24),
            memory_editor,
            text(memory_preview(cpu)),
        ]
        .spacing(8);

        let devices = column![
            text("Peripherals").size(24),
            text(format!(
                "Monitor: {:?}",
                self.snapshot.devices.monitor.status
            )),
            text(format!("Floppy: {:?}", self.snapshot.devices.floppy.status)),
            text(format!("HDD: {:?}", self.snapshot.devices.hdd.status)),
            text(format!(
                "Network: {:?}",
                self.snapshot.devices.network.status
            )),
            text(format!(
                "Printer: {:?}",
                self.snapshot.devices.printer.status
            )),
        ]
        .spacing(5);

        let content = column![
            text("KR580 / Intel 8080 Emulator").size(32),
            controls,
            files,
            registers,
            memory,
            devices,
            text(format!("Status: {}", self.status)),
        ]
        .padding(16)
        .spacing(14);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn memory_preview(cpu: &k580_core::Cpu8080State) -> String {
    let mut lines = Vec::new();
    for row in 0..4u16 {
        let base = row * 16;
        let bytes = (0..16u16)
            .map(|offset| format!("{:02X}", cpu.memory.read(base + offset)))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!("{base:04X}: {bytes}"));
    }
    lines.join("\n")
}
