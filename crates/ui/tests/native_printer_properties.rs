#![cfg(windows)]

use k580_ui::devices::printer::{
    PrinterPropertyChange, apply_native_printer_property, list_native_printers,
    load_native_printer_configuration, load_native_printer_properties,
};

#[test]
#[ignore = "requires an installed Windows printer driver"]
fn native_print_ticket_properties_round_trip() {
    let printers = list_native_printers().unwrap();
    let printer = printers
        .iter()
        .find(|printer| printer.name.contains("HP Laser"))
        .or_else(|| printers.first())
        .expect("no installed printers");
    let configuration = load_native_printer_configuration(printer).unwrap();
    let sheet = load_native_printer_properties(printer, &configuration.settings).unwrap();
    assert_eq!(sheet.configuration.settings.printer_name, printer.name);
    assert_eq!(sheet.provider_error, None);
    assert!(!sheet.features.is_empty());
    println!(
        "{}: {} features, {} parameters",
        printer.name,
        sheet.features.len(),
        sheet.parameters.len()
    );
    if std::env::var_os("KR580_DUMP_PRINTER_PROPERTIES").is_some() {
        for feature in &sheet.features {
            println!("FEATURE\t{}\t{}", feature.name, feature.display_name);
            for option in &feature.options {
                println!(
                    "OPTION\t{}\t{}\t{}",
                    option.name, option.display_name, option.constrained
                );
            }
        }
        for parameter in &sheet.parameters {
            println!(
                "PARAMETER\t{}\t{}\t{}\t{}",
                parameter.name, parameter.display_name, parameter.value_type, parameter.value
            );
        }
    }
    if let Some((feature, option)) = sheet.features.iter().find_map(|feature| {
        let selected = feature.selected_option.as_ref()?;
        feature
            .options
            .iter()
            .find(|option| &option.name == selected)
            .map(|option| (feature, option))
    }) {
        let applied = apply_native_printer_property(
            printer,
            &sheet.configuration.settings,
            &PrinterPropertyChange::Feature {
                feature_name: feature.name.clone(),
                option_name: option.name.clone(),
            },
        )
        .unwrap();
        assert_eq!(applied.configuration.settings.printer_name, printer.name);
        assert!(!applied.configuration.settings.devmode.is_empty());
    }
}
