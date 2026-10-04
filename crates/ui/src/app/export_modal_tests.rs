use super::{
    DesktopApp, ExportFlag, ExportFlagSelection, ExportMemoryColumn, ExportModalFocus,
    ExportRegister, ExportRegisterSelection, ExportTab,
};
use crate::app::Message;
use crate::i18n::Lang;
use crate::persistence::{ExportFlagKind, ExportRegisterKind};

#[test]
fn export_opens_with_defaults_and_can_switch_to_import() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    let _task = app.update(Message::OpenMonitor);

    let _task = app.update(Message::Export);

    assert!(app.export.export_modal_open);
    assert!(!app.panels.monitor_open);
    assert_eq!(app.export.export_tab, ExportTab::Xlsx);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::TabXlsx);
    assert_eq!(app.export.export_memory_start_input, "0000");
    assert_eq!(app.export.export_memory_end_input, "FFFF");
    assert!(!app.export.export_registers.accumulator);
    assert!(!app.export.export_registers.b);
    assert!(!app.export.export_registers.stack_pointer);
    assert!(!app.export.export_flags.sign);
    assert!(!app.export.export_flags.zero);
    assert!(!app.export.export_flags.carry);

    let _task = app.update(Message::Import);
    assert!(!app.export.export_modal_open);
    assert!(app.import.import_modal_open);
    assert!(!app.panels.monitor_open);

    let _task = app.update(Message::CancelImport);

    assert!(!app.import.import_modal_open);
    assert!(!app.panels.monitor_open);
}

#[test]
fn language_change_relocalizes_generated_export_targets() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.preferences.lang = Lang::Ru;
    app.export.export_xlsx_pages = ["Подпрограмма 1", "Подпрограмма 2", "Отчёт"]
        .into_iter()
        .map(|name| super::ExportTarget::named(name.to_owned()))
        .collect();
    app.export.export_xlsx_page_input = "Подпрограмма 2".to_owned();
    app.export.export_text_sections = ["Раздел 1", "Данные"]
        .into_iter()
        .map(|name| super::ExportTarget::named(name.to_owned()))
        .collect();
    app.export.export_text_section_input = "Данные".to_owned();

    let _task = app.update(Message::SettingsDraftLanguageChanged(Lang::En));

    assert_eq!(
        app.export
            .export_xlsx_pages
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        ["Subprogram 1", "Subprogram 2", "Отчёт"]
    );
    assert_eq!(app.export.export_xlsx_page_input, "Subprogram 2");
    assert_eq!(
        app.export
            .export_text_sections
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        ["Section 1", "Данные"]
    );
    assert_eq!(app.export.export_text_section_input, "Данные");

    let _task = app.update(Message::SettingsDraftLanguageChanged(Lang::Ru));

    assert_eq!(
        app.export
            .export_xlsx_pages
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        ["Подпрограмма 1", "Подпрограмма 2", "Отчёт"]
    );
    assert_eq!(app.export.export_xlsx_page_input, "Подпрограмма 2");
    assert_eq!(
        app.export
            .export_text_sections
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        ["Раздел 1", "Данные"]
    );
    assert_eq!(app.export.export_text_section_input, "Данные");
}

#[test]
fn tab_cycles_export_modal_focus_through_tabs_and_settings() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::TabText);
    assert!(app.export.export_modal_keyboard_focus_visible);

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(
        app.export.export_modal_focus,
        ExportModalFocus::TargetDropdown
    );

    let _task = app.update(Message::FocusCycle { backward: true });
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::TabText);
}

#[test]
fn selecting_text_tab_changes_active_tab_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTabSelected(ExportTab::Text));

    assert!(app.export.export_modal_open);
    assert_eq!(app.export.export_tab, ExportTab::Text);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::TabText);
    assert!(!app.export.export_modal_keyboard_focus_visible);
}

#[test]
fn toggling_register_updates_export_selection() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportRegister(ExportRegister::B));

    assert!(app.export.export_registers.b);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::RegisterB);
}

#[test]
fn toggling_flag_updates_export_selection() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportFlag(ExportFlag::Zero));

    assert!(app.export.export_flags.zero);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::FlagZero);
}

#[test]
fn selected_flags_follow_visible_flag_strip_order() {
    let flags = ExportFlagSelection {
        sign: true,
        zero: true,
        auxiliary_carry: true,
        parity: true,
        carry: true,
    };

    assert_eq!(
        flags.selected(),
        vec![
            ExportFlagKind::Zero,
            ExportFlagKind::Sign,
            ExportFlagKind::Parity,
            ExportFlagKind::Carry,
            ExportFlagKind::AuxiliaryCarry,
        ]
    );
}

#[test]
fn esc_clears_export_input_focus_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::EscPressed);

    assert!(app.export.export_modal_open);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn esc_clears_export_checkbox_focus_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportFlag(ExportFlag::Zero));
    let _task = app.update(Message::EscPressed);

    assert!(app.export.export_modal_open);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn mouse_press_clears_export_value_focus_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::MousePressedIgnored);

    assert!(app.export.export_modal_open);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn captured_mouse_press_keeps_export_value_focus() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::MousePressed);

    assert!(app.export.export_modal_open);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::MemoryStart);
}

#[test]
fn esc_closes_export_modal_without_value_focus() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::EscPressed);

    assert!(!app.export.export_modal_open);
}

mod targets;
