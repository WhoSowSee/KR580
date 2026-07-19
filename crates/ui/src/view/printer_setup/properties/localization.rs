use crate::i18n::Lang;
use k580_ui::devices::printer::{PrinterFeature, PrinterFeatureOption, PrinterParameter};

#[cfg(test)]
mod tests;

pub(super) fn feature_label(feature: &PrinterFeature, lang: Lang) -> String {
    let local = local_name(&feature.name);
    match lang {
        Lang::Ru => ru_feature(local)
            .map(str::to_owned)
            .or_else(|| has_cyrillic(&feature.display_name).then(|| feature.display_name.clone()))
            .unwrap_or_else(|| humanize_ru(local)),
        Lang::En => en_feature(local)
            .map(str::to_owned)
            .unwrap_or_else(|| feature.display_name.clone()),
    }
}

pub(super) fn localized_options(feature: &PrinterFeature, lang: Lang) -> Vec<PrinterFeatureOption> {
    feature
        .options
        .iter()
        .filter(|option| {
            !option.constrained || feature.selected_option.as_deref() == Some(&option.name)
        })
        .cloned()
        .map(|mut option| {
            option.display_name = option_label(feature, &option, lang);
            option
        })
        .collect()
}

pub(super) fn parameter_label(parameter: &PrinterParameter, lang: Lang) -> String {
    let local = local_name(&parameter.name);
    match lang {
        Lang::Ru => ru_parameter(local)
            .map(str::to_owned)
            .or_else(|| {
                has_cyrillic(&parameter.display_name).then(|| parameter.display_name.clone())
            })
            .unwrap_or_else(|| humanize_ru(local)),
        Lang::En => parameter.display_name.clone(),
    }
}

pub(super) fn parameter_visible(parameter: &PrinterParameter) -> bool {
    local_name(&parameter.name) != "PageDevmodeSnapshot"
}

fn option_label(feature: &PrinterFeature, option: &PrinterFeatureOption, lang: Lang) -> String {
    if lang == Lang::En {
        return option.display_name.clone();
    }
    let feature = local_name(&feature.name);
    let option_name = local_name(&option.name);
    if feature == "JobPageOrder" {
        match option_name {
            "Standard" => return "Обычный (1, 2, 3)".to_owned(),
            "Reverse" => return "Обратный (3, 2, 1)".to_owned(),
            _ => {}
        }
    }
    if feature == "PresentationDirection" {
        match option_name {
            "RightBottom" => return "Слева направо, затем вниз".to_owned(),
            "BottomRight" => return "Сверху вниз, затем вправо".to_owned(),
            "LeftBottom" => return "Справа налево, затем вниз".to_owned(),
            "BottomLeft" => return "Сверху вниз, затем влево".to_owned(),
            "RightTop" => return "Слева направо, затем вверх".to_owned(),
            _ => {}
        }
    }
    if let Some(translated) = ru_option(option_name) {
        return translated.to_owned();
    }
    if has_cyrillic(&option.display_name)
        || option_name.starts_with('k')
            && option_name[1..]
                .chars()
                .all(|character| character.is_ascii_digit())
    {
        return option.display_name.clone();
    }
    humanize_ru(option_name)
}

