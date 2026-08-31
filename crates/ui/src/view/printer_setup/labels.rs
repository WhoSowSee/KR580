use super::driver_locale;
use crate::i18n::Lang;
use k580_ui::devices::printer::{PrinterPaper, PrinterSource, PrinterStatus};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) enum Label {
    Title,
    Printer,
    Name,
    Properties,
    Status,
    Type,
    Place,
    Comment,
    Paper,
    Size,
    Source,
    Orientation,
    Portrait,
    Landscape,
    Ok,
    Cancel,
    Close,
    Loading,
    LoadingSettings,
    SelectPrinter,
    NoSelection,
    NoPrinters,
}

pub(super) fn label(lang: Lang, label: Label) -> &'static str {
    match (lang, label) {
        (Lang::Ru, Label::Title) => "Настройка печати",
        (Lang::En, Label::Title) => "Print setup",
        (Lang::Ru, Label::Printer) => "Принтер",
        (Lang::En, Label::Printer) => "Printer",
        (Lang::Ru, Label::Name) => "Имя:",
        (Lang::En, Label::Name) => "Name:",
        (Lang::Ru, Label::Properties) => "Свойства...",
        (Lang::En, Label::Properties) => "Properties...",
        (Lang::Ru, Label::Status) => "Состояние:",
        (Lang::En, Label::Status) => "Status:",
        (Lang::Ru, Label::Type) => "Тип:",
        (Lang::En, Label::Type) => "Type:",
        (Lang::Ru, Label::Place) => "Место:",
        (Lang::En, Label::Place) => "Location:",
        (Lang::Ru, Label::Comment) => "Комментарий:",
        (Lang::En, Label::Comment) => "Comment:",
        (Lang::Ru, Label::Paper) => "Бумага",
        (Lang::En, Label::Paper) => "Paper",
        (Lang::Ru, Label::Size) => "Размер:",
        (Lang::En, Label::Size) => "Size:",
        (Lang::Ru, Label::Source) => "Подача:",
        (Lang::En, Label::Source) => "Source:",
        (Lang::Ru, Label::Orientation) => "Ориентация",
        (Lang::En, Label::Orientation) => "Orientation",
        (Lang::Ru, Label::Portrait) => "Книжная",
        (Lang::En, Label::Portrait) => "Portrait",
        (Lang::Ru, Label::Landscape) => "Альбомная",
        (Lang::En, Label::Landscape) => "Landscape",
        (_, Label::Ok) => "OK",
        (Lang::Ru, Label::Cancel) => "Отмена",
        (Lang::En, Label::Cancel) => "Cancel",
        (Lang::Ru, Label::Close) => "Закрыть",
        (Lang::En, Label::Close) => "Close",
        (Lang::Ru, Label::Loading) => "Загрузка принтеров...",
        (Lang::En, Label::Loading) => "Loading printers...",
        (Lang::Ru, Label::LoadingSettings) => "Загрузка параметров...",
        (Lang::En, Label::LoadingSettings) => "Loading settings...",
        (Lang::Ru, Label::SelectPrinter) => "Выберите принтер",
        (Lang::En, Label::SelectPrinter) => "Select a printer",
        (Lang::Ru, Label::NoSelection) => "Принтер не выбран",
        (Lang::En, Label::NoSelection) => "No printer selected",
        (Lang::Ru, Label::NoPrinters) => "Принтеры не найдены",
        (Lang::En, Label::NoPrinters) => "No printers found",
    }
}

