use super::focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
use crate::app::messages::SpeedTier;
use crate::i18n::Lang;

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
    pub(crate) language_dropdown_open: bool,
    /// Keyboard highlight inside the open language dropdown. `None`
    /// when the dropdown is closed; while open, ArrowUp / ArrowDown
    /// move the highlight here without committing – the draft only
    /// changes once the user presses Enter or clicks an option.
    pub(crate) dropdown_highlight: Option<Lang>,
    pub(crate) original_lang: Lang,
    pub(crate) original_speed: SpeedTier,
    pub(crate) footer_focus: FooterFocus,
    pub(crate) reset_confirm_open: bool,
    pub(crate) reset_confirm_focus: ResetConfirmFocus,
    pub(crate) section: SettingsSection,
    pub(crate) content_focus: Option<ContentFocus>,
}

impl SettingsDialog {
    pub(crate) fn new(lang: Lang, speed: SpeedTier) -> Self {
        Self {
            category: SettingsCategory::General,
            search: String::new(),
            draft_lang: lang,
            draft_speed: speed,
            language_dropdown_open: false,
            dropdown_highlight: None,
            original_lang: lang,
            original_speed: speed,
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
            SettingsCategory::Appearance => ContentFocus::Theme,
            SettingsCategory::Shortcuts => ContentFocus::Shortcuts,
        }
    }

    pub(crate) fn last_content_focus(&self) -> ContentFocus {
        match self.category {
            SettingsCategory::General => ContentFocus::SpeedMax,
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
                ContentFocus::SpeedMax => None,
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
