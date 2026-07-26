use crate::i18n::Lang;

pub(super) fn contextual(feature: &str, name: &str, lang: Lang) -> Option<&'static str> {
    let translations = match (feature, name) {
        ("JobPageOrder", "Standard") => ("Обычный (1, 2, 3)", "Standard (1, 2, 3)"),
        ("JobPageOrder", "Reverse") => ("Обратный (3, 2, 1)", "Reverse (3, 2, 1)"),
        ("PresentationDirection", "RightBottom") => {
            ("Слева направо, затем вниз", "Left to right, then down")
        }
        ("PresentationDirection", "BottomRight") => {
            ("Сверху вниз, затем вправо", "Top to bottom, then right")
        }
        ("PresentationDirection", "LeftBottom") => {
            ("Справа налево, затем вниз", "Right to left, then down")
        }
        ("PresentationDirection", "BottomLeft") => {
            ("Сверху вниз, затем влево", "Top to bottom, then left")
        }
        ("PresentationDirection", "RightTop") => {
            ("Слева направо, затем вверх", "Left to right, then up")
        }
        ("PageDefaultSource" | "PageInputBin", "AUTO") => ("Автовыбор", "Auto select"),
        ("PageMediaType", "OFF") => ("Не указано", "Unspecified"),
        ("PageMediaType", "NORMAL") => ("Обычная бумага", "Plain paper"),
        ("PageMediaType", "THICK") => ("Плотная бумага (90–120 г/м²)", "Thick paper (90–120 g/m²)"),
        ("PageMediaType", "THIN") => ("Тонкая бумага (60–69 г/м²)", "Thin paper (60–69 g/m²)"),
        ("PageMediaType", "BOND") => ("Документная бумага", "Bond paper"),
        ("PageMediaType", "COLOR") => ("Цветная бумага", "Colored paper"),
        ("PageMediaType", "CARD") => ("Картон (121–163 г/м²)", "Cardstock (121–163 g/m²)"),
        ("PageMediaType", "LABEL") => ("Наклейки", "Labels"),
        ("PageMediaType", "ENV") => ("Конверт", "Envelope"),
        ("PageMediaType", "USED") => ("Бланки", "Preprinted paper"),
        ("PageMediaType", "RECYCLED") => ("Переработанная бумага", "Recycled paper"),
        ("PageResolution", "1_600x600_dpi") => ("Стандартное", "Standard"),
        ("PageResolution", "0_1200x1200_dpi") => ("Высокое разрешение", "High resolution"),
        _ => return None,
    };
    Some(pick(lang, translations))
}

