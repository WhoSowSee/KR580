//! iced view layer.
//!
//! Per `prompt/05_ui_and_workflows.md`: forms only render state and dispatch
//! commands. We hold the latest [`StateView`] from the core actor, draw it,
//! and forward button / menu actions onto the command channel.

use crate::runtime::{RuntimeHandles, StateView, UiCommand, UiEvent};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{executor, Application, Command, Element, Length, Subscription, Theme};
use kr580_core::Reg8;
use std::sync::Arc;
use std::time::Duration;

/// UI message type.
#[derive(Debug, Clone)]
pub enum Message {
    /// Tick: drain pending events from the core actor.
    Tick,
    /// Step a single instruction.
    Step,
    /// Run continuously.
    Run,
    /// Stop a running core.
    Stop,
    /// Reset CPU.
    ResetCpu,
    /// Reset registers only.
    ResetRegisters,
    /// Reset RAM.
    ResetRam,
    /// Save snapshot.
    SaveSnapshot,
    /// Load snapshot from disk.
    LoadSnapshot,
    /// Snapshot bytes returned by the core.
    SnapshotBytes(Vec<u8>),
    /// Edit a register input.
    RegEdit(Reg8, String),
    /// Commit a register edit.
    RegCommit(Reg8),
    /// Memory address typed by the user (hex).
    MemAddrEdit(String),
    /// Memory value typed by the user (hex).
    MemValueEdit(String),
    /// Commit the memory edit.
    MemCommit,
}

/// iced application that owns the runtime handles and the latest snapshot.
pub struct EmulatorApp {
    handles: RuntimeHandles,
    state: Option<Arc<StateView>>,
    last_error: Option<String>,
    reg_edits: [(Reg8, String); 7],
    mem_addr: String,
    mem_value: String,
}

impl EmulatorApp {
    /// Build an iced [`iced::Application::Flags`] payload from runtime handles.
    pub fn flags(handles: RuntimeHandles) -> RuntimeHandles {
        handles
    }
}

