use iced::time;
use iced::{Subscription, Task, Theme};
use k580_app::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use std::time::Duration;

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) status: String,
    pub(crate) register_a_input: String,
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Tick,
    StepInstruction,
    StepTact,
    Run,
    ResetCpu,
    ResetRam,
    OpenSnapshot,
    SaveSnapshot,
    ExportTxt,
    ExportXlsx,
    ExportDocx,
    RegisterAChanged(String),
    ApplyRegisterA,
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    ApplyMemory,
}

impl DesktopApp {
    pub(crate) fn new() -> (Self, Task<Message>) {
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

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.pull_events(),
            Message::StepInstruction => self.dispatch(k580_app::AppCommand::StepInstruction),
            Message::StepTact => self.dispatch(k580_app::AppCommand::StepTact),
            Message::Run => self.dispatch(k580_app::AppCommand::Run),
            Message::ResetCpu => self.dispatch(k580_app::AppCommand::ResetCpu),
            Message::ResetRam => self.dispatch(k580_app::AppCommand::ResetRam),
            Message::OpenSnapshot => self.open_snapshot(),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::ExportTxt => self.export_txt(),
            Message::ExportXlsx => self.export_xlsx(),
            Message::ExportDocx => self.export_docx(),
            Message::RegisterAChanged(value) => self.register_a_input = value,
            Message::ApplyRegisterA => self.apply_register_a(),
            Message::MemoryAddressChanged(value) => self.memory_address_input = value,
            Message::MemoryValueChanged(value) => self.memory_value_input = value,
            Message::ApplyMemory => self.apply_memory(),
        }
        Task::none()
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }
}
