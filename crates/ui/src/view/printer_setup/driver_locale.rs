//! Single Russian-to-English dictionary for driver-supplied labels.
//!
//! Windows reports paper sizes, input bins, PrintTicket features, and
//! option names in the printer driver's locale. Top-level Setup and the
//! Properties dialog both have to translate those strings, so the table
//! and the Cyrillic test live here once instead of once per dialog.

pub(super) fn has_cyrillic(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '\u{0400}'..='\u{052F}'))
}

pub(super) fn english(label: &str) -> Option<String> {
    let label = label.trim();
    translated(label)
        .map(str::to_owned)
        .or_else(|| numbered(label))
}

fn translated(label: &str) -> Option<&'static str> {
    Some(match label {
        "Авто" => "Auto",
        "Автовыбор" | "Автоматический выбор" => "Auto select",
        "Вручную" => "Manual",
        "Ручная подача" | "Лоток ручной подачи" => "Manual feed",
        "Вкл." | "Включено" => "On",
        "Выкл." | "Выключено" => "Off",
        "Да" => "Yes",
        "Нет" => "None",
        "Стандартный" => "Standard",
        "Высокое разрешение" => "High resolution",
        "Обычный" => "Normal",
        "Оттенки серого" => "Grayscale",
        "По копиям" => "Collated",
        "По страницам" => "Uncollated",
        "Минимум" => "Minimum",
        "Максимум" => "Maximum",
        "Односторонняя" => "One-sided",
        "По длинной кромке" => "Long-edge binding",
        "По короткой кромке" => "Short-edge binding",
        "Универсальный лоток" | "Многоцелевой лоток" => {
            "Multipurpose tray"
        }
        "Обходной лоток" => "Bypass tray",
        "Основной лоток" => "Main tray",
        "Высота 1" => "Altitude 1",
        "Высота 2" => "Altitude 2",
        "Высота 3" => "Altitude 3",
        "Брошюра" => "Booklet",
        "Сплошная линия" => "Solid line",
        "Штриховая линия" => "Dashed line",
        "Пунктирная линия" => "Dotted line",
        "Штрихпунктирная линия" => "Dash-dot line",
        "Двойная штрихпунктирная линия" => "Double dash-dot line",
        "Объёмная линия" => "3D line",
        "Прозрачная" => "Transparent",
        "Двойная сплошная линия" => "Double solid line",
        "Метки обрезки" => "Crop marks",
        "Угловые метки" => "Corner marks",
        "Пользовательский масштаб" => "Custom scale",
        "По размеру страницы" => "Fit to page",
        "Снизу по центру" => "Bottom center",
        "По центру" => "Center",
        "Слева по центру" => "Center left",
        "Справа по центру" => "Center right",
        "Сверху по центру" => "Top center",
        "Сверху слева" => "Top left",
        "Сверху справа" => "Top right",
        "Альбомная" => "Landscape",
        "Книжная" => "Portrait",
        "Обратная альбомная" => "Reverse landscape",
        "Обратная книжная" => "Reverse portrait",
        "Использовать наложение" => "Use overlay",
        "Создать наложение" => "Create overlay",
        "Все страницы" => "All pages",
        "Первая страница" => "First page",
        "Только первая страница" => "First page only",
        "Все, кроме первой" => "All except first page",
        "Нечётные страницы" => "Odd pages",
        "Чётные страницы" => "Even pages",
        "Лицевые стороны" => "Front pages",
        "Обратные стороны" => "Back pages",
        "Пользовательский" | "Пользовательский размер" => {
            "Custom size"
        }
        "Открытка" => "Postcard",
        "Двойная открытка" => "Double postcard",
        "Миллиметры" => "Millimeters",
        "Дюймы" => "Inches",
        "Значение разрешения" => "Resolution value",
        "Текст" => "Text",
        "Под содержимым" => "Below content",
        "Над содержимым" => "Above content",
        "Дата" => "Date",
        "Имя пользователя" => "User name",
        "Имя учётной записи задания" => "Job account name",
        "Шаблон изображения" => "Image template",
        "Изображение" => "Image",
        "Сверху" => "Top",
        "Слева" => "Left",
        "Справа" => "Right",
        "Снизу слева" => "Bottom left",
        "Снизу" => "Bottom",
        "Снизу справа" => "Bottom right",
        "Параметры" => "Options",
        "Короткая дата" => "Short date",
        "Полная дата" => "Long date",
        "Короткое время" => "Short time",
        "Полное время" => "Long time",
        "Номер страницы" => "Page number",
        "Имя компьютера" => "Computer name",
        _ => return None,
    })
}

fn numbered(label: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dictionary_covers_both_dialog_vocabularies() {
        assert_eq!(english("Автовыбор").as_deref(), Some("Auto select"));
        assert_eq!(english(" Автовыбор ").as_deref(), Some("Auto select"));
        assert_eq!(
            english("Универсальный лоток").as_deref(),
            Some("Multipurpose tray")
        );
        assert_eq!(english("Лоток 1").as_deref(), Some("Tray 1"));
        assert_eq!(english("Конверт DL").as_deref(), Some("DL envelope"));
        assert_eq!(
            english("Лоток ручной подачи").as_deref(),
            Some("Manual feed")
        );
        assert_eq!(english("Нестандартная подача"), None);
        assert_eq!(english("Tray 1"), None);
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