impl Application for EmulatorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = RuntimeHandles;

    fn new(handles: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = Self {
            handles,
            state: None,
            last_error: None,
            reg_edits: [
                (Reg8::A, String::from("00")),
                (Reg8::B, String::from("00")),
                (Reg8::C, String::from("00")),
                (Reg8::D, String::from("00")),
                (Reg8::E, String::from("00")),
                (Reg8::H, String::from("00")),
                (Reg8::L, String::from("00")),
            ],
            mem_addr: String::from("0000"),
            mem_value: String::from("00"),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("KR580 / Intel 8080 Emulator")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                // Drain pending core events. State snapshots take priority
                // over other events so the UI always shows authoritative
                // data on the next render.
                let mut latest_state: Option<Arc<StateView>> = None;
                while let Ok(evt) = self.handles.events.try_recv() {
                    match evt {
                        UiEvent::State(s) => latest_state = Some(s),
                        UiEvent::Error(msg) => self.last_error = Some(msg),
                        UiEvent::SnapshotSaved(bytes) => {
                            return Command::perform(
                                async move { save_snapshot_via_dialog(bytes).await },
                                |_| Message::Tick,
                            );
                        }
                        UiEvent::SnapshotLoaded => self.last_error = None,
                        UiEvent::InstructionExecuted { .. }
                        | UiEvent::HaltChanged(_) => {}
                    }
                }
                if let Some(s) = latest_state {
                    self.state = Some(s);
                }
            }
            Message::Step => {
                let _ = self.handles.commands.send(UiCommand::StepInstruction);
            }
            Message::Run => {
                let _ = self.handles.commands.send(UiCommand::Run);
            }
            Message::Stop => {
                let _ = self.handles.commands.send(UiCommand::Stop);
            }
            Message::ResetCpu => {
                let _ = self.handles.commands.send(UiCommand::ResetCpu);
            }
            Message::ResetRegisters => {
                let _ = self.handles.commands.send(UiCommand::ResetRegisters);
            }
            Message::ResetRam => {
                let _ = self.handles.commands.send(UiCommand::ResetRam);
            }
            Message::SaveSnapshot => {
                let _ = self.handles.commands.send(UiCommand::SaveSnapshot);
            }
            Message::LoadSnapshot => {
                return Command::perform(load_snapshot_via_dialog(), |maybe| match maybe {
                    Some(bytes) => Message::SnapshotBytes(bytes),
                    None => Message::Tick,
                });
            }
            Message::SnapshotBytes(bytes) => {
                let _ = self.handles.commands.send(UiCommand::LoadSnapshot(bytes));
            }
            Message::RegEdit(reg, val) => {
                if let Some(slot) = self.reg_edits.iter_mut().find(|(r, _)| *r == reg) {
                    slot.1 = val;
                }
            }
            Message::RegCommit(reg) => {
                if let Some((_, val)) = self.reg_edits.iter().find(|(r, _)| *r == reg) {
                    if let Ok(v) = u8::from_str_radix(val.trim(), 16) {
                        let _ = self.handles.commands.send(UiCommand::SetRegister(reg, v));
                    } else {
                        self.last_error = Some(format!("invalid hex for register: {val}"));
                    }
                }
            }
            Message::MemAddrEdit(s) => self.mem_addr = s,
            Message::MemValueEdit(s) => self.mem_value = s,
            Message::MemCommit => {
                let addr = u16::from_str_radix(self.mem_addr.trim(), 16);
                let val = u8::from_str_radix(self.mem_value.trim(), 16);
                match (addr, val) {
                    (Ok(a), Ok(v)) => {
                        let _ = self.handles.commands.send(UiCommand::SetMemory(a, v));
                    }
                    _ => {
                        self.last_error =
                            Some("memory edit needs hex address and hex value".to_string());
                    }
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // iced 0.12 requires the closure passed to `Subscription::map` to be
        // non-capturing. We therefore use the tick purely as a heartbeat and
        // drain events from `self.handles.events` inside `update(Tick)`,
        // where `&mut self` is available.
        iced::time::every(Duration::from_millis(33)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let header = text("KR580 / Intel 8080 Emulator").size(24);
        let toolbar = row![
            button(text("Step")).on_press(Message::Step),
            button(text("Run")).on_press(Message::Run),
            button(text("Stop")).on_press(Message::Stop),
            Space::with_width(Length::Fixed(16.0)),
            button(text("Reset CPU")).on_press(Message::ResetCpu),
            button(text("Reset Regs")).on_press(Message::ResetRegisters),
            button(text("Reset RAM")).on_press(Message::ResetRam),
            Space::with_width(Length::Fixed(16.0)),
            button(text("Save .580")).on_press(Message::SaveSnapshot),
            button(text("Load .580")).on_press(Message::LoadSnapshot),
        ]
        .spacing(8);

        let state_view: Element<'_, Self::Message> = match &self.state {
            Some(s) => self.render_state(s),
            None => text("Awaiting initial state…").into(),
        };

        let body = column![
            header,
            toolbar,
            state_view,
            Space::with_height(Length::Fixed(8.0)),
            self.render_register_editor(),
            self.render_memory_editor(),
            self.render_error(),
        ]
        .spacing(8);

        container(scrollable(body))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl EmulatorApp {
    fn render_state(&self, s: &StateView) -> Element<'_, Message> {
        let regs = s
            .registers
            .iter()
            .fold(row![].spacing(12), |r, (name, value)| {
                r.push(text(format!("{:?}={:02X}", name, value)).size(16))
            });
        let flags = format!(
            "S={} Z={} AC={} P={} CY={}",
            s.flags.s as u8, s.flags.z as u8, s.flags.ac as u8, s.flags.p as u8, s.flags.cy as u8,
        );
        column![
            regs,
            text(format!(
                "PC={:04X}  SP={:04X}  cycles={}  halted={}",
                s.pc, s.sp, s.cycle_count, s.halted
            )),
            text(format!("Flags: {flags}")),
        ]
        .spacing(4)
        .into()
    }

    fn render_register_editor(&self) -> Element<'_, Message> {
        let mut row_widget = row![text("Edit register: ")].spacing(4);
        for (reg, val) in &self.reg_edits {
            let r = *reg;
            row_widget = row_widget.push(
                row![
                    text(format!("{r:?}")),
                    text_input("hex", val)
                        .width(Length::Fixed(48.0))
                        .on_input(move |s| Message::RegEdit(r, s)),
                    button(text("Set")).on_press(Message::RegCommit(r)),
                ]
                .spacing(2),
            );
        }
        row_widget.into()
    }

    fn render_memory_editor(&self) -> Element<'_, Message> {
        row![
            text("Memory: addr "),
            text_input("hex", &self.mem_addr)
                .width(Length::Fixed(64.0))
                .on_input(Message::MemAddrEdit),
            text(" value "),
            text_input("hex", &self.mem_value)
                .width(Length::Fixed(48.0))
                .on_input(Message::MemValueEdit),
            button(text("Write")).on_press(Message::MemCommit),
        ]
        .spacing(4)
        .into()
    }

    fn render_error(&self) -> Element<'_, Message> {
        match &self.last_error {
            Some(e) => text(format!("Error: {e}")).size(14).into(),
            None => Space::with_height(Length::Fixed(0.0)).into(),
        }
    }
}

async fn save_snapshot_via_dialog(bytes: Vec<u8>) -> Option<()> {
    let path = rfd::AsyncFileDialog::new()
        .add_filter(".580", &["580"])
        .save_file()
        .await?;
    let _ = path.write(&bytes).await;
    Some(())
}

async fn load_snapshot_via_dialog() -> Option<Vec<u8>> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter(".580", &["580"])
        .pick_file()
        .await?;
    Some(handle.read().await)
}
