#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrinterKey {
    BufferContents,
    Status,
    BytesBuffered,
    ShowText,
    ShowBytes,
    PrintNative,
    ConfigureSession,
    ClearBuffer,
    Target,
    SystemDefault,
    PrintFailed,
}

pub(super) fn translate_ru(key: PrinterKey) -> &'static str {
    match key {
        PrinterKey::BufferContents => "Содержимое буфера принтера",
        PrinterKey::Status => "Статус",
        PrinterKey::BytesBuffered => "байт в буфере",
        PrinterKey::ShowText => "Показать текст",
        PrinterKey::ShowBytes => "Показать байты",
        PrinterKey::PrintNative => "Печатать",
        PrinterKey::ConfigureSession => "Настройки принтера",
        PrinterKey::ClearBuffer => "Очистить буфер принтера",
        PrinterKey::Target => "Принтер",
        PrinterKey::SystemDefault => "системный по умолчанию",
        PrinterKey::PrintFailed => "ошибка печати",
    }
}

pub(super) fn translate_en(key: PrinterKey) -> &'static str {
    match key {
        PrinterKey::BufferContents => "Printer buffer contents",
        PrinterKey::Status => "Status",
        PrinterKey::BytesBuffered => "bytes buffered",
        PrinterKey::ShowText => "Show text",
        PrinterKey::ShowBytes => "Show bytes",
        PrinterKey::PrintNative => "Print",
        PrinterKey::ConfigureSession => "Printer setup",
        PrinterKey::ClearBuffer => "Clear printer buffer",
        PrinterKey::Target => "Printer",
        PrinterKey::SystemDefault => "system default",
        PrinterKey::PrintFailed => "print failed",
    }
}