pub(super) fn feature(name: &str, lang: Lang) -> Option<&'static str> {
    let translations = match name {
        "DocumentColorAdjust" => ("Коррекция цвета", "Color adjustment"),
        "UsePreferredColor" => ("Предпочтительные цвета", "Preferred colors"),
        "PageOutputColor" => ("Цветовой режим", "Color mode"),
        "DocumentBlackOptimization" => ("Оптимизация чёрного", "Black optimization"),
        "DocumentFirstPageColorOnly" => ("Цветная только первая страница", "Color first page only"),
        "DocumentCmykColorPreservation" => ("Сохранение цветов CMYK", "Preserve CMYK colors"),
        "DocumentCollate" => ("Разобрать по копиям", "Collate"),
        "DocumentDarkenText" => ("Чёткий текст", "Clear text"),
        "JobDuplexPrinterDefault" => ("Двусторонняя печать по умолчанию", "Duplex printer default"),
        "JobDuplexAllDocumentsContiguously" => ("Двусторонняя печать", "Double-sided printing"),
        "DocumentDuplexReverse" => (
            "Обратная сторона при двусторонней печати",
            "Reverse duplex printing",
        ),
        "DocumentDuplexIgnoreOrientation" => (
            "Игнорировать ориентацию при двусторонней печати",
            "Ignore orientation for duplex printing",
        ),
        "DocumentPreventPopup" => (
            "Не показывать уведомления драйвера",
            "Suppress driver popups",
        ),
        "DocumentFirstPageInputBin" => ("Лоток первой страницы", "First-page paper source"),
        "JobHighAltitude" => ("Поправка на высоту", "Altitude correction"),
        "DocumentBinding" => ("Переплёт", "Binding"),
        "DocumentNUp" => ("Страниц на листе", "Pages per sheet"),
        "PresentationDirection" => ("Порядок страниц на листе", "Presentation direction"),
        "PageBorder" => ("Рамка страницы", "Page border"),
        "PageScaling" | "ScalingOptions" => ("Масштабирование", "Scaling"),
        "ScaleOffsetAlignment" => ("Выравнивание при масштабировании", "Scale offset alignment"),
        "PageOrientation" => ("Ориентация", "Orientation"),
        "DocumentOverlay" | "DocumentDeviceOverlay" => ("Наложение", "Overlay"),
        "ConfirmPrintOverlay" => ("Подтверждать печать наложения", "Confirm overlay printing"),
        "PageRange1" => ("Первый диапазон страниц", "Page range 1"),
        "PageRange2" => ("Второй диапазон страниц", "Page range 2"),
        "PageMediaSize" => ("Размер бумаги", "Paper size"),
        "PageDefaultSource" | "PageInputBin" => ("Источник бумаги", "Paper source"),
        "PageMediaType" => ("Тип бумаги", "Paper type"),
        "JobPreviewLayout" => ("Единицы предпросмотра", "Preview units"),
        "DocumentAllTextToBlack" => ("Печатать весь текст чёрным", "Print all text in black"),
        "JobPageOrder" => ("Порядок страниц", "Page order"),
        "PageResolution" | "PageIResolution" => ("Качество печати", "Print quality"),
        "DocumentTonerSave" => ("Экономия тонера", "Toner saving"),
        "PageWatermark" => ("Водяной знак", "Watermark"),
        "TransparencyType" => ("Расположение водяного знака", "Watermark placement"),
        "Layering" => ("Слой водяного знака", "Watermark layer"),
        "PageApply" => ("Область применения", "Apply to"),
        "DrawPerSide" => ("Рисовать на каждой стороне", "Draw on each side"),
        "Type" => ("Тип содержимого", "Content type"),
        "Alignment" => ("Расположение", "Alignment"),
        "HeaderFooter" => ("Колонтитулы", "Headers and footers"),
        "LeftHeader" => ("Левый верхний колонтитул", "Left header"),
        "CenterHeader" => ("Центральный верхний колонтитул", "Center header"),
        "RightHeader" => ("Правый верхний колонтитул", "Right header"),
        "LeftFooter" => ("Левый нижний колонтитул", "Left footer"),
        "CenterFooter" => ("Центральный нижний колонтитул", "Center footer"),
        "RightFooter" => ("Правый нижний колонтитул", "Right footer"),
        "DocumentSkipBlankPages" => ("Пропускать пустые страницы", "Skip blank pages"),
        "DocumentFineEdge" => ("Усиление контуров", "Edge enhancement"),
        "JobAutoConfiguration" => ("Автоматическая конфигурация", "Automatic configuration"),
        _ => return None,
    };
    Some(pick(lang, translations))
}

