use super::{
    PrinterPropertiesDialog, PrinterPropertiesFocus, PrinterPropertiesTab, PrinterPropertyDropdown,
    PrinterSetupDialog, PrinterSetupDropdown, PrinterSetupFocus, PrinterSetupTarget,
};
use crate::app::{DesktopApp, Message};
use crate::persistence::PrinterPreset;
use k580_ui::devices::printer::{PrinterInfo, PrinterSettings};

#[test]
fn setup_dropdown_arrows_move_highlight_without_changing_selection() {
    let mut app = app_with_printers();

    let _ = app.update(Message::PrinterSetupDropdownToggled(
        PrinterSetupDropdown::Printer,
    ));
    let dialog = app.printer_setup_dialog.as_ref().unwrap();
    assert_eq!(dialog.open_dropdown, Some(PrinterSetupDropdown::Printer));
    assert_eq!(dialog.dropdown_highlight, Some(0));
    assert_eq!(dialog.selected_name.as_deref(), Some("First"));

    let _ = app.update(Message::ArrowKey(-1));
    let dialog = app.printer_setup_dialog.as_ref().unwrap();
    assert_eq!(dialog.dropdown_highlight, Some(1));
    assert_eq!(dialog.selected_name.as_deref(), Some("First"));
}

#[test]
fn setup_tab_cycles_forward_and_backward_over_enabled_controls() {
    let mut app = app_with_printers();

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().focus,
        PrinterSetupFocus::Cancel
    );

    app.printer_setup_dialog.as_mut().unwrap().focus = PrinterSetupFocus::Printer;
    let _ = app.update(Message::FocusCycle { backward: true });
    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().focus,
        PrinterSetupFocus::Close
    );
}

#[test]
fn escape_closes_dropdown_before_closing_setup() {
    let mut app = app_with_printers();
    let _ = app.update(Message::PrinterSetupDropdownToggled(
        PrinterSetupDropdown::Printer,
    ));

    let _ = app.update(Message::EscPressed);

    let dialog = app.printer_setup_dialog.as_ref().unwrap();
    assert_eq!(dialog.open_dropdown, None);
}

#[test]
fn setup_dropdown_dismissal_only_closes_its_source() {
    let mut app = app_with_printers();
    let _ = app.update(Message::PrinterSetupDropdownToggled(
        PrinterSetupDropdown::Printer,
    ));

    let _ = app.update(Message::PrinterSetupDropdownDismissed(
        PrinterSetupDropdown::Paper,
    ));
    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().open_dropdown,
        Some(PrinterSetupDropdown::Printer)
    );
    let _ = app.update(Message::PrinterSetupDropdownDismissed(
        PrinterSetupDropdown::Printer,
    ));

    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().open_dropdown,
        None
    );
}

#[test]
fn property_dropdown_arrows_track_profile_options() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let presets = ["First", "Second"]
        .into_iter()
        .map(|name| PrinterPreset {
            name: name.to_owned(),
            settings: PrinterSettings::named("Printer".to_owned()),
        })
        .collect();
    let mut setup =
        PrinterSetupDialog::new(PrinterSetupTarget::Session, Some("Printer".to_owned()));
    let mut properties = PrinterPropertiesDialog::new(String::new(), presets);
    properties.loading = false;
    properties.selected_preset = Some("First".to_owned());
    setup.properties = Some(properties);
    app.printer_setup_dialog = Some(setup);

    let _ = app.update(Message::PrinterPropertyDropdownToggled(
        PrinterPropertyDropdown::Preset,
    ));
    let _ = app.update(Message::ArrowKey(-1));

    let properties = app
        .printer_setup_dialog
        .as_ref()
        .unwrap()
        .properties
        .as_ref()
        .unwrap();
    assert_eq!(properties.dropdown_highlight, Some(1));
    assert_eq!(properties.selected_preset.as_deref(), Some("First"));
}

#[test]
fn property_tabs_start_on_favorites_and_show_focus_only_after_keyboard_navigation() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let mut setup =
        PrinterSetupDialog::new(PrinterSetupTarget::Session, Some("Printer".to_owned()));
    let mut properties = PrinterPropertiesDialog::new(String::new(), Vec::new());
    properties.loading = false;
    setup.properties = Some(properties);
    app.printer_setup_dialog = Some(setup);
    let properties = app
        .printer_setup_dialog
        .as_ref()
        .unwrap()
        .properties
        .as_ref()
        .unwrap();
    assert_eq!(
        properties.focus,
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Favorites)
    );
    assert!(!properties.tab_focus_visible);

    let _ = app.update(Message::PrinterPropertiesFocusResolved {
        focused: None,
        backward: false,
    });
    assert_eq!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .as_ref()
            .unwrap()
            .focus,
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::General)
    );
    assert!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .as_ref()
            .unwrap()
            .tab_focus_visible
    );

    let _ = app.update(Message::PrinterPropertiesTabSelected(
        PrinterPropertiesTab::Paper,
    ));
    let properties = app
        .printer_setup_dialog
        .as_ref()
        .unwrap()
        .properties
        .as_ref()
        .unwrap();
    assert_eq!(
        properties.focus,
        PrinterPropertiesFocus::Tab(PrinterPropertiesTab::Paper)
    );
    assert!(!properties.tab_focus_visible);
}

#[test]
fn property_dropdown_dismissal_only_closes_its_source() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let mut setup =
        PrinterSetupDialog::new(PrinterSetupTarget::Session, Some("Printer".to_owned()));
    let mut properties = PrinterPropertiesDialog::new(String::new(), Vec::new());
    properties.open_dropdown = Some(PrinterPropertyDropdown::Preset);
    setup.properties = Some(properties);
    app.printer_setup_dialog = Some(setup);

    let _ = app.update(Message::PrinterPropertyDropdownDismissed(
        PrinterPropertyDropdown::Paper,
    ));
    assert_eq!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .as_ref()
            .unwrap()
            .open_dropdown,
        Some(PrinterPropertyDropdown::Preset)
    );
    let _ = app.update(Message::PrinterPropertyDropdownDismissed(
        PrinterPropertyDropdown::Preset,
    ));

    assert_eq!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .as_ref()
            .unwrap()
            .open_dropdown,
        None
    );
}

fn app_with_printers() -> DesktopApp {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let mut dialog = PrinterSetupDialog::new(PrinterSetupTarget::Session, Some("First".to_owned()));
    dialog.loading = false;
    dialog.printers = vec![printer("First"), printer("Second")];
    app.printer_setup_dialog = Some(dialog);
    app
}

fn printer(name: &str) -> PrinterInfo {
    PrinterInfo {
        name: name.to_owned(),
        driver: String::new(),
        port: String::new(),
        location: String::new(),
        comment: String::new(),
        status: String::new(),
        is_default: false,
    }
}
