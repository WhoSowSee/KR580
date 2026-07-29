use crate::i18n::Key;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImportFileFormat {
    Xlsx,
    Text,
}

impl ImportFileFormat {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        if extension.eq_ignore_ascii_case("xlsx") {
            Some(Self::Xlsx)
        } else if extension.eq_ignore_ascii_case("txt") {
            Some(Self::Text)
        } else {
            None
        }
    }

    pub(crate) fn target_label_key(self) -> Key {
        match self {
            Self::Xlsx => Key::ImportSheetLabel,
            Self::Text => Key::ImportSectionLabel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImportModalFocus {
    None,
    Browse,
    Target,
    Cancel,
    Confirm,
}

impl ImportModalFocus {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::None => Self::Browse,
            Self::Browse => Self::Target,
            Self::Target => Self::Cancel,
            Self::Cancel => Self::Confirm,
            Self::Confirm => Self::Browse,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::None => Self::Confirm,
            Self::Browse => Self::Confirm,
            Self::Target => Self::Browse,
            Self::Cancel => Self::Target,
            Self::Confirm => Self::Cancel,
        }
    }
}
