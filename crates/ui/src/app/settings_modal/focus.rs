use crate::i18n::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsCategory {
    General,
    Appearance,
    Shortcuts,
}

impl SettingsCategory {
    pub(crate) const ALL: [Self; 3] = [Self::General, Self::Appearance, Self::Shortcuts];

    pub(crate) fn label_key(self) -> Key {
        match self {
            Self::General => Key::SettingsCategoryGeneral,
            Self::Appearance => Key::SettingsCategoryAppearance,
            Self::Shortcuts => Key::SettingsCategoryShortcuts,
        }
    }
}

/// Footer button focus. `Tab` cycles forward, `Shift+Tab` backward,
/// `Enter` activates whichever side is focused. Defaults to `Cancel`
/// so an accidental `Enter` press does not commit a draft change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FooterFocus {
    Reset,
    Cancel,
    Save,
}

impl FooterFocus {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Reset => Self::Cancel,
            Self::Cancel => Self::Save,
            Self::Save => Self::Reset,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::Reset => Self::Save,
            Self::Cancel => Self::Reset,
            Self::Save => Self::Cancel,
        }
    }
}

/// Reset-confirm sub-modal focus. Two buttons (Cancel / Confirm), Tab
/// toggles between them. Defaults to `Cancel` so a stray `Enter` does
/// not destroy settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResetConfirmFocus {
    Cancel,
    Confirm,
}

impl ResetConfirmFocus {
    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Cancel => Self::Confirm,
            Self::Confirm => Self::Cancel,
        }
    }
}

/// Top-level keyboard zone. `Ctrl+Tab` cycles forward through these
/// zones, `Ctrl+Shift+Tab` backward; plain `Tab` walks within the
/// active zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsSection {
    Search,
    Sidebar,
    Content,
    Footer,
}

impl SettingsSection {
    pub(crate) const ALL: [Self; 4] = [Self::Search, Self::Sidebar, Self::Content, Self::Footer];

    pub(crate) fn next(self) -> Self {
        let cur = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(cur + 1) % Self::ALL.len()]
    }

    pub(crate) fn previous(self) -> Self {
        let cur = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(cur + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Focus inside the right-hand content pane. Order matches the
/// vertical layout: language anchor on top, then the speed segments
/// left-to-right, or the theme placeholder on the Appearance tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContentFocus {
    LanguageAnchor,
    SpeedSlow,
    SpeedMedium,
    SpeedFast,
    SpeedMax,
    Theme,
    Shortcuts,
}

impl ContentFocus {
    pub(crate) const SPEEDS: [Self; 4] = [
        Self::SpeedSlow,
        Self::SpeedMedium,
        Self::SpeedFast,
        Self::SpeedMax,
    ];
}
