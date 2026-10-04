use crate::app::settings_modal::SettingsDialog;
use crate::app::{DesktopApp, Message};
use crate::persistence::NetworkSettings;
use k580_ui::devices::printer::PrinterSettings;

#[test]
fn settings_printer_setup_completion_updates_draft_and_clears_pending() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.printer_setup.printer_setup_pending = true;
    app.preferences.settings_dialog = Some(SettingsDialog::new(
        crate::app::settings_modal::SettingsInitialState {
            lang: app.preferences.lang,
            speed: app.preferences.default_speed,
            follow_pc: true,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: NetworkSettings::default(),
            ..Default::default()
        },
    ));

    let _ = app.update(Message::SettingsPrinterSetupFinished(Ok(Some(
        PrinterSettings::named("HP Laser MFP 131 133 135-139".to_owned()),
    ))));

    let dialog = app.preferences.settings_dialog.as_ref().unwrap();
    assert_eq!(
        dialog
            .draft_printer_settings
            .as_ref()
            .map(|settings| settings.printer_name.as_str()),
        Some("HP Laser MFP 131 133 135-139")
    );
    assert!(!app.printer_setup.printer_setup_pending);
}

#[test]
fn session_printer_setup_completion_updates_session_and_clears_pending() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.printer_setup.printer_setup_pending = true;

    let _ = app.update(Message::PrinterSessionSetupFinished(Ok(Some(
        PrinterSettings::named("HP Laser MFP 131 133 135-139".to_owned()),
    ))));

    assert_eq!(
        app.printer_setup
            .printer_session_settings
            .as_ref()
            .map(|settings| settings.printer_name.as_str()),
        Some("HP Laser MFP 131 133 135-139")
    );
    assert!(!app.printer_setup.printer_setup_pending);
}
