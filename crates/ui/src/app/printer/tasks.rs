use k580_ui::devices::printer::{
    PrinterConfiguration, PrinterInfo, PrinterPropertyChange, PrinterPropertySheet, PrinterSettings,
};

pub(super) async fn configure_native_printer_blocking() -> Result<Option<PrinterSettings>, String> {
    tokio::task::spawn_blocking(k580_ui::devices::printer::configure_native_printer)
        .await
        .map_err(|error| format!("printer setup task failed: {error}"))?
}

pub(super) async fn list_native_printers_blocking() -> Result<Vec<PrinterInfo>, String> {
    tokio::task::spawn_blocking(k580_ui::devices::printer::list_native_printers)
        .await
        .map_err(|error| format!("printer list task failed: {error}"))?
}

pub(super) async fn load_native_printer_configuration_blocking(
    printer: PrinterInfo,
) -> Result<PrinterConfiguration, String> {
    tokio::task::spawn_blocking(move || {
        k580_ui::devices::printer::load_native_printer_configuration(&printer)
    })
    .await
    .map_err(|error| format!("printer capabilities task failed: {error}"))?
}

pub(super) async fn load_native_printer_properties_blocking(
    printer: PrinterInfo,
    settings: PrinterSettings,
) -> Result<PrinterPropertySheet, String> {
    tokio::task::spawn_blocking(move || {
        k580_ui::devices::printer::load_native_printer_properties(&printer, &settings)
    })
    .await
    .map_err(|error| format!("printer properties task failed: {error}"))?
}

pub(super) async fn apply_native_printer_property_blocking(
    printer: PrinterInfo,
    settings: PrinterSettings,
    change: PrinterPropertyChange,
) -> Result<PrinterPropertySheet, String> {
    tokio::task::spawn_blocking(move || {
        k580_ui::devices::printer::apply_native_printer_property(&printer, &settings, &change)
    })
    .await
    .map_err(|error| format!("printer property task failed: {error}"))?
}
