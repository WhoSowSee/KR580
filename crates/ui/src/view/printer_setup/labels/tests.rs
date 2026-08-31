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
fn russian_printer_capabilities_ignore_the_driver_locale() {
    let auto = PrinterSource {
        id: 257,
        name: "Automatically Select".to_owned(),
    };
    let standard_auto = PrinterSource {
        id: 7,
        name: "Driver default".to_owned(),
    };
    let envelope = PrinterPaper {
        id: 27,
        name: "Driver default".to_owned(),
    };
    let vendor_source = PrinterSource {
        id: 259,
        name: "Custom feeder".to_owned(),
    };
    let blank_paper = PrinterPaper {
        id: 200,
        name: "   ".to_owned(),
    };

    assert_eq!(localized_source_name(&auto, Lang::Ru), "Автовыбор");
    assert_eq!(localized_source_name(&standard_auto, Lang::Ru), "Автовыбор");
    assert_eq!(localized_paper_name(&envelope, Lang::Ru), "Конверт DL");
    assert_eq!(
        localized_source_name(&vendor_source, Lang::Ru),
        "Подача 259"
    );
    assert_eq!(localized_paper_name(&blank_paper, Lang::Ru), "Бумага 200");
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
