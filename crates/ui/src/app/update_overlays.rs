use iced::Task;

use super::changelog::ChangelogDialog;
use super::constants::{MEMORY_SCROLL_VISIBLE_TICKS, MONITOR_HEX_SCROLL_ID};
use super::help::HelpDialog;
use super::messages::Message;
use super::{DesktopApp, PendingAction};
use crate::runtime::parse::scroll_y_to;

impl DesktopApp {
    pub(crate) fn dispatch_overlay_message(&mut self, message: &Message) -> Option<Task<Message>> {
        match message {
            Message::OpenAbout => {
                self.close_top_menu();
                self.close_open_device_panel();
                self.shell.about_dialog_open = true;
            }
            Message::CloseAbout => {
                self.shell.about_dialog_open = false;
            }
            Message::OpenChangelog => {
                self.shell.changelog_dialog = Some(ChangelogDialog::new(self.preferences.lang));
            }
            Message::CloseChangelog => {
                self.shell.changelog_dialog = None;
            }
            Message::ChangelogReleaseSelected(selected) => {
                if let Some(dialog) = self.shell.changelog_dialog.as_mut() {
                    dialog.select_release(*selected);
                }
            }
            Message::ChangelogTextAction(action) => {
                if let Some(dialog) = self.shell.changelog_dialog.as_mut() {
                    dialog.perform_text_action(action.clone());
                }
            }
            Message::OpenHelp => {
                self.close_top_menu();
                self.hide_opcode_dropdown();
                self.close_open_device_panel();
                self.shell.help_dialog = Some(HelpDialog::new(self.preferences.lang));
            }
            Message::CloseHelp => {
                self.shell.help_dialog = None;
            }
            Message::HelpNodeSelected(node) => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
                    dialog.select_node(*node, self.preferences.lang);
                }
            }
            Message::HelpNodeToggled(node) => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
                    dialog.toggle_expanded(*node, self.preferences.lang);
                }
            }
            Message::HelpSearchChanged(query) => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
                    dialog.update_search_input(query.clone(), self.preferences.lang);
                }
            }
            Message::HelpSearchFinished(response) => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
                    dialog.apply_search_response(response.clone(), self.preferences.lang);
                }
            }
            Message::HelpTextAction(action) => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
                    dialog.perform_text_action(action.clone());
                }
            }
            Message::HelpToggleExpandAll => {
                if let Some(dialog) = self.shell.help_dialog.as_mut() {
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
                self.close_top_menu();
                self.hide_opcode_dropdown();
                let close_storage = Task::batch([
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.panels.monitor_open = true;
                if self.panels.monitor_window.detached()
                    && let Some(id) = self.panels.monitor_window.id()
                {
                    return Some(close_storage.chain(iced::window::gain_focus(id)));
                }
                return Some(close_storage);
            }
            Message::CloseMonitor => {
                return Some(self.close_monitor());
            }
            Message::ToggleMonitorSplit => {
                self.panels.monitor_split = !self.panels.monitor_split;
            }
            Message::ToggleMonitorHexPopup => {
                self.panels.monitor_hex_popup = !self.panels.monitor_hex_popup;
                if self.panels.monitor_hex_popup {
                    self.panels.monitor_hex_scroll_offset = 0.0;
                    self.panels.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
                }
            }
            Message::CycleMonitorHexFilter => {
                self.panels.monitor_hex_filter = self.panels.monitor_hex_filter.next();
                self.panels.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::MonitorHexScrolled(offset) => {
                self.panels.monitor_hex_scroll_offset = *offset;
                self.panels.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::MonitorHexScrollbarDragged(offset) => {
                self.panels.monitor_hex_scroll_offset = *offset;
                self.panels.monitor_hex_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
                return Some(scroll_y_to(MONITOR_HEX_SCROLL_ID, *offset));
            }
            Message::ClearMonitorBuffer => {
                self.dispatch(crate::backend::AppCommand::ClearMonitorBuffer);
            }
            Message::SaveMonitorImage => {
                return Some(self.save_monitor_image());
            }
            Message::MonitorImagePathSelected(path) => {
                self.save_monitor_image_to_path(path.clone());
            }
            Message::OpenFloppy => {
                self.close_top_menu();
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_hdd(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.panels.floppy_open = true;
                if self.panels.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
                if self.panels.floppy_window.detached()
                    && let Some(id) = self.panels.floppy_window.id()
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::CloseFloppy => {
                return Some(self.close_floppy());
            }
            Message::ToggleFloppyImageContents => {
                self.panels.floppy_show_image_contents = !self.panels.floppy_show_image_contents;
                self.invalidate_image_preview(k580_ui::devices::StorageKind::Floppy);
                if self.panels.floppy_show_image_contents {
                    self.refresh_floppy_image_contents();
                }
            }
            Message::OpenFloppyImage => {
                return Some(self.open_floppy_image());
            }
            Message::FloppyImagePathSelected(path) => {
                self.attach_floppy_image(path.clone());
            }
            Message::DetachFloppyImage => {
                self.dispatch_action(
                    crate::backend::AppCommand::DetachFloppyImage,
                    super::BackendAction::FloppyDetached,
                );
            }
            Message::SaveFloppyBuffer => {
                return Some(self.save_floppy_buffer());
            }
            Message::FloppyBufferPathSelected(path) => {
                self.save_floppy_buffer_to_path(path.clone());
            }
            Message::ToggleFloppyDebugBuffer => {
                self.dispatch_request(crate::backend::AppCommand::ToggleFloppyDebugBuffer);
            }
            Message::OpenHdd => {
                self.close_top_menu();
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_network(),
                    self.close_printer(),
                ]);
                self.panels.hdd_open = true;
                self.refresh_hdd_file_exists();
                if self.panels.hdd_show_image_contents {
                    self.refresh_hdd_image_contents();
                }
                if self.panels.hdd_window.detached()
                    && let Some(id) = self.panels.hdd_window.id()
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::CloseHdd => {
                return Some(self.close_hdd());
            }
            Message::ChooseHddDirectory => {
                return Some(self.choose_hdd_directory());
            }
            Message::HddDirectorySelected(path) => {
                self.attach_hdd_directory(path.clone());
            }
            Message::ToggleHddDebugBuffer => {
                self.dispatch_request(crate::backend::AppCommand::ToggleHddDebugBuffer);
            }
            Message::CreateHddFile => {
                self.create_hdd_file();
            }
            Message::ToggleHddImageContents => {
                self.panels.hdd_show_image_contents = !self.panels.hdd_show_image_contents;
                self.invalidate_image_preview(k580_ui::devices::StorageKind::Hdd);
                if self.panels.hdd_show_image_contents {
                    self.refresh_hdd_image_contents();
                }
            }
            Message::DeleteHddFile => {
                self.open_discard_modal(PendingAction::DeleteHdd);
            }
            Message::ClearHddBuffer => {
                self.dispatch(crate::backend::AppCommand::ClearHddBuffer);
            }
            Message::ClearFloppyBuffer => {
                self.dispatch(crate::backend::AppCommand::ClearFloppyBuffer);
            }
            Message::OpenNetwork => {
                self.close_top_menu();
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_printer(),
                ]);
                self.panels.network_open = true;
                if self.panels.network_window.detached()
                    && let Some(id) = self.panels.network_window.id()
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
                self.panels.network_settings_open = false;
                self.panels.network_settings_error = None;
            }
            Message::NetworkModeChanged(mode) => self.select_network_mode(*mode),
            Message::NetworkHostChanged(host) => {
                self.panels.network_host_input = host.clone();
                self.panels.network_settings_error = None;
            }
            Message::NetworkPortChanged(port) => {
                self.panels.network_port_input = port.clone();
                self.panels.network_settings_error = None;
            }
            Message::ApplyNetworkSettings => self.apply_network_settings(),
            Message::ClearNetworkBuffers => {
                self.dispatch(crate::backend::AppCommand::ClearNetworkBuffers);
            }
            Message::ToggleNetworkBufferView => {
                self.panels.network_text_view = !self.panels.network_text_view;
            }
            Message::OpenPrinter => {
                self.close_top_menu();
                self.hide_opcode_dropdown();
                let close_other = Task::batch([
                    self.close_monitor(),
                    self.close_floppy(),
                    self.close_hdd(),
                    self.close_network(),
                ]);
                self.panels.printer_open = true;
                if self.panels.printer_window.detached()
                    && let Some(id) = self.panels.printer_window.id()
                {
                    return Some(close_other.chain(iced::window::gain_focus(id)));
                }
                return Some(close_other);
            }
            Message::ClosePrinter => return Some(self.close_printer()),
            Message::TogglePrinterBufferView => {
                self.panels.printer_text_view = !self.panels.printer_text_view;
            }
            Message::ClearPrinterBuffer => {
                self.dispatch(crate::backend::AppCommand::ClearPrinterBuffer);
            }
            Message::PrintPrinterNative => self.print_printer_native(),
            Message::ConfigurePrinterSession => return Some(self.configure_printer_session()),
            Message::PrinterSessionSetupFinished(result) => {
                self.finish_printer_session_setup(result.clone());
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
