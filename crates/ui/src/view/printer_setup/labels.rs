use crate::i18n::Lang;

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
    }
}

pub(super) fn localized_status(status: &str, lang: Lang) -> &str {
    if lang != Lang::Ru {
        return status;
    }
    match status {
        "Ready" => "Готов",
        "Paused" => "Приостановлен",
        "Offline" => "Не в сети",
        "Busy" => "Занят",
        "Printing" => "Печать",
        "Error" => "Ошибка",
        _ => status,
    }
}