fn ru_feature(name: &str) -> Option<&'static str> {
    Some(match name {
        "DocumentColorAdjust" => "Коррекция цвета",
        "UsePreferredColor" => "Предпочтительные цвета",
        "PageOutputColor" => "Цветовой режим",
        "DocumentBlackOptimization" => "Оптимизация чёрного",
        "DocumentFirstPageColorOnly" => "Цветная только первая страница",
        "DocumentCmykColorPreservation" => "Сохранение цветов CMYK",
        "DocumentCollate" => "Разобрать по копиям",
        "DocumentDarkenText" => "Чёткий текст",
        "JobDuplexPrinterDefault" => "Двусторонняя печать по умолчанию",
        "JobDuplexAllDocumentsContiguously" => "Двусторонняя печать",
        "DocumentDuplexReverse" => "Обратная сторона при двусторонней печати",
        "DocumentDuplexIgnoreOrientation" => "Игнорировать ориентацию при двусторонней печати",
        "DocumentPreventPopup" => "Не показывать уведомления драйвера",
        "DocumentFirstPageInputBin" => "Лоток первой страницы",
        "JobHighAltitude" => "Поправка на высоту",
        "DocumentBinding" => "Переплёт",
        "DocumentNUp" => "Страниц на листе",
        "PresentationDirection" => "Порядок страниц на листе",
        "PageBorder" => "Рамка страницы",
        "PageScaling" => "Масштабирование",
        "ScaleOffsetAlignment" => "Выравнивание при масштабировании",
        "PageOrientation" => "Ориентация",
        "DocumentOverlay" | "DocumentDeviceOverlay" => "Наложение",
        "ConfirmPrintOverlay" => "Подтверждать печать наложения",
        "PageRange1" => "Первый диапазон страниц",
        "PageRange2" => "Второй диапазон страниц",
        "PageMediaSize" => "Размер бумаги",
        "PageDefaultSource" | "PageInputBin" => "Источник бумаги",
        "PageMediaType" => "Тип бумаги",
        "JobPreviewLayout" => "Единицы предпросмотра",
        "DocumentAllTextToBlack" => "Печатать весь текст чёрным",
        "JobPageOrder" => "Порядок страниц",
        "PageResolution" | "PageIResolution" => "Качество печати",
        "DocumentTonerSave" => "Экономия тонера",
        "PageWatermark" => "Водяной знак",
        "TransparencyType" => "Расположение водяного знака",
        "Layering" => "Слой водяного знака",
        "PageApply" => "Область применения",
        "DrawPerSide" => "Рисовать на каждой стороне",
        "Type" => "Тип содержимого",
        "Alignment" => "Расположение",
        "HeaderFooter" => "Колонтитулы",
        "LeftHeader" => "Левый верхний колонтитул",
        "CenterHeader" => "Центральный верхний колонтитул",
        "RightHeader" => "Правый верхний колонтитул",
        "LeftFooter" => "Левый нижний колонтитул",
        "CenterFooter" => "Центральный нижний колонтитул",
        "RightFooter" => "Правый нижний колонтитул",
        "DocumentSkipBlankPages" => "Пропускать пустые страницы",
        "DocumentFineEdge" => "Усиление контуров",
        "JobAutoConfiguration" => "Автоматическая конфигурация",
        _ => return None,
    })
}

fn en_feature(name: &str) -> Option<&'static str> {
    Some(match name {
        "DocumentColorAdjust" => "Color adjustment",
        "UsePreferredColor" => "Preferred colors",
        "PageOutputColor" => "Color mode",
        "DocumentBlackOptimization" => "Black optimization",
        "DocumentFirstPageColorOnly" => "Color first page only",
        "DocumentCmykColorPreservation" => "Preserve CMYK colors",
        "DocumentDarkenText" => "Clear text",
        "DocumentAllTextToBlack" => "Print all text in black",
        "DocumentFineEdge" => "Edge enhancement",
        "DocumentTonerSave" => "Toner saving",
        "JobPageOrder" => "Page order",
        "DocumentSkipBlankPages" => "Skip blank pages",
        "PageWatermark" => "Watermark",
        "DocumentNUp" => "Pages per sheet",
        _ => return None,
    })
}

