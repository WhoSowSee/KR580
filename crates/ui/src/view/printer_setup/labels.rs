use super::driver_locale;
use crate::i18n::Lang;
use k580_ui::devices::printer::{PrinterPaper, PrinterSource, PrinterStatus};

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
    if lang == Lang::Ru {
        return paper.name.clone();
    }
    standard_paper_name(paper.id)
        .map(str::to_owned)
        .unwrap_or_else(|| english_driver_name(&paper.name, format!("Paper {}", paper.id)))
}

pub(super) fn localized_source_name(source: &PrinterSource, lang: Lang) -> String {
    if lang == Lang::Ru {
        return source.name.clone();
    }
    standard_source_name(source.id)
        .map(str::to_owned)
        .unwrap_or_else(|| english_driver_name(&source.name, format!("Source {}", source.id)))
}

fn standard_source_name(id: i16) -> Option<&'static str> {
    Some(match id {
        1 => "Upper tray",
        2 => "Lower tray",
        3 => "Middle tray",
        4 => "Manual feed",
        5 => "Envelope",
        6 => "Manual envelope feed",
        7 => "Auto select",
        8 => "Tractor feed",
        9 => "Small format",
        10 => "Large format",
        11 => "Large capacity",
        14 => "Cassette",
        15 => "Form source",
        _ => return None,
    })
}

fn standard_paper_name(id: i16) -> Option<&'static str> {
    Some(match id {
        1 => "Letter",
        2 => "Letter small",
        3 => "Tabloid",
        4 => "Ledger",
        5 => "Legal",
        6 => "Statement",
        7 => "Executive",
        8 => "A3",
        9 => "A4",
        10 => "A4 small",
        11 => "A5",
        12 => "B4 (JIS)",
        13 => "B5 (JIS)",
        14 => "Folio",
        15 => "Quarto",
        16 => "10 × 14 in",
        17 => "11 × 17 in",
        18 => "Note",
        19 => "No. 9 envelope",
        20 => "No. 10 envelope",
        21 => "No. 11 envelope",
        22 => "No. 12 envelope",
        23 => "No. 14 envelope",
        24 => "C sheet",
        25 => "D sheet",
        26 => "E sheet",
        27 => "DL envelope",
        28 => "C5 envelope",
        29 => "C3 envelope",
        30 => "C4 envelope",
        31 => "C6 envelope",
        32 => "C65 envelope",
        33 => "B4 envelope",
        34 => "B5 envelope",
        35 => "B6 envelope",
        36 => "Italy envelope",
        37 => "Monarch envelope",
        38 => "6 3/4 envelope",
        39 => "US standard fanfold",
        40 => "German standard fanfold",
        41 => "German legal fanfold",
        42 => "B4 (ISO)",
        43 => "Japanese postcard",
        44 => "9 × 11 in",
        45 => "10 × 11 in",
        46 => "15 × 11 in",
        47 => "Invite envelope",
        49 => "Letter extra",
        50 => "Legal extra",
        51 => "Tabloid extra",
        52 => "A4 extra",
        53 => "Letter transverse",
        54 => "A4 transverse",
        55 => "Letter extra transverse",
        56 => "Super A",
        57 => "Super B",
        58 => "Letter plus",
        59 => "A4 plus",
        60 => "A5 transverse",
        61 => "B5 (JIS) transverse",
        62 => "A3 extra",
        63 => "A5 extra",
        64 => "B5 (ISO) extra",
        65 => "A2",
        66 => "A3 transverse",
        67 => "A3 extra transverse",
        68 => "Japanese double postcard",
        69 => "A6",
        70 => "Japanese Kaku No. 2 envelope",
        71 => "Japanese Kaku No. 3 envelope",
        72 => "Japanese Chou No. 3 envelope",
        73 => "Japanese Chou No. 4 envelope",
        74 => "Letter rotated",
        75 => "A3 rotated",
        76 => "A4 rotated",
        77 => "A5 rotated",
        78 => "B4 (JIS) rotated",
        79 => "B5 (JIS) rotated",
        80 => "Japanese postcard rotated",
        81 => "Japanese double postcard rotated",
        82 => "A6 rotated",
        83 => "Japanese Kaku No. 2 envelope rotated",
        84 => "Japanese Kaku No. 3 envelope rotated",
        85 => "Japanese Chou No. 3 envelope rotated",
        86 => "Japanese Chou No. 4 envelope rotated",
        87 => "B6 (JIS)",
        88 => "B6 (JIS) rotated",
        89 => "12 × 11 in",
        90 => "Japanese You No. 4 envelope",
        91 => "Japanese You No. 4 envelope rotated",
        92 => "PRC 16K",
        93 => "PRC 32K",
        94 => "PRC 32K big",
        95 => "PRC No. 1 envelope",
        96 => "PRC No. 2 envelope",
        97 => "PRC No. 3 envelope",
        98 => "PRC No. 4 envelope",
        99 => "PRC No. 5 envelope",
        100 => "PRC No. 6 envelope",
        101 => "PRC No. 7 envelope",
        102 => "PRC No. 8 envelope",
        103 => "PRC No. 9 envelope",
        104 => "PRC No. 10 envelope",
        105 => "PRC 16K rotated",
        106 => "PRC 32K rotated",
        107 => "PRC 32K big rotated",
        108 => "PRC No. 1 envelope rotated",
        109 => "PRC No. 2 envelope rotated",
        110 => "PRC No. 3 envelope rotated",
        111 => "PRC No. 4 envelope rotated",
        112 => "PRC No. 5 envelope rotated",
        113 => "PRC No. 6 envelope rotated",
        114 => "PRC No. 7 envelope rotated",
        115 => "PRC No. 8 envelope rotated",
        116 => "PRC No. 9 envelope rotated",
        117 => "PRC No. 10 envelope rotated",
        _ => return None,
    })
}

