use crate::persistence::Language;

pub fn default_language() -> Language {
    if system_locale_is_russian() {
        Language::Ru
    } else {
        Language::En
    }
}

#[cfg(windows)]
fn system_locale_is_russian() -> bool {
    const LANG_RUSSIAN: u16 = 0x19;
    const PRIMARY_LANGUAGE_MASK: u16 = 0x03ff;
    // SAFETY: GetUserDefaultUILanguage has no preconditions and only reads the user UI language.
    let langid = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    langid & PRIMARY_LANGUAGE_MASK == LANG_RUSSIAN
}

#[cfg(not(windows))]
fn system_locale_is_russian() -> bool {
    ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
        .into_iter()
        .filter_map(std::env::var_os)
        .any(|value| locale_tag_is_russian(&value.to_string_lossy()))
}

#[cfg(not(windows))]
fn locale_tag_is_russian(value: &str) -> bool {
    value.split(':').any(|part| {
        let tag = part.trim().to_ascii_lowercase();
        tag == "ru" || tag.starts_with("ru_") || tag.starts_with("ru-") || tag.starts_with("ru.")
    })
}

#[cfg(test)]
mod tests {
    #[cfg(not(windows))]
    use super::locale_tag_is_russian;

    #[cfg(not(windows))]
    #[test]
    fn locale_tag_detects_only_russian() {
        assert!(locale_tag_is_russian("ru_RU.UTF-8"));
        assert!(locale_tag_is_russian("en_US:ru_RU"));
        assert!(!locale_tag_is_russian("en_US.UTF-8"));
        assert!(!locale_tag_is_russian("uk_UA.UTF-8"));
    }
}
