//! Shared bidirectional dictionary for driver-supplied labels.

const TRANSLATIONS: &[(&str, &str)] = &[
    ("Авто", "Auto"),
    ("Автовыбор", "Auto select"),
    ("Автоматический выбор", "Auto select"),
    ("Автовыбор", "Automatically Select"),
    ("Вручную", "Manual"),
    ("Ручная подача", "Manual feed"),
    ("Лоток ручной подачи", "Manual feed"),
    ("Включено", "On"),
    ("Вкл.", "On"),
    ("Выключено", "Off"),
    ("Выкл.", "Off"),
    ("Да", "Yes"),
    ("Нет", "None"),
    ("Стандартный", "Standard"),
    ("Высокое разрешение", "High resolution"),
    ("Обычный", "Normal"),
    ("Оттенки серого", "Grayscale"),
    ("По копиям", "Collated"),
    ("По страницам", "Uncollated"),
    ("Минимум", "Minimum"),
    ("Максимум", "Maximum"),
    ("Односторонняя", "One-sided"),
    ("По длинной кромке", "Long-edge binding"),
    ("По короткой кромке", "Short-edge binding"),
    ("Многоцелевой лоток", "Multipurpose tray"),
    ("Универсальный лоток", "Multipurpose tray"),
    ("Обходной лоток", "Bypass tray"),
    ("Основной лоток", "Main tray"),
    ("Верхний лоток", "Upper tray"),
    ("Нижний лоток", "Lower tray"),
    ("Средний лоток", "Middle tray"),
    ("Ручная подача конвертов", "Manual envelope feed"),
    ("Конверт", "Envelope"),
    ("Тракторная подача", "Tractor feed"),
    ("Малый формат", "Small format"),
    ("Большой формат", "Large format"),
    ("Лоток большой ёмкости", "Large capacity"),
    ("Кассета", "Cassette"),
    ("Источник формы", "Form source"),
    ("Высота 1", "Altitude 1"),
    ("Высота 2", "Altitude 2"),
    ("Высота 3", "Altitude 3"),
    ("Брошюра", "Booklet"),
    ("Сплошная линия", "Solid line"),
    ("Штриховая линия", "Dashed line"),
    ("Пунктирная линия", "Dotted line"),
    ("Штрихпунктирная линия", "Dash-dot line"),
    ("Двойная штрихпунктирная линия", "Double dash-dot line"),
    ("Объёмная линия", "3D line"),
    ("Прозрачная", "Transparent"),
    ("Двойная сплошная линия", "Double solid line"),
    ("Метки обрезки", "Crop marks"),
    ("Угловые метки", "Corner marks"),
    ("Пользовательский масштаб", "Custom scale"),
    ("По размеру страницы", "Fit to page"),
    ("Снизу по центру", "Bottom center"),
    ("По центру", "Center"),
    ("Слева по центру", "Center left"),
    ("Справа по центру", "Center right"),
    ("Сверху по центру", "Top center"),
    ("Сверху слева", "Top left"),
    ("Сверху справа", "Top right"),
    ("Альбомная", "Landscape"),
    ("Книжная", "Portrait"),
    ("Обратная альбомная", "Reverse landscape"),
    ("Обратная книжная", "Reverse portrait"),
    ("Использовать наложение", "Use overlay"),
    ("Создать наложение", "Create overlay"),
    ("Все страницы", "All pages"),
    ("Первая страница", "First page"),
    ("Только первая страница", "First page only"),
    ("Все, кроме первой", "All except first page"),
    ("Нечётные страницы", "Odd pages"),
    ("Чётные страницы", "Even pages"),
    ("Лицевые стороны", "Front pages"),
    ("Обратные стороны", "Back pages"),
    ("Пользовательский размер", "Custom size"),
    ("Пользовательский", "Custom size"),
    ("Открытка", "Postcard"),
    ("Двойная открытка", "Double postcard"),
    ("Миллиметры", "Millimeters"),
    ("Дюймы", "Inches"),
    ("Значение разрешения", "Resolution value"),
    ("Текст", "Text"),
    ("Под содержимым", "Below content"),
    ("Над содержимым", "Above content"),
    ("Дата", "Date"),
    ("Имя пользователя", "User name"),
    ("Имя учётной записи задания", "Job account name"),
    ("Шаблон изображения", "Image template"),
    ("Изображение", "Image"),
    ("Сверху", "Top"),
    ("Слева", "Left"),
    ("Справа", "Right"),
    ("Снизу слева", "Bottom left"),
    ("Снизу", "Bottom"),
    ("Снизу справа", "Bottom right"),
    ("Параметры", "Options"),
    ("Короткая дата", "Short date"),
    ("Полная дата", "Long date"),
    ("Короткое время", "Short time"),
    ("Полное время", "Long time"),
    ("Номер страницы", "Page number"),
    ("Имя компьютера", "Computer name"),
];

pub(super) fn has_cyrillic(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '\u{0400}'..='\u{052F}'))
}

pub(super) fn english(label: &str) -> Option<String> {
    let label = label.trim();
    TRANSLATIONS
        .iter()
        .find_map(|(ru, en)| (*ru == label).then(|| (*en).to_owned()))
        .or_else(|| numbered_english(label))
}

pub(super) fn russian(label: &str) -> Option<String> {
    let label = label.trim();
    TRANSLATIONS
        .iter()
        .find_map(|(ru, en)| en.eq_ignore_ascii_case(label).then(|| (*ru).to_owned()))
        .or_else(|| numbered_russian(label))
}

fn numbered_english(label: &str) -> Option<String> {
    if let Some(suffix) = label.strip_prefix("Лоток ")
        && !has_cyrillic(suffix)
    {
        return Some(format!("Tray {suffix}"));
    }
    if let Some(suffix) = label.strip_prefix("Конверт ")
        && !has_cyrillic(suffix)
    {
        return Some(format!("{suffix} envelope"));
    }
    None
}

fn numbered_russian(label: &str) -> Option<String> {
    if let Some(suffix) = label.strip_prefix("Tray ") {
        return Some(format!("Лоток {suffix}"));
    }
    if let Some(suffix) = label.strip_suffix(" envelope") {
        return Some(format!("Конверт {suffix}"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dictionary_covers_both_driver_locales() {
        assert_eq!(english("Автовыбор").as_deref(), Some("Auto select"));
        assert_eq!(english(" Автовыбор ").as_deref(), Some("Auto select"));
        assert_eq!(russian("Auto select").as_deref(), Some("Автовыбор"));
        assert_eq!(english("Лоток 1").as_deref(), Some("Tray 1"));
        assert_eq!(russian("Tray 1").as_deref(), Some("Лоток 1"));
        assert_eq!(english("Конверт DL").as_deref(), Some("DL envelope"));
        assert_eq!(russian("DL envelope").as_deref(), Some("Конверт DL"));
        assert_eq!(english("Нестандартная подача"), None);
        assert_eq!(russian("Custom feeder"), None);
    }

    #[test]
    fn numbered_labels_reject_a_cyrillic_suffix() {
        assert_eq!(english("Конверт С5"), None);
        assert_eq!(
            english("Лоток ручной подачи").as_deref(),
            Some("Manual feed")
        );
        assert_eq!(english("Лоток верхний"), None);
    }

    #[test]
    fn cyrillic_detection_covers_the_supplement_block() {
        assert!(has_cyrillic("Лоток"));
        assert!(has_cyrillic("\u{0510}"));
        assert!(!has_cyrillic("Tray 1"));
    }
}
