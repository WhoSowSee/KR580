use crate::app::{BackendAction, DesktopApp, REGISTER_VALUE_INPUT_ID};
use iced::Task;

impl DesktopApp {
    pub(super) fn finish_backend_action(&mut self, action: BackendAction) {
        let task = match action {
            BackendAction::None => return,
            BackendAction::Instruction => self.follow_pc_after_execution_boundary(),
            BackendAction::KeepCursor => {
                self.execution.pending_follow_pc = false;
                return;
            }
            BackendAction::Tact => self.follow_pc_after_execution_boundary(),
            BackendAction::Restart => {
                self.execution.running = true;
                self.dispatch(crate::backend::AppCommand::Run);
                return;
            }
            BackendAction::Replay(register) => {
                self.recompute_dirty();
                let memory = self.follow_pc_into_memory_list();
                if let Some(register) = register {
                    self.select_register(register);
                    self.interaction.focused_input = Some(REGISTER_VALUE_INPUT_ID);
                    Task::batch([
                        memory,
                        iced::widget::operation::focus(REGISTER_VALUE_INPUT_ID),
                    ])
                } else {
                    memory
                }
            }
            BackendAction::NewFile { edit_epoch } => {
                self.document.current_snapshot_path = None;
                self.document.current_subprogram_range = None;
                self.document.saved_cpu = k580_core::Cpu8080State::default();
                if edit_epoch == self.document.edit_epoch {
                    self.mark_saved();
                } else {
                    self.recompute_dirty();
                }
                self.set_status(crate::app::StatusKind::NewFile);
                return;
            }
            BackendAction::Register {
                source,
                value,
                target,
                replacing,
            } => self.finish_register_completion(source, value, target, replacing),
            BackendAction::Memory {
                address,
                value,
                target,
            } => self.finish_memory_completion(address, value, target),
            BackendAction::FloppyAttached(path) => {
                self.finish_floppy_attachment(path);
                return;
            }
            BackendAction::FloppyDetached => {
                if self.snapshot.devices.floppy.path.is_none() {
                    if self.panels.floppy_show_image_contents {
                        self.refresh_floppy_image_contents();
                    }
                    self.set_status_custom(
                        self.preferences
                            .lang
                            .t(crate::i18n::Key::FloppyImageDetached)
                            .to_owned(),
                    );
                }
                return;
            }
            BackendAction::HddAttached(path) => {
                if self.snapshot.devices.hdd.path.as_ref() == Some(&path) {
                    self.panels.hdd_file_exists = true;
                    self.set_status(crate::app::StatusKind::HddImageAttached {
                        display: path.display().to_string(),
                    });
                }
                return;
            }
            BackendAction::HddDeleted(path) => {
                self.panels.hdd_file_exists = false;
                self.set_status(crate::app::StatusKind::HddFileDeleted {
                    display: path.display().to_string(),
                });
                return;
            }
            BackendAction::NetworkConfigured { mode, host, port } => {
                if self.panels.network_mode_draft == mode
                    && self.panels.network_host_input.trim() == host
                    && self.panels.network_port_input.trim().parse::<u16>() == Ok(port)
                {
                    self.panels.network_settings_open = false;
                    self.panels.network_settings_error = None;
                }
                return;
            }
        };
        self.requests.tasks.push(task);
    }
}