pub(super) fn option(name: &str, lang: Lang) -> Option<&'static str> {
    let translations = match name {
        "Auto" | "AUTO" => ("Авто", "Auto"),
        "AutoSelect" => ("Автовыбор", "Auto select"),
        "Manual" => ("Вручную", "Manual"),
        "On" => ("Вкл.", "On"),
        "Off" | "OFF" => ("Выкл.", "Off"),
        "None" | "NoOverlay" => ("Нет", "None"),
        "Grayscale" => ("Оттенки серого", "Grayscale"),
        "Collated" => ("По копиям", "Collated"),
        "Uncollated" => ("По страницам", "Uncollated"),
        "Minimum" => ("Минимум", "Minimum"),
        "Maximum" => ("Максимум", "Maximum"),
        "OneSided" => ("Односторонняя", "One-sided"),
        "TwoSidedLongEdge" => ("По длинной кромке", "Long-edge binding"),
        "TwoSidedShortEdge" => ("По короткой кромке", "Short-edge binding"),
        "MPTray" => ("Универсальный лоток", "Multipurpose tray"),
        "Normal" => ("Обычный", "Normal"),
        "HIGH_a32_1" => ("Высота 1", "Altitude 1"),
        "HIGH_a32_2" => ("Высота 2", "Altitude 2"),
        "HIGH_a32_3" => ("Высота 3", "Altitude 3"),
        "Booklet" => ("Брошюра", "Booklet"),
        "SolidLine" => ("Сплошная линия", "Solid line"),
        "DashedLine" => ("Штриховая линия", "Dashed line"),
        "DottedLine" => ("Пунктирная линия", "Dotted line"),
        "ChainLine" => ("Штрихпунктирная линия", "Dash-dot line"),
        "ChainDoubleDashedLine" => ("Двойная штрихпунктирная линия", "Double dash-dot line"),
        "3DimensionalLine" => ("Объёмная линия", "3D line"),
        "Transparent" => ("Прозрачная", "Transparent"),
        "DoubleSolidLine" => ("Двойная сплошная линия", "Double solid line"),
        "CropMarks" => ("Метки обрезки", "Crop marks"),
        "CornerMarks" => ("Угловые метки", "Corner marks"),
        "CustomSquare" => ("Пользовательский масштаб", "Custom scale"),
        "FitToPage" => ("По размеру страницы", "Fit to page"),
        "BottomCenter" => ("Снизу по центру", "Bottom center"),
        "Center" => ("По центру", "Center"),
        "LeftCenter" => ("Слева по центру", "Center left"),
        "RightCenter" => ("Справа по центру", "Center right"),
        "TopCenter" => ("Сверху по центру", "Top center"),
        "TopLeft" => ("Сверху слева", "Top left"),
        "TopRight" => ("Сверху справа", "Top right"),
        "Landscape" => ("Альбомная", "Landscape"),
        "Portrait" => ("Книжная", "Portrait"),
        "ReverseLandscape" => ("Обратная альбомная", "Reverse landscape"),
        "ReversePortrait" => ("Обратная книжная", "Reverse portrait"),
        "LoadOverlay" => ("Использовать наложение", "Use overlay"),
        "CreateOverlay" => ("Создать наложение", "Create overlay"),
        "AllPages" | "AllPage" => ("Все страницы", "All pages"),
        "FirstPage" => ("Первая страница", "First page"),
        "FirstPageOnly" => ("Только первая страница", "First page only"),
        "AllExceptFirstPages" => ("Все, кроме первой", "All except first page"),
        "OddPages" => ("Нечётные страницы", "Odd pages"),
        "EvenPages" => ("Чётные страницы", "Even pages"),
        "FrontPages" => ("Лицевые стороны", "Front pages"),
        "BackPages" => ("Обратные стороны", "Back pages"),
        "ISOC5Envelope" => ("Конверт C5", "C5 envelope"),
        "ISOC4Envelope" => ("Конверт C4", "C4 envelope"),
        "ISODLEnvelope" => ("Конверт DL", "DL envelope"),
        "NorthAmericaNumber10Envelope" => ("Конверт №10", "No. 10 envelope"),
        "NorthAmericaMonarchEnvelope" => ("Конверт Monarch", "Monarch envelope"),
        "CustomMediaSize" => ("Пользовательский размер", "Custom size"),
        "Millimeter" => ("Миллиметры", "Millimeters"),
        "Inch" => ("Дюймы", "Inches"),
        "PageIResolutionValue" => ("Значение разрешения", "Resolution value"),
        "Text" => ("Текст", "Text"),
        "Underlying" | "Underlay" => ("Под содержимым", "Below content"),
        "Floating" | "Overlay" => ("Над содержимым", "Above content"),
        "Date" => ("Дата", "Date"),
        "LogonName" => ("Имя пользователя", "User name"),
        "JobAccountingName" => ("Имя учётной записи задания", "Job account name"),
        "ImageTemplate" => ("Шаблон изображения", "Image template"),
        "Image" => ("Изображение", "Image"),
        "Top" => ("Сверху", "Top"),
        "Left" => ("Слева", "Left"),
        "Right" => ("Справа", "Right"),
        "BottomLeft" => ("Снизу слева", "Bottom left"),
        "Bottom" => ("Снизу", "Bottom"),
        "BottomRight" => ("Снизу справа", "Bottom right"),
        "Options" => ("Параметры", "Options"),
        "ShortDate" => ("Короткая дата", "Short date"),
        "LongDate" => ("Полная дата", "Long date"),
        "ShortTime" => ("Короткое время", "Short time"),
        "LongTime" => ("Полное время", "Long time"),
        "PageNumber" => ("Номер страницы", "Page number"),
        "ComputerName" => ("Имя компьютера", "Computer name"),
        _ => return None,
    };
    Some(pick(lang, translations))
}

