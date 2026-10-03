use crate::i18n::Key;
use crate::persistence::ShortcutAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsCategory {
    General,
    ExternalDevices,
    Appearance,
    Shortcuts,
}

impl SettingsCategory {
    pub(crate) const ALL: [Self; 4] = [
        Self::General,
        Self::ExternalDevices,
        Self::Appearance,
        Self::Shortcuts,
    ];

    pub(crate) fn label_key(self) -> Key {
        match self {
            Self::General => Key::SettingsCategoryGeneral,
            Self::ExternalDevices => Key::SettingsCategoryExternalDevices,
            Self::Appearance => Key::SettingsCategoryAppearance,
            Self::Shortcuts => Key::SettingsCategoryShortcuts,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FooterFocus {
    Reset,
    ShortcutReset,
    Cancel,
    Save,
}

impl FooterFocus {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Reset => Self::Cancel,
            Self::ShortcutReset => Self::Cancel,
            Self::Cancel => Self::Save,
            Self::Save => Self::Reset,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::Reset => Self::Save,
            Self::ShortcutReset => Self::Reset,
            Self::Cancel => Self::Reset,
            Self::Save => Self::Cancel,
        }
    }

    pub(crate) fn next_with_shortcuts(self, shortcuts: bool) -> Self {
        if !shortcuts {
            return self.next();
        }
        match self {
            Self::Reset => Self::ShortcutReset,
            Self::ShortcutReset => Self::Cancel,
            Self::Cancel => Self::Save,
            Self::Save => Self::Reset,
        }
    }

    pub(crate) fn previous_with_shortcuts(self, shortcuts: bool) -> Self {
        if !shortcuts {
            return self.previous();
        }
        match self {
            Self::Reset => Self::Save,
            Self::ShortcutReset => Self::Reset,
            Self::Cancel => Self::ShortcutReset,
            Self::Save => Self::Cancel,
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContentFocus {
    LanguageAnchor,
    SpeedSlow,
    SpeedMedium,
    SpeedFast,
    SpeedMax,
    FollowPcOn,
    FollowPcOff,
    MemoryOperandHighlightingOn,
    MemoryOperandHighlightingOff,
    ShowFileNameOn,
    ShowFileNameOff,
    FloppyImage,
    HddDirectory,
    PrinterDefault,
    PrinterDialogModeCustom,
    PrinterDialogModeSystem,
    MonitorLayoutUnified,
    MonitorLayoutSplit,
    NetworkDefaults,
    FileAssociation,
    Theme,
    Shortcut(ShortcutAction),
}

impl ContentFocus {
    pub(crate) const SPEEDS: [Self; 4] = [
        Self::SpeedSlow,
        Self::SpeedMedium,
        Self::SpeedFast,
        Self::SpeedMax,
    ];
}
