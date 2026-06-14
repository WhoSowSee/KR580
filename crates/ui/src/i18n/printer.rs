#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrinterKey {
    BufferContents,
    Status,
    BytesBuffered,
    PdfTarget,
    PdfTargetMissing,
    ShowText,
    ShowBytes,
    PrintPdf,
    ClearBuffer,
}

pub(super) fn translate_ru(key: PrinterKey) -> &'static str {
    match key {
        PrinterKey::BufferContents => "Содержимое буфера принтера",
        PrinterKey::Status => "Статус",
        PrinterKey::BytesBuffered => "байт в буфере",
        PrinterKey::PdfTarget => "PDF",
        PrinterKey::PdfTargetMissing => "ещё не печатался",
        PrinterKey::ShowText => "Показать текст",
        PrinterKey::ShowBytes => "Показать байты",
        PrinterKey::PrintPdf => "Печатать в PDF",
        PrinterKey::ClearBuffer => "Очистить буфер принтера",
    }
}

pub(super) fn translate_en(key: PrinterKey) -> &'static str {
    match key {
        PrinterKey::BufferContents => "Printer buffer contents",
        PrinterKey::Status => "Status",
        PrinterKey::BytesBuffered => "bytes buffered",
        PrinterKey::PdfTarget => "PDF",
        PrinterKey::PdfTargetMissing => "not printed yet",
        PrinterKey::ShowText => "Show text",
        PrinterKey::ShowBytes => "Show bytes",
        PrinterKey::PrintPdf => "Print to PDF",
        PrinterKey::ClearBuffer => "Clear printer buffer",
    }
}
