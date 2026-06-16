use super::focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
use crate::app::messages::SpeedTier;
use crate::i18n::Lang;
use k580_persistence::NetworkSettings;

/// Draft state edited by the dialog. Live language and speed (the
/// fields on `DesktopApp`) are kept in sync with the draft so the user
/// sees changes apply immediately; `original_*` snapshots remember the
/// values at the time the dialog was opened so Cancel / backdrop click
/// can roll the live state back to that snapshot.
#[derive(Clone, Debug)]
pub(crate) struct SettingsDialog {
    pub(crate) category: SettingsCategory,
    pub(crate) search: String,
    pub(crate) draft_lang: Lang,
    pub(crate) draft_speed: SpeedTier,
    pub(crate) draft_follow_pc: bool,
    pub(crate) draft_floppy_image_path: Option<std::path::PathBuf>,
    pub(crate) draft_hdd_directory: Option<std::path::PathBuf>,
    pub(crate) draft_network_client_host: String,
    pub(crate) draft_network_client_port: String,
    pub(crate) draft_network_server_host: String,
    pub(crate) draft_network_server_port: String,
    pub(crate) network_error: Option<String>,
    pub(crate) language_dropdown_open: bool,
    /// Keyboard highlight inside the open language dropdown. `None`
    /// when the dropdown is closed; while open, ArrowUp / ArrowDown
    /// move the highlight here without committing – the draft only
    /// changes once the user presses Enter or clicks an option.
    pub(crate) dropdown_highlight: Option<Lang>,
    pub(crate) original_lang: Lang,
    pub(crate) original_speed: SpeedTier,
    pub(crate) original_follow_pc: bool,
    pub(crate) footer_focus: FooterFocus,
    pub(crate) reset_confirm_open: bool,
    pub(crate) reset_confirm_focus: ResetConfirmFocus,
    pub(crate) section: SettingsSection,
    pub(crate) content_focus: Option<ContentFocus>,
}

impl SettingsDialog {
    pub(crate) fn new(
        lang: Lang,
        speed: SpeedTier,
        follow_pc: bool,
        floppy_image_path: Option<std::path::PathBuf>,
        hdd_directory: Option<std::path::PathBuf>,
        network: NetworkSettings,
    ) -> Self {
        Self {
            category: SettingsCategory::General,
            search: String::new(),
            draft_lang: lang,
            draft_speed: speed,
            draft_follow_pc: follow_pc,
            draft_floppy_image_path: floppy_image_path.clone(),
            draft_hdd_directory: hdd_directory.clone(),
            draft_network_client_host: network.host,
            draft_network_client_port: network.port.to_string(),
            draft_network_server_host: network.bind_host,
            draft_network_server_port: network.bind_port.to_string(),
            network_error: None,
            language_dropdown_open: false,
            dropdown_highlight: None,
            original_lang: lang,
            original_speed: speed,
            original_follow_pc: follow_pc,
            footer_focus: FooterFocus::Cancel,
            reset_confirm_open: false,
            reset_confirm_focus: ResetConfirmFocus::Cancel,
            section: SettingsSection::Footer,
            content_focus: None,
        }
    }

    pub(crate) fn search_query(&self) -> &str {
        self.search.trim()
    }

    pub(crate) fn first_content_focus(&self) -> ContentFocus {
        match self.category {
            SettingsCategory::General => ContentFocus::LanguageAnchor,
            SettingsCategory::ExternalDevices => ContentFocus::FloppyImage,
            SettingsCategory::Appearance => ContentFocus::Theme,
            SettingsCategory::Shortcuts => ContentFocus::Shortcuts,
        }
    }

    pub(crate) fn last_content_focus(&self) -> ContentFocus {
        match self.category {
            SettingsCategory::General => ContentFocus::FileAssociation,
            SettingsCategory::ExternalDevices => ContentFocus::NetworkDefaults,
            SettingsCategory::Appearance => ContentFocus::Theme,
            SettingsCategory::Shortcuts => ContentFocus::Shortcuts,
        }
    }

    pub(crate) fn next_content_focus(&self, current: ContentFocus) -> Option<ContentFocus> {
        match self.category {
            SettingsCategory::General => match current {
                ContentFocus::LanguageAnchor => Some(ContentFocus::SpeedSlow),
                ContentFocus::SpeedSlow => Some(ContentFocus::SpeedMedium),
                ContentFocus::SpeedMedium => Some(ContentFocus::SpeedFast),
                ContentFocus::SpeedFast => Some(ContentFocus::SpeedMax),
                ContentFocus::SpeedMax => Some(ContentFocus::FollowPc),
                ContentFocus::FollowPc => Some(ContentFocus::FileAssociation),
                ContentFocus::FileAssociation => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::ExternalDevices => match current {
                ContentFocus::FloppyImage => Some(ContentFocus::HddDirectory),
                ContentFocus::HddDirectory => Some(ContentFocus::NetworkDefaults),
                ContentFocus::NetworkDefaults => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::Appearance => match current {
                ContentFocus::Theme => None,
                _ => Some(self.first_content_focus()),
            },
            SettingsCategory::Shortcuts => match current {
                ContentFocus::Shortcuts => None,
                _ => Some(self.first_content_focus()),
            },
        }
    }

    pub(crate) fn previous_content_focus(&self, current: ContentFocus) -> Option<ContentFocus> {
        match self.category {
            SettingsCategory::General => match current {
                ContentFocus::LanguageAnchor => None,
                ContentFocus::SpeedSlow => Some(ContentFocus::LanguageAnchor),
                ContentFocus::SpeedMedium => Some(ContentFocus::SpeedSlow),
                ContentFocus::SpeedFast => Some(ContentFocus::SpeedMedium),
                ContentFocus::SpeedMax => Some(ContentFocus::SpeedFast),
                ContentFocus::FollowPc => Some(ContentFocus::SpeedMax),
                ContentFocus::FileAssociation => Some(ContentFocus::FollowPc),
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::ExternalDevices => match current {
                ContentFocus::FloppyImage => None,
                ContentFocus::HddDirectory => Some(ContentFocus::FloppyImage),
                ContentFocus::NetworkDefaults => Some(ContentFocus::HddDirectory),
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::Appearance => match current {
                ContentFocus::Theme => None,
                _ => Some(self.last_content_focus()),
            },
            SettingsCategory::Shortcuts => match current {
                ContentFocus::Shortcuts => None,
                _ => Some(self.last_content_focus()),
            },
        }
    }
}