fn ru_option(name: &str) -> Option<&'static str> {
    Some(match name {
        "Auto" | "AUTO" => "Авто",
        "Manual" => "Вручную",
        "On" => "Вкл.",
        "Off" | "OFF" => "Выкл.",
        "None" | "NoOverlay" => "Нет",
        "Grayscale" => "Оттенки серого",
        "Collated" => "По копиям",
        "Uncollated" => "По страницам",
        "Minimum" => "Минимум",
        "Maximum" => "Максимум",
        "OneSided" => "Односторонняя",
        "TwoSidedLongEdge" => "По длинной кромке",
        "TwoSidedShortEdge" => "По короткой кромке",
        "MPTray" => "Универсальный лоток",
        "Normal" => "Обычный",
        "HIGH_a32_1" => "Высота 1",
        "HIGH_a32_2" => "Высота 2",
        "HIGH_a32_3" => "Высота 3",
        "Booklet" => "Брошюра",
        "SolidLine" => "Сплошная линия",
        "DashedLine" => "Штриховая линия",
        "DottedLine" => "Пунктирная линия",
        "ChainLine" => "Штрихпунктирная линия",
        "ChainDoubleDashedLine" => "Двойная штрихпунктирная линия",
        "3DimensionalLine" => "Объёмная линия",
        "Transparent" => "Прозрачная",
        "DoubleSolidLine" => "Двойная сплошная линия",
        "CropMarks" => "Метки обрезки",
        "CornerMarks" => "Угловые метки",
        "CustomSquare" => "Пользовательский масштаб",
        "FitToPage" => "По размеру страницы",
        "BottomCenter" => "Снизу по центру",
        "Center" => "По центру",
        "LeftCenter" => "Слева по центру",
        "RightCenter" => "Справа по центру",
        "TopCenter" => "Сверху по центру",
        "TopLeft" => "Сверху слева",
        "TopRight" => "Сверху справа",
        "Landscape" => "Альбомная",
        "Portrait" => "Книжная",
        "ReverseLandscape" => "Обратная альбомная",
        "ReversePortrait" => "Обратная книжная",
        "LoadOverlay" => "Использовать наложение",
        "CreateOverlay" => "Создать наложение",
        "AllPages" | "AllPage" => "Все страницы",
        "FirstPage" => "Первая страница",
        "FirstPageOnly" => "Только первая страница",
        "AllExceptFirstPages" => "Все, кроме первой",
        "OddPages" => "Нечётные страницы",
        "EvenPages" => "Чётные страницы",
        "FrontPages" => "Лицевые стороны",
        "BackPages" => "Обратные стороны",
        "CustomMediaSize" => "Пользовательский размер",
        "Millimeter" => "Миллиметры",
        "Inch" => "Дюймы",
        "PageIResolutionValue" => "Значение разрешения",
        "Text" => "Текст",
        "Underlying" | "Underlay" => "Под содержимым",
        "Floating" | "Overlay" => "Над содержимым",
        "Date" => "Дата",
        "LogonName" => "Имя пользователя",
        "JobAccountingName" => "Имя учётной записи задания",
        "ImageTemplate" => "Шаблон изображения",
        "Image" => "Изображение",
        "Top" => "Сверху",
        "Left" => "Слева",
        "Right" => "Справа",
        "BottomLeft" => "Снизу слева",
        "Bottom" => "Снизу",
        "BottomRight" => "Снизу справа",
        "Options" => "Параметры",
        "ShortDate" => "Короткая дата",
        "LongDate" => "Полная дата",
        "ShortTime" => "Короткое время",
        "LongTime" => "Полное время",
        "PageNumber" => "Номер страницы",
        "ComputerName" => "Имя компьютера",
        _ => return None,
    })
}