pub(super) fn parameter(name: &str, lang: Lang) -> Option<&'static str> {
    let translations = match name {
        "DocumentColorAdjustBrightness" => ("Яркость", "Brightness"),
        "DocumentColorAdjustContrast" => ("Контрастность", "Contrast"),
        "DocumentColorAdjustSaturation" => ("Насыщенность", "Saturation"),
        "DocumentColorAdjustRedValue" => ("Красный", "Red"),
        "DocumentColorAdjustGreenValue" => ("Зелёный", "Green"),
        "DocumentColorAdjustBlueValue" => ("Синий", "Blue"),
        "DocumentColorAdjustPreferredColorSkin" => ("Оттенки кожи", "Skin tones"),
        "DocumentColorAdjustPreferredColorGrass" => ("Оттенки травы", "Grass tones"),
        "DocumentColorAdjustPreferredColorSky" => ("Оттенки неба", "Sky tones"),
        "JobCopiesAllDocuments" => ("Количество копий", "Copies"),
        "PagePosterOverlapValue" => ("Перекрытие частей плаката", "Poster overlap"),
        "PageScalingOffsetWidth" => ("Смещение по горизонтали", "Horizontal offset"),
        "PageScalingOffsetHeight" => ("Смещение по вертикали", "Vertical offset"),
        "PageScalingScale" => ("Масштаб", "Scale"),
        "PageScalingTargetMediaSizeId" => ("Код целевого размера бумаги", "Target paper size ID"),
        "PageScalingTargetMediaSizeWidth" => ("Ширина целевой бумаги", "Target paper width"),
        "PageScalingTargetMediaSizeHeight" => ("Высота целевой бумаги", "Target paper height"),
        "PageScalingTargetMediaSizeName" => ("Целевой размер бумаги", "Target paper size"),
        "PageScalingTargetMediaSizeXOffset" => ("Горизонтальное поле", "Horizontal margin"),
        "PageScalingTargetMediaSizeYOffset" => ("Вертикальное поле", "Vertical margin"),
        "DocumentOverlayOverlayPath" => ("Файл наложения", "Overlay file"),
        "PageMediaSizeMediaSizeWidth" => ("Пользовательская ширина бумаги", "Custom paper width"),
        "PageMediaSizeMediaSizeHeight" => ("Пользовательская высота бумаги", "Custom paper height"),
        "PageIResolutionIResolution" => ("Разрешение изображения", "Image resolution"),
        "PageIResolutionBPP" => ("Глубина цвета", "Color depth"),
        "PageIResolutionImageQuality" => ("Качество изображения", "Image quality"),
        "PageWatermarkTextColor" => ("Цвет водяного знака", "Watermark color"),
        "PageWatermarkTextFontSize" => ("Размер шрифта водяного знака", "Watermark font size"),
        "PageWatermarkTextText" => ("Текст водяного знака", "Watermark text"),
        "PageWatermarkTextAngle" => ("Угол водяного знака", "Watermark angle"),
        "PageWatermarkTextFontFace" => ("Шрифт водяного знака", "Watermark font"),
        "PageWatermarkTextFontCharset" => ("Кодировка шрифта", "Font encoding"),
        "PageWatermarkTextFontWeight" => ("Насыщенность шрифта", "Font weight"),
        "PageWatermarkTextFontItalic" => ("Курсив", "Italic"),
        "PageWatermarkTextWatermarkName" => ("Название водяного знака", "Watermark name"),
        "PageWatermarkTextImageScale" => ("Масштаб водяного знака", "Watermark scale"),
        "PageWatermarkTextTransparencyLevel" => {
            ("Прозрачность водяного знака", "Watermark transparency")
        }
        "PageHeaderFooterOptionsFontFace" => ("Шрифт колонтитулов", "Header and footer font"),
        "PageHeaderFooterOptionsFontSize" => {
            ("Размер шрифта колонтитулов", "Header and footer font size")
        }
        "PageHeaderFooterOptionsFontColor" => ("Цвет колонтитулов", "Header and footer color"),
        "PageHeaderFooterOptionsFontCharset" => {
            ("Кодировка колонтитулов", "Header and footer encoding")
        }
        _ => return None,
    };
    Some(pick(lang, translations))
}

fn pick(lang: Lang, (ru, en): (&'static str, &'static str)) -> &'static str {
    match lang {
        Lang::Ru => ru,
        Lang::En => en,
    }
}
