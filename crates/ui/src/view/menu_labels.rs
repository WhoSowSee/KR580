use crate::i18n::Key;

pub(super) fn inactive_category_keys() -> [Key; 0] {
    []
}

pub(super) fn settings_category_key() -> Key {
    Key::MenuSettings
}

#[cfg(test)]
mod tests {
    use super::{inactive_category_keys, settings_category_key};
    use crate::i18n::Lang;

    #[test]
    fn inactive_menu_categories_are_empty() {
        assert!(inactive_category_keys().is_empty());
    }

    #[test]
    fn settings_category_translates() {
        assert_eq!(Lang::Ru.t(settings_category_key()), "Настройки");
        assert_eq!(Lang::En.t(settings_category_key()), "Settings");
    }
}