fn ru_parameter(name: &str) -> Option<&'static str> {
    Some(match name {
        "DocumentColorAdjustBrightness" => "Яркость",
        "DocumentColorAdjustContrast" => "Контрастность",
        "DocumentColorAdjustSaturation" => "Насыщенность",
        "DocumentColorAdjustRedValue" => "Красный",
        "DocumentColorAdjustGreenValue" => "Зелёный",
        "DocumentColorAdjustBlueValue" => "Синий",
        "DocumentColorAdjustPreferredColorSkin" => "Оттенки кожи",
        "DocumentColorAdjustPreferredColorGrass" => "Оттенки травы",
        "DocumentColorAdjustPreferredColorSky" => "Оттенки неба",
        "JobCopiesAllDocuments" => "Количество копий",
        "PagePosterOverlapValue" => "Перекрытие частей плаката",
        "PageScalingOffsetWidth" => "Смещение по горизонтали",
        "PageScalingOffsetHeight" => "Смещение по вертикали",
        "PageScalingScale" => "Масштаб",
        "PageScalingTargetMediaSizeId" => "Код целевого размера бумаги",
        "PageScalingTargetMediaSizeWidth" => "Ширина целевой бумаги",
        "PageScalingTargetMediaSizeHeight" => "Высота целевой бумаги",
        "PageScalingTargetMediaSizeName" => "Целевой размер бумаги",
        "PageScalingTargetMediaSizeXOffset" => "Горизонтальное поле",
        "PageScalingTargetMediaSizeYOffset" => "Вертикальное поле",
        "DocumentOverlayOverlayPath" => "Файл наложения",
        "PageMediaSizeMediaSizeWidth" => "Пользовательская ширина бумаги",
        "PageMediaSizeMediaSizeHeight" => "Пользовательская высота бумаги",
        "PageIResolutionIResolution" => "Разрешение изображения",
        "PageIResolutionBPP" => "Глубина цвета",
        "PageIResolutionImageQuality" => "Качество изображения",
        "PageWatermarkTextColor" => "Цвет водяного знака",
        "PageWatermarkTextFontSize" => "Размер шрифта водяного знака",
        "PageWatermarkTextText" => "Текст водяного знака",
        "PageWatermarkTextAngle" => "Угол водяного знака",
        "PageWatermarkTextFontFace" => "Шрифт водяного знака",
        "PageWatermarkTextFontCharset" => "Кодировка шрифта",
        "PageWatermarkTextFontWeight" => "Насыщенность шрифта",
        "PageWatermarkTextFontItalic" => "Курсив",
        "PageWatermarkTextWatermarkName" => "Название водяного знака",
        "PageWatermarkTextImageScale" => "Масштаб водяного знака",
        "PageWatermarkTextTransparencyLevel" => "Прозрачность водяного знака",
        "PageHeaderFooterOptionsFontFace" => "Шрифт колонтитулов",
        "PageHeaderFooterOptionsFontSize" => "Размер шрифта колонтитулов",
        "PageHeaderFooterOptionsFontColor" => "Цвет колонтитулов",
        "PageHeaderFooterOptionsFontCharset" => "Кодировка колонтитулов",
        _ => return None,
    })
}

fn humanize_ru(value: &str) -> String {
    split_words(value)
        .into_iter()
        .map(|word| translate_word(&word).unwrap_or(word.as_str()).to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for character in value.replace(['_', '-'], " ").chars() {
        let boundary = character.is_uppercase()
            && current
                .chars()
                .last()
                .is_some_and(|previous| previous.is_lowercase() || previous.is_ascii_digit());
        if character.is_whitespace() || boundary {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            if character.is_whitespace() {
                continue;
            }
        }
        current.push(character);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn translate_word(word: &str) -> Option<&'static str> {
    Some(match word.to_ascii_lowercase().as_str() {
        "document" => "Документ",
        "job" => "Задание",
        "page" => "Страница",
        "color" => "Цвет",
        "adjust" => "Настройка",
        "preferred" => "Предпочтительный",
        "black" => "Чёрный",
        "first" => "Первая",
        "only" => "Только",
        "printer" => "Принтер",
        "default" => "По умолчанию",
        "duplex" => "Двусторонняя печать",
        "reverse" => "Обратный",
        "orientation" => "Ориентация",
        "input" => "Подача",
        "bin" => "Лоток",
        "binding" => "Переплёт",
        "border" => "Рамка",
        "scaling" => "Масштабирование",
        "offset" => "Смещение",
        "alignment" => "Выравнивание",
        "overlay" => "Наложение",
        "media" => "Бумага",
        "size" => "Размер",
        "source" => "Источник",
        "type" => "Тип",
        "preview" => "Предпросмотр",
        "layout" => "Разметка",
        "text" => "Текст",
        "order" => "Порядок",
        "resolution" => "Разрешение",
        "watermark" => "Водяной знак",
        "header" => "Верхний колонтитул",
        "footer" => "Нижний колонтитул",
        "left" => "Левый",
        "center" => "Центральный",
        "right" => "Правый",
        _ => return None,
    })
}

fn has_cyrillic(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '\u{0400}'..='\u{04FF}'))
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}