pub(super) fn localized_status(status: PrinterStatus, lang: Lang) -> &'static str {
    let (ru, en) = match status {
        PrinterStatus::Ready => ("Готов", "Ready"),
        PrinterStatus::Paused => ("Приостановлен", "Paused"),
        PrinterStatus::Error => ("Ошибка", "Error"),
        PrinterStatus::PendingDeletion => ("Удаляется", "Pending deletion"),
        PrinterStatus::PaperJam => ("Замятие бумаги", "Paper jam"),
        PrinterStatus::PaperOut => ("Нет бумаги", "Paper out"),
        PrinterStatus::ManualFeed => ("Ручная подача", "Manual feed"),
        PrinterStatus::PaperProblem => ("Проблема с бумагой", "Paper problem"),
        PrinterStatus::Offline => ("Не в сети", "Offline"),
        PrinterStatus::Busy => ("Занят", "Busy"),
        PrinterStatus::Printing => ("Печать", "Printing"),
        PrinterStatus::OutputBinFull => ("Приёмный лоток заполнен", "Output bin full"),
        PrinterStatus::NotAvailable => ("Недоступен", "Not available"),
        PrinterStatus::Waiting => ("Ожидание", "Waiting"),
        PrinterStatus::Processing => ("Обработка", "Processing"),
        PrinterStatus::Initializing => ("Инициализация", "Initializing"),
        PrinterStatus::WarmingUp => ("Прогрев", "Warming up"),
        PrinterStatus::TonerLow => ("Мало тонера", "Toner low"),
        PrinterStatus::NoToner => ("Нет тонера", "No toner"),
        PrinterStatus::UserIntervention => ("Требуется вмешательство", "User intervention"),
        PrinterStatus::OutOfMemory => ("Недостаточно памяти", "Out of memory"),
        PrinterStatus::DoorOpen => ("Открыта крышка", "Door open"),
        PrinterStatus::Unknown => ("Неизвестно", "Unknown"),
    };
    match lang {
        Lang::Ru => ru,
        Lang::En => en,
    }
}

pub(super) fn localized_paper_name(paper: &PrinterPaper, lang: Lang) -> String {
    localized_capability_name(
        paper.id,
        &paper.name,
        lang,
        standard_paper_name(paper.id, lang),
        ("Бумага", "Paper"),
    )
}

pub(super) fn localized_source_name(source: &PrinterSource, lang: Lang) -> String {
    localized_capability_name(
        source.id,
        &source.name,
        lang,
        standard_source_name(source.id, lang),
        ("Подача", "Source"),
    )
}

fn localized_capability_name(
    id: i16,
    name: &str,
    lang: Lang,
    standard: Option<&'static str>,
    fallback_prefix: (&'static str, &'static str),
) -> String {
    let localized = match lang {
        Lang::Ru => localized_driver_name(name, lang).or_else(|| standard.map(str::to_owned)),
        Lang::En => standard
            .map(str::to_owned)
            .or_else(|| localized_driver_name(name, lang)),
    };
    localized.unwrap_or_else(|| format!("{} {id}", pick(lang, fallback_prefix)))
}

fn localized_driver_name(name: &str, lang: Lang) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    match lang {
        Lang::Ru if driver_locale::has_cyrillic(name) => Some(name.to_owned()),
        Lang::Ru => driver_locale::russian(name),
        Lang::En if driver_locale::has_cyrillic(name) => driver_locale::english(name),
        Lang::En => Some(name.to_owned()),
    }
}

fn standard_source_name(id: i16, lang: Lang) -> Option<&'static str> {
    let names = match id {
        1 => ("Верхний лоток", "Upper tray"),
        2 => ("Нижний лоток", "Lower tray"),
        3 => ("Средний лоток", "Middle tray"),
        4 => ("Ручная подача", "Manual feed"),
        5 => ("Конверт", "Envelope"),
        6 => ("Ручная подача конвертов", "Manual envelope feed"),
        7 => ("Автовыбор", "Auto select"),
        8 => ("Тракторная подача", "Tractor feed"),
        9 => ("Малый формат", "Small format"),
        10 => ("Большой формат", "Large format"),
        11 => ("Лоток большой ёмкости", "Large capacity"),
        14 => ("Кассета", "Cassette"),
        15 => ("Источник формы", "Form source"),
        _ => return None,
    };
    Some(pick(lang, names))
}

