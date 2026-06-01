use crate::i18n::Key;

pub(super) fn inactive_category_keys() -> [Key; 1] {
    [Key::MenuView]
}

pub(super) fn settings_category_key() -> Key {
    Key::MenuSettings
}

#[cfg(test)]
mod tests {
    use super::{inactive_category_keys, settings_category_key};
    use crate::i18n::{Key, Lang};

    #[test]
    fn inactive_menu_categories_are_localized() {
        let keys = inactive_category_keys();
        assert_eq!(keys, [Key::MenuView]);
        assert_eq!(Lang::Ru.t(keys[0]), "Вид");
        assert_eq!(Lang::En.t(keys[0]), "View");
    }

    #[test]
    fn settings_category_translates() {
        assert_eq!(Lang::Ru.t(settings_category_key()), "Настройки");
        assert_eq!(Lang::En.t(settings_category_key()), "Settings");
    }
}
