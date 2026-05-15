use iced::time;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Subscription, Task, Theme};
use k580_app::{
    AppCommand, AppEvent, AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator,
};
use k580_core::RegisterName;
use std::time::Duration;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    iced::application(DesktopApp::new, DesktopApp::update, DesktopApp::view)
        .title("KR580 Emulator")
        .subscription(DesktopApp::subscription)
        .theme(DesktopApp::theme)
        .run()
}

struct DesktopApp {
    handle: EmulatorHandle,
    snapshot: AppSnapshot,
    status: String,
    register_a_input: String,
    memory_address_input: String,
    memory_value_input: String,
}

#[derive(Clone, Debug)]
enum Message {
    Tick,
    StepInstruction,
    StepTact,
    Run,
    ResetCpu,
    ResetRam,
    OpenSnapshot,
    SaveSnapshot,
    ExportTxt,
    RegisterAChanged(String),
    ApplyRegisterA,
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    ApplyMemory,
}

impl DesktopApp {
    fn new() -> (Self, Task<Message>) {
        let handle = spawn_emulator();
        (
            Self {
                handle,
                snapshot: initial_snapshot(),
                status: "Ready".to_owned(),
                register_a_input: String::new(),
                memory_address_input: "0000".to_owned(),
                memory_value_input: "00".to_owned(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.pull_events(),
            Message::StepInstruction => self.dispatch(AppCommand::StepInstruction),
            Message::StepTact => self.dispatch(AppCommand::StepTact),
            Message::Run => self.dispatch(AppCommand::Run),
            Message::ResetCpu => self.dispatch(AppCommand::ResetCpu),
            Message::ResetRam => self.dispatch(AppCommand::ResetRam),
            Message::OpenSnapshot => self.open_snapshot(),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::ExportTxt => self.export_txt(),
            Message::RegisterAChanged(value) => self.register_a_input = value,
            Message::ApplyRegisterA => self.apply_register_a(),
            Message::MemoryAddressChanged(value) => self.memory_address_input = value,
            Message::MemoryValueChanged(value) => self.memory_value_input = value,
            Message::ApplyMemory => self.apply_memory(),
        }
        Task::none()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<'_, Message> {
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

    fn dispatch(&mut self, command: AppCommand) {
        if let Err(error) = self.handle.send(command) {
            self.status = error.to_string();
        }
        self.pull_events();
    }

    fn pull_events(&mut self) {
        for event in self.handle.drain_events() {
            match event {
                AppEvent::StateChanged(snapshot) => self.snapshot = *snapshot,
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

    fn open_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
            .pick_file()
        {
            self.dispatch(AppCommand::LoadSnapshot(path));
        }
    }

    fn save_snapshot(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KR580 snapshot", &["580"])
            .save_file()
        {
            self.dispatch(AppCommand::SaveSnapshot(path));
        }
    }

    fn export_txt(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .save_file()
        {
            self.dispatch(AppCommand::ExportTxt(path));
        }
    }

    fn apply_register_a(&mut self) {
        match parse_hex_u8(&self.register_a_input) {
            Ok(value) => self.dispatch(AppCommand::SetRegister(RegisterName::A, value)),
            Err(error) => self.status = error,
        }
    }

    fn apply_memory(&mut self) {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => self.dispatch(AppCommand::SetMemory(address, value)),
            (Err(error), _) | (_, Err(error)) => self.status = error,
        }
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

fn parse_hex_u8(input: &str) -> Result<u8, String> {
    u8::from_str_radix(input.trim().trim_start_matches("0x"), 16)
        .map_err(|_| format!("Invalid byte hex: {input}"))
}

fn parse_hex_u16(input: &str) -> Result<u16, String> {
    u16::from_str_radix(input.trim().trim_start_matches("0x"), 16)
        .map_err(|_| format!("Invalid address hex: {input}"))
}