fn english_driver_name(name: &str, fallback: String) -> String {
    let name = name.trim();
    if name.is_empty() {
        return fallback;
    }
    if !driver_locale::has_cyrillic(name) {
        return name.to_owned();
    }
    driver_locale::english(name).unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_printer_capabilities_ignore_the_driver_locale() {
        let auto = PrinterSource {
            id: 7,
            name: "Автовыбор".to_owned(),
        };
        let tray = PrinterSource {
            id: 257,
            name: "Лоток 1".to_owned(),
        };
        let envelope = PrinterPaper {
            id: 27,
            name: "Конверт DL".to_owned(),
        };
        let vendor_source = PrinterSource {
            id: 258,
            name: "Нестандартная подача".to_owned(),
        };
        let blank_paper = PrinterPaper {
            id: 200,
            name: "   ".to_owned(),
        };

        assert_eq!(localized_source_name(&auto, Lang::En), "Auto select");
        assert_eq!(localized_source_name(&tray, Lang::En), "Tray 1");
        assert_eq!(localized_paper_name(&envelope, Lang::En), "DL envelope");
        assert_eq!(
            localized_source_name(&vendor_source, Lang::En),
            "Source 258"
        );
        assert_eq!(localized_paper_name(&blank_paper, Lang::En), "Paper 200");
    }

    #[test]
    fn every_printer_status_is_translated_into_russian() {
        let statuses = [
            PrinterStatus::Ready,
            PrinterStatus::Paused,
            PrinterStatus::Error,
            PrinterStatus::PendingDeletion,
            PrinterStatus::PaperJam,
            PrinterStatus::PaperOut,
            PrinterStatus::ManualFeed,
            PrinterStatus::PaperProblem,
            PrinterStatus::Offline,
            PrinterStatus::Busy,
            PrinterStatus::Printing,
            PrinterStatus::OutputBinFull,
            PrinterStatus::NotAvailable,
            PrinterStatus::Waiting,
            PrinterStatus::Processing,
            PrinterStatus::Initializing,
            PrinterStatus::WarmingUp,
            PrinterStatus::TonerLow,
            PrinterStatus::NoToner,
            PrinterStatus::UserIntervention,
            PrinterStatus::OutOfMemory,
            PrinterStatus::DoorOpen,
            PrinterStatus::Unknown,
        ];

        for status in statuses {
            let russian = localized_status(status, Lang::Ru);
            let english = localized_status(status, Lang::En);
            assert!(
                driver_locale::has_cyrillic(russian),
                "{status:?}: {russian}"
            );
            assert!(
                !driver_locale::has_cyrillic(english),
                "{status:?}: {english}"
            );
        }

        assert_eq!(
            localized_status(PrinterStatus::PaperJam, Lang::Ru),
            "Замятие бумаги"
        );
        assert_eq!(
            localized_status(PrinterStatus::TonerLow, Lang::En),
            "Toner low"
        );
    }

    #[test]
    fn russian_printer_capabilities_keep_driver_labels() {
        let source = PrinterSource {
            id: 7,
            name: "Автовыбор".to_owned(),
        };

        assert_eq!(localized_source_name(&source, Lang::Ru), "Автовыбор");
    }
}
