use super::window::{detached_printer_properties_position, detached_printer_setup_position};
use super::{
    PrinterPropertiesDialog, PrinterPropertiesFocus, PrinterPropertiesTab, PrinterPropertyDropdown,
    PrinterSetupDialog, PrinterSetupDropdown, PrinterSetupFocus, PrinterSetupTarget,
};
use crate::app::{DesktopApp, Message};
use crate::persistence::PrinterPreset;
use iced::Point;
use k580_ui::devices::printer::{PrinterInfo, PrinterSettings};
use std::time::{Duration, Instant};

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
    assert!(!app.printer_setup_dialog.as_ref().unwrap().focus_visible);

    let _ = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().focus,
        PrinterSetupFocus::Cancel
    );
    assert!(app.printer_setup_dialog.as_ref().unwrap().focus_visible);

    let _ = app.update(Message::EnterPressed);
    assert!(!app.printer_setup_dialog.as_ref().unwrap().focus_visible);

    app.printer_setup_dialog.as_mut().unwrap().focus = PrinterSetupFocus::Printer;
    let _ = app.update(Message::FocusCycle { backward: true });
    assert_eq!(
        app.printer_setup_dialog.as_ref().unwrap().focus,
        PrinterSetupFocus::Close
    );
    assert!(app.printer_setup_dialog.as_ref().unwrap().focus_visible);
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
    assert!(!properties.focus_visible);

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
            .focus_visible
    );

    let _ = app.update(Message::EnterPressed);
    let properties = app
        .printer_setup_dialog
        .as_ref()
        .unwrap()
        .properties
        .as_ref()
        .unwrap();
    assert_eq!(properties.tab, PrinterPropertiesTab::General);
    assert!(!properties.focus_visible);

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
    assert!(!properties.focus_visible);
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

#[test]
fn detached_setup_uses_dedicated_window_without_replacing_printer() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.printer_open = true;
    app.printer_window.detached = true;
    let printer_window = iced::window::Id::unique();
    app.printer_window.id = Some(printer_window);

    let _ = app.open_printer_setup_dialog(PrinterSetupTarget::Session);
    assert!(!app.printer_setup_dialog.as_ref().unwrap().owner_ready);

    let _ = app.update(Message::PrinterSetupWindowPositionLoaded(Some(Point::new(
        680.0, 560.0,
    ))));
    let setup_window = app.printer_setup_window_id.unwrap();
    assert_ne!(setup_window, printer_window);
    assert_eq!(app.printer_window.id, Some(printer_window));
    let setup = app.printer_setup_dialog.as_ref().unwrap();
    assert!(!setup.owner_ready);
    assert_eq!(setup.owner_position, Some(Point::new(680.0, 560.0)));

    let _ = app.update(Message::WindowOpened(setup_window));
    assert!(app.printer_setup_dialog.as_ref().unwrap().owner_ready);
}

#[test]
fn detached_printer_dialogs_are_centered_on_printer() {
    assert_eq!(
        detached_printer_setup_position(Point::new(680.0, 560.0)),
        Point::new(700.0, 480.0)
    );
    assert_eq!(
        detached_printer_properties_position(Point::new(680.0, 560.0)),
        Point::new(540.0, 390.0)
    );
}

#[test]
fn detached_properties_use_their_own_window() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.printer_open = true;
    app.printer_window.detached = true;
    let printer_window = iced::window::Id::unique();
    let setup_window = iced::window::Id::unique();
    app.printer_window.id = Some(printer_window);
    app.printer_setup_window_id = Some(setup_window);

    let mut setup = PrinterSetupDialog::new(PrinterSetupTarget::Session, None);
    setup.owner_position = Some(Point::new(680.0, 560.0));
    setup.properties = Some(PrinterPropertiesDialog::new(String::new(), Vec::new()));
    setup.properties_surface_ready = false;
    app.printer_setup_dialog = Some(setup);

    let _ = app.open_detached_printer_properties_window();
    let properties_window = app.printer_properties_window_id.unwrap();
    assert_ne!(properties_window, setup_window);
    assert_ne!(properties_window, printer_window);
    assert_eq!(app.printer_setup_window_id, Some(setup_window));
    assert_eq!(app.printer_window.id, Some(printer_window));

    let _ = app.update(Message::WindowOpened(properties_window));
    assert!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties_surface_ready
    );

    let _ = app.update(Message::WindowCloseRequested(setup_window));
    assert_eq!(app.printer_setup_window_id, Some(setup_window));
    assert_eq!(app.printer_properties_window_id, Some(properties_window));
    assert!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .is_some()
    );

    let _ = app.update(Message::ClosePrinterProperties);
    assert_eq!(app.printer_properties_window_id, None);
    assert_eq!(app.printer_setup_window_id, Some(setup_window));
    assert!(
        app.printer_setup_dialog
            .as_ref()
            .unwrap()
            .properties
            .is_none()
    );
}

#[test]
fn property_attention_pulse_rises_fades_and_expires() {
    let started_at = Instant::now();
    let mut properties = PrinterPropertiesDialog::new(String::new(), Vec::new());
    properties.restart_attention(started_at);

    assert_eq!(properties.attention_strength(started_at), 0.0);
    let rising = properties.attention_strength(started_at + Duration::from_millis(130));
    let peak = properties.attention_strength(started_at + Duration::from_millis(260));
    let falling = properties.attention_strength(started_at + Duration::from_millis(390));
    assert!((0.70..0.71).contains(&rising));
    assert!(peak > 0.99);
    assert!((0.70..0.71).contains(&falling));

    properties.expire_attention(started_at + Duration::from_millis(520));
    assert_eq!(properties.attention_strength(started_at), 0.0);
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
