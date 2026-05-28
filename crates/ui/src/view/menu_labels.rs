pub(super) fn inactive_category_labels() -> [&'static str; 3] {
    ["Вид", "Настройки", "Справка"]
}

#[cfg(test)]
mod tests {
    use super::inactive_category_labels;

    #[test]
    fn inactive_menu_categories_are_localized() {
        assert_eq!(inactive_category_labels(), ["Вид", "Настройки", "Справка"]);
    }
}
