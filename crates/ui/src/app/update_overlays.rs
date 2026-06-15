use iced::Task;

use super::constants::MEMORY_SCROLL_VISIBLE_TICKS;
use super::help::HelpDialog;
use super::messages::Message;
use super::state::{DesktopApp, PendingAction};

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
                let close_storage = Task::batch([
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.monitor_open = true;
                if self.monitor_window.detached
                    && let Some(id) = self.monitor_window.id
                {
                    return Some(close_storage.chain(iced::window::gain_focus(id)));
                }
                return Some(close_storage);
            }
            Message::CloseMonitor => {
                return Some(self.close_monitor());
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
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_hdd(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.floppy_open = true;
                if self.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
                if self.floppy_window.detached
                    && let Some(id) = self.floppy_window.id
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::CloseFloppy => {
                return Some(self.close_floppy());
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
            Message::OpenHdd => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.hdd_open = true;
                self.refresh_hdd_file_exists();
                if self.hdd_show_image_contents {
                    self.refresh_hdd_image_contents();
                }
                if self.hdd_window.detached
                    && let Some(id) = self.hdd_window.id
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::CloseHdd => {
                return Some(self.close_hdd());
            }
            Message::ChooseHddDirectory => {
                self.choose_hdd_directory();
            }
            Message::ToggleHddDebugBuffer => {
                let enabled = !self.snapshot.devices.hdd.debug_buffer;
                self.dispatch_sync(k580_app::AppCommand::SetHddDebugBuffer(enabled));
            }
            Message::CreateHddFile => {
                self.create_hdd_file();
            }
            Message::ToggleHddImageContents => {
                self.hdd_show_image_contents = !self.hdd_show_image_contents;
                if self.hdd_show_image_contents {
                    self.refresh_hdd_image_contents();
                }
            }
            Message::DeleteHddFile => {
                self.open_discard_modal(PendingAction::DeleteHdd);
            }
            Message::ClearHddBuffer => {
                self.dispatch(k580_app::AppCommand::ClearHddBuffer);
            }
            Message::ClearFloppyBuffer => {
                self.dispatch(k580_app::AppCommand::ClearFloppyBuffer);
            }
            Message::OpenNetwork => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_printer(),
                ]);
                self.network_open = true;
                if self.network_window.detached
                    && let Some(id) = self.network_window.id
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::CloseNetwork => {
                return Some(self.close_network());
            }
            Message::OpenNetworkSettings => self.open_network_settings(),
            Message::CloseNetworkSettings => {
                self.network_settings_open = false;
                self.network_settings_error = None;
            }
            Message::NetworkModeChanged(mode) => self.select_network_mode(*mode),
            Message::NetworkHostChanged(host) => {
                self.network_host_input = host.clone();
                self.network_settings_error = None;
            }
            Message::NetworkPortChanged(port) => {
                self.network_port_input = port.clone();
                self.network_settings_error = None;
            }
            Message::ApplyNetworkSettings => self.apply_network_settings(),
            Message::ClearNetworkBuffers => {
                self.dispatch(k580_app::AppCommand::ClearNetworkBuffers);
            }
            Message::ToggleNetworkBufferView => {
                self.network_text_view = !self.network_text_view;
            }
            Message::OpenPrinter => {
                self.open_menu = None;
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_network(),
                ]);
                self.printer_open = true;
                if self.printer_window.detached
                    && let Some(id) = self.printer_window.id
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::ClosePrinter => return Some(self.close_printer()),
            Message::TogglePrinterBufferView => {
                self.printer_text_view = !self.printer_text_view;
            }
            Message::ClearPrinterBuffer => {
                self.dispatch(k580_app::AppCommand::ClearPrinterBuffer);
            }
            Message::PrintPrinterPdf => self.print_printer_pdf(),
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
