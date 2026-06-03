use iced::Task;

use super::constants::MEMORY_SCROLL_VISIBLE_TICKS;
use super::help::HelpDialog;
use super::messages::Message;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn dispatch_overlay_message(&mut self, message: &Message) -> Option<Task<Message>> {
        match message {
            Message::OpenAbout => {
                self.open_menu = None;
                self.about_dialog_open = true;
            }
            Message::CloseAbout => {
                self.about_dialog_open = false;
            }
            Message::OpenHelp => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                self.help_dialog = Some(HelpDialog::new(self.lang));
            }
            Message::CloseHelp => {
                self.help_dialog = None;
            }
            Message::HelpNodeSelected(node) => {
                if let Some(dialog) = self.help_dialog.as_mut() {
                    dialog.select_node(*node, self.lang);
                }
            }
            Message::HelpNodeToggled(node) => {
                if let Some(dialog) = self.help_dialog.as_mut() {
                    dialog.toggle_expanded(*node, self.lang);
                }
            }
            Message::HelpSearchChanged(query) => {
                if let Some(dialog) = self.help_dialog.as_mut() {
                    dialog.update_search_input(query.clone(), self.lang);
                }
            }
            Message::HelpTextAction(action) => {
                if let Some(dialog) = self.help_dialog.as_mut() {
                    dialog.perform_text_action(action.clone());
                }
            }
            Message::HelpToggleExpandAll => {
                if let Some(dialog) = self.help_dialog.as_mut() {
                    if dialog.all_expanded() {
                        dialog.collapse_all();
                    } else {
                        dialog.expand_all();
                    }
                }
            }
            Message::OpenUrl(url) => {
                if let Err(error) = open_external_url(url) {
                    tracing::warn!("failed to open url {url}: {error}");
                }
            }
            Message::OpenMonitor => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                self.floppy_open = false;
                self.monitor_open = true;
            }
            Message::CloseMonitor => {
                self.monitor_open = false;
                self.monitor_hex_popup = false;
            }
            Message::ToggleMonitorSplit => {
                self.monitor_split = !self.monitor_split;
            }
            Message::ToggleMonitorHexPopup => {
                self.monitor_hex_popup = !self.monitor_hex_popup;
                if self.monitor_hex_popup {
                    self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
                }
            }
            Message::CycleMonitorHexFilter => {
                self.monitor_hex_filter = self.monitor_hex_filter.next();
                self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::MonitorHexScrolled => {
                self.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::ClearMonitorBuffer => {
                self.dispatch(k580_app::AppCommand::ClearMonitorBuffer);
            }
            Message::SaveMonitorImage => {
                self.save_monitor_image();
            }
            Message::OpenFloppy => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                self.monitor_open = false;
                self.monitor_hex_popup = false;
                self.floppy_open = true;
                if self.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
            }
            Message::CloseFloppy => {
                self.floppy_open = false;
            }
            Message::ToggleFloppyImageContents => {
                self.floppy_show_image_contents = !self.floppy_show_image_contents;
                if self.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
            }
            Message::OpenFloppyImage => {
                self.open_floppy_image();
            }
            Message::DetachFloppyImage => {
                self.dispatch_sync(k580_app::AppCommand::DetachFloppyImage);
                self.set_status_custom(
                    self.lang
                        .t(crate::i18n::Key::FloppyImageDetached)
                        .to_owned(),
                );
                if self.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
            }
            Message::SaveFloppyBuffer => {
                self.save_floppy_buffer();
            }
            Message::ToggleFloppyDebugBuffer => {
                let enabled = !self.snapshot.devices.floppy.debug_buffer;
                self.dispatch_sync(k580_app::AppCommand::SetFloppyDebugBuffer(enabled));
            }
            Message::ClearFloppyBuffer => {
                self.dispatch(k580_app::AppCommand::ClearFloppyBuffer);
            }
            _ => return None,
        }
        Some(Task::none())
    }
}

#[cfg(target_os = "windows")]
fn open_external_url(url: &str) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_external_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open").arg(url).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_external_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
