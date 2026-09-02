use super::network::parse_network_defaults;
use crate::app::state::DesktopApp;
use crate::settings_storage::{default_settings, lang_from_language, speed_tier_from_preset};

impl DesktopApp {
    pub(super) fn reset_settings(&mut self) {
        let defaults = default_settings();
        let default_lang = lang_from_language(defaults.general.language);
        let default_speed = speed_tier_from_preset(defaults.general.default_speed);
        let general = defaults.general;
        let color_scheme = defaults.ui.theme;
        let network = defaults.network;
        if let Some(dialog) = self.settings_dialog.as_mut() {
            dialog.draft_lang = default_lang;
            dialog.draft_speed = default_speed;
            dialog.draft_color_scheme = color_scheme;
            dialog.draft_floppy_image_path = general.floppy_image_path;
            dialog.draft_hdd_directory = general.hdd_directory;
            dialog.draft_printer_settings = general.printer_settings;
            dialog.draft_printer_dialog_mode = general.printer_dialog_mode;
            dialog.draft_network_client_host = network.host;
            dialog.draft_network_client_port = network.port.to_string();
            dialog.draft_network_server_host = network.bind_host;
            dialog.draft_network_server_port = network.bind_port.to_string();
            dialog.draft_shortcuts = defaults.shortcuts;
            dialog.recording_shortcut = None;
            dialog.draft_follow_pc = general.follow_pc;
            dialog.draft_memory_operand_highlighting = general.memory_operand_highlighting;
            dialog.draft_show_file_name = general.show_file_name;
            dialog.draft_monitor_split = general.monitor_split;
            dialog.network_error = None;
            dialog.reset_confirm_open = false;
            dialog.reset_confirm_keyboard_focus_visible = false;
        }
        self.follow_pc = general.follow_pc;
        self.memory_operand_highlighting = general.memory_operand_highlighting;
        self.show_file_name = general.show_file_name;
        self.monitor_split = general.monitor_split;
        self.default_speed = default_speed;
        self.color_scheme = color_scheme;
        self.apply_speed_tier(default_speed);
        self.apply_language(default_lang);
        self.commit_settings_dialog_state();
        if let Some(dialog) = self.settings_dialog.as_ref()
            && let Ok(network) = parse_network_defaults(dialog)
        {
            self.save_settings_dialog(dialog, network);
        }
    }
}