fn standard_paper_name(id: i16, lang: Lang) -> Option<&'static str> {
    let names = match id {
        1 => ("Letter", "Letter"),
        2 => ("Letter (малый)", "Letter small"),
        3 => ("Tabloid", "Tabloid"),
        4 => ("Ledger", "Ledger"),
        5 => ("Legal", "Legal"),
        6 => ("Statement", "Statement"),
        7 => ("Executive", "Executive"),
        8 => ("A3", "A3"),
        9 => ("A4", "A4"),
        10 => ("A4 (малый)", "A4 small"),
        11 => ("A5", "A5"),
        12 => ("B4 (JIS)", "B4 (JIS)"),
        13 => ("B5 (JIS)", "B5 (JIS)"),
        14 => ("Folio", "Folio"),
        15 => ("Quarto", "Quarto"),
        16 => ("10 × 14 дюймов", "10 × 14 in"),
        17 => ("11 × 17 дюймов", "11 × 17 in"),
        18 => ("Note", "Note"),
        19 => ("Конверт № 9", "No. 9 envelope"),
        20 => ("Конверт № 10", "No. 10 envelope"),
        21 => ("Конверт № 11", "No. 11 envelope"),
        22 => ("Конверт № 12", "No. 12 envelope"),
        23 => ("Конверт № 14", "No. 14 envelope"),
        24 => ("Лист C", "C sheet"),
        25 => ("Лист D", "D sheet"),
        26 => ("Лист E", "E sheet"),
        27 => ("Конверт DL", "DL envelope"),
        28 => ("Конверт C5", "C5 envelope"),
        29 => ("Конверт C3", "C3 envelope"),
        30 => ("Конверт C4", "C4 envelope"),
        31 => ("Конверт C6", "C6 envelope"),
        32 => ("Конверт C65", "C65 envelope"),
        33 => ("Конверт B4", "B4 envelope"),
        34 => ("Конверт B5", "B5 envelope"),
        35 => ("Конверт B6", "B6 envelope"),
        36 => ("Итальянский конверт", "Italy envelope"),
        37 => ("Конверт Monarch", "Monarch envelope"),
        38 => ("Конверт 6 3/4", "6 3/4 envelope"),
        39 => ("Стандартная фальцованная бумага США", "US standard fanfold"),
        40 => (
            "Стандартная фальцованная бумага Германии",
            "German standard fanfold",
        ),
        41 => (
            "Фальцованная бумага Legal (Германия)",
            "German legal fanfold",
        ),
        42 => ("B4 (ISO)", "B4 (ISO)"),
        43 => ("Японская открытка", "Japanese postcard"),
        44 => ("9 × 11 дюймов", "9 × 11 in"),
        45 => ("10 × 11 дюймов", "10 × 11 in"),
        46 => ("15 × 11 дюймов", "15 × 11 in"),
        47 => ("Конверт Invite", "Invite envelope"),
        49 => ("Letter (увеличенный)", "Letter extra"),
        50 => ("Legal (увеличенный)", "Legal extra"),
        51 => ("Tabloid (увеличенный)", "Tabloid extra"),
        52 => ("A4 (увеличенный)", "A4 extra"),
        53 => ("Letter (поперечный)", "Letter transverse"),
        54 => ("A4 (поперечный)", "A4 transverse"),
        55 => ("Letter (увеличенный поперечный)", "Letter extra transverse"),
        56 => ("Super A", "Super A"),
        57 => ("Super B", "Super B"),
        58 => ("Letter Plus", "Letter plus"),
        59 => ("A4 Plus", "A4 plus"),
        60 => ("A5 (поперечный)", "A5 transverse"),
        61 => ("B5 (JIS, поперечный)", "B5 (JIS) transverse"),
        62 => ("A3 (увеличенный)", "A3 extra"),
        63 => ("A5 (увеличенный)", "A5 extra"),
        64 => ("B5 (ISO, увеличенный)", "B5 (ISO) extra"),
        65 => ("A2", "A2"),
        66 => ("A3 (поперечный)", "A3 transverse"),
        67 => ("A3 (увеличенный поперечный)", "A3 extra transverse"),
        68 => ("Японская двойная открытка", "Japanese double postcard"),
        69 => ("A6", "A6"),
        70 => ("Японский конверт Kaku № 2", "Japanese Kaku No. 2 envelope"),
        71 => ("Японский конверт Kaku № 3", "Japanese Kaku No. 3 envelope"),
        72 => ("Японский конверт Chou № 3", "Japanese Chou No. 3 envelope"),
        73 => ("Японский конверт Chou № 4", "Japanese Chou No. 4 envelope"),
        74 => ("Letter (с поворотом)", "Letter rotated"),
        75 => ("A3 (с поворотом)", "A3 rotated"),
        76 => ("A4 (с поворотом)", "A4 rotated"),
        77 => ("A5 (с поворотом)", "A5 rotated"),
        78 => ("B4 (JIS, с поворотом)", "B4 (JIS) rotated"),
        79 => ("B5 (JIS, с поворотом)", "B5 (JIS) rotated"),
        80 => (
            "Японская открытка (с поворотом)",
            "Japanese postcard rotated",
        ),
        81 => (
            "Японская двойная открытка (с поворотом)",
            "Japanese double postcard rotated",
        ),
        82 => ("A6 (с поворотом)", "A6 rotated"),
        83 => (
            "Японский конверт Kaku № 2 (с поворотом)",
            "Japanese Kaku No. 2 envelope rotated",
        ),
        84 => (
            "Японский конверт Kaku № 3 (с поворотом)",
            "Japanese Kaku No. 3 envelope rotated",
        ),
        85 => (
            "Японский конверт Chou № 3 (с поворотом)",
            "Japanese Chou No. 3 envelope rotated",
        ),
        86 => (
            "Японский конверт Chou № 4 (с поворотом)",
            "Japanese Chou No. 4 envelope rotated",
        ),
        87 => ("B6 (JIS)", "B6 (JIS)"),
        88 => ("B6 (JIS, с поворотом)", "B6 (JIS) rotated"),
        89 => ("12 × 11 дюймов", "12 × 11 in"),
        90 => ("Японский конверт You № 4", "Japanese You No. 4 envelope"),
        91 => (
            "Японский конверт You № 4 (с поворотом)",
            "Japanese You No. 4 envelope rotated",
        ),
        92 => ("PRC 16K", "PRC 16K"),
        93 => ("PRC 32K", "PRC 32K"),
        94 => ("PRC 32K (большой)", "PRC 32K big"),
        95 => ("Конверт PRC № 1", "PRC No. 1 envelope"),
        96 => ("Конверт PRC № 2", "PRC No. 2 envelope"),
        97 => ("Конверт PRC № 3", "PRC No. 3 envelope"),
        98 => ("Конверт PRC № 4", "PRC No. 4 envelope"),
        99 => ("Конверт PRC № 5", "PRC No. 5 envelope"),
        100 => ("Конверт PRC № 6", "PRC No. 6 envelope"),
        101 => ("Конверт PRC № 7", "PRC No. 7 envelope"),
        102 => ("Конверт PRC № 8", "PRC No. 8 envelope"),
        103 => ("Конверт PRC № 9", "PRC No. 9 envelope"),
        104 => ("Конверт PRC № 10", "PRC No. 10 envelope"),
        105 => ("PRC 16K (с поворотом)", "PRC 16K rotated"),
        106 => ("PRC 32K (с поворотом)", "PRC 32K rotated"),
        107 => ("PRC 32K (большой, с поворотом)", "PRC 32K big rotated"),
        108 => (
            "Конверт PRC № 1 (с поворотом)",
            "PRC No. 1 envelope rotated",
        ),
        109 => (
            "Конверт PRC № 2 (с поворотом)",
            "PRC No. 2 envelope rotated",
        ),
        110 => (
            "Конверт PRC № 3 (с поворотом)",
            "PRC No. 3 envelope rotated",
        ),
        111 => (
            "Конверт PRC № 4 (с поворотом)",
            "PRC No. 4 envelope rotated",
        ),
        112 => (
            "Конверт PRC № 5 (с поворотом)",
            "PRC No. 5 envelope rotated",
        ),
        113 => (
            "Конверт PRC № 6 (с поворотом)",
            "PRC No. 6 envelope rotated",
        ),
        114 => (
            "Конверт PRC № 7 (с поворотом)",
            "PRC No. 7 envelope rotated",
        ),
        115 => (
            "Конверт PRC № 8 (с поворотом)",
            "PRC No. 8 envelope rotated",
        ),
        116 => (
            "Конверт PRC № 9 (с поворотом)",
            "PRC No. 9 envelope rotated",
        ),
        117 => (
            "Конверт PRC № 10 (с поворотом)",
            "PRC No. 10 envelope rotated",
        ),
        _ => return None,
    };
    Some(pick(lang, names))
}

fn pick(lang: Lang, names: (&'static str, &'static str)) -> &'static str {
    match lang {
        Lang::Ru => names.0,
        Lang::En => names.1,
    }
}
