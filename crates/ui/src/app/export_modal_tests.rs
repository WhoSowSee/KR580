use super::{
    DesktopApp, ExportFlag, ExportFlagSelection, ExportMemoryColumn, ExportModalFocus,
    ExportRegister, ExportRegisterSelection, ExportTab,
};
use crate::app::Message;
use crate::persistence::{ExportFlagKind, ExportRegisterKind};

#[test]
fn export_opens_excel_tab_with_full_range_defaults() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    let _task = app.update(Message::Export);

    assert!(app.export_modal_open);
    assert_eq!(app.export_tab, ExportTab::Xlsx);
    assert_eq!(app.export_modal_focus, ExportModalFocus::TabXlsx);
    assert_eq!(app.export_memory_start_input, "0000");
    assert_eq!(app.export_memory_end_input, "FFFF");
    assert!(!app.export_registers.accumulator);
    assert!(!app.export_registers.b);
    assert!(!app.export_registers.stack_pointer);
    assert!(!app.export_flags.sign);
    assert!(!app.export_flags.zero);
    assert!(!app.export_flags.carry);
}

#[test]
fn tab_cycles_export_modal_focus_through_tabs_and_settings() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.export_modal_focus, ExportModalFocus::TabText);

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.export_modal_focus, ExportModalFocus::Page);

    let _task = app.update(Message::FocusCycle { backward: true });
    assert_eq!(app.export_modal_focus, ExportModalFocus::TabText);
}

#[test]
fn selecting_text_tab_changes_active_tab_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTabSelected(ExportTab::Text));

    assert!(app.export_modal_open);
    assert_eq!(app.export_tab, ExportTab::Text);
    assert_eq!(app.export_modal_focus, ExportModalFocus::TabText);
}

#[test]
fn toggling_register_updates_export_selection() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportRegister(ExportRegister::B));

    assert!(app.export_registers.b);
    assert_eq!(app.export_modal_focus, ExportModalFocus::RegisterB);
}

#[test]
fn toggling_flag_updates_export_selection() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportFlag(ExportFlag::Zero));

    assert!(app.export_flags.zero);
    assert_eq!(app.export_modal_focus, ExportModalFocus::FlagZero);
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
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::EscPressed);

    assert!(app.export_modal_open);
    assert_eq!(app.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn esc_clears_export_checkbox_focus_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ToggleExportFlag(ExportFlag::Zero));
    let _task = app.update(Message::EscPressed);

    assert!(app.export_modal_open);
    assert_eq!(app.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn mouse_press_clears_export_value_focus_without_closing_modal() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::MousePressedIgnored);

    assert!(app.export_modal_open);
    assert_eq!(app.export_modal_focus, ExportModalFocus::None);
}

#[test]
fn captured_mouse_press_keeps_export_value_focus() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::MousePressed);

    assert!(app.export_modal_open);
    assert_eq!(app.export_modal_focus, ExportModalFocus::MemoryStart);
}

#[test]
fn esc_closes_export_modal_without_value_focus() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::EscPressed);

    assert!(!app.export_modal_open);
}

#[test]
fn text_tab_uses_separate_section_list_for_export_target() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTabSelected(ExportTab::Text));
    let _task = app.update(Message::ExportTargetChanged("Раздел X".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Раздел X");
    assert!(app.export_text_sections.contains(&"Раздел X".to_owned()));
    assert!(!app.export_xlsx_pages.contains(&"Раздел X".to_owned()));
    assert!(!app.export_target_dropdown_open);
}

#[test]
fn adding_existing_export_target_without_open_dropdown_keeps_dropdown_closed() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Подпрограмма 2");
    assert!(app.export_xlsx_pages.contains(&"Подпрограмма 2".to_owned()));
    assert!(!app.export_target_dropdown_open);
    assert_eq!(app.export_target_highlight, None);
    assert_eq!(app.export_modal_focus, ExportModalFocus::Page);
}

#[test]
fn adding_existing_export_target_with_open_dropdown_keeps_dropdown_open() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetDropdownToggled);
    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Подпрограмма 2");
    assert!(app.export_xlsx_pages.contains(&"Подпрограмма 2".to_owned()));
    assert!(app.export_target_dropdown_open);
    assert_eq!(app.export_target_highlight, Some(1));
    assert_eq!(app.export_modal_focus, ExportModalFocus::TargetDropdown);
}

#[test]
fn deleting_export_target_falls_back_to_remaining_session_entry() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetChanged("Лист 2".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportTargetDelete);

    assert_eq!(app.export_target_input(), "Подпрограмма 1");
    assert!(!app.export_xlsx_pages.contains(&"Лист 2".to_owned()));
    assert!(!app.export_target_dropdown_open);
    assert_eq!(app.export_target_highlight, None);
    assert_eq!(app.export_modal_focus, ExportModalFocus::Page);
}

#[test]
fn deleting_export_target_with_open_dropdown_keeps_dropdown_open() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetChanged("Лист 2".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportTargetDropdownToggled);
    let _task = app.update(Message::ExportTargetDelete);

    assert_eq!(app.export_target_input(), "Подпрограмма 1");
    assert!(!app.export_xlsx_pages.contains(&"Лист 2".to_owned()));
    assert!(app.export_target_dropdown_open);
    assert_eq!(app.export_target_highlight, Some(0));
    assert_eq!(app.export_modal_focus, ExportModalFocus::TargetDropdown);
}

#[test]
fn export_options_parse_range_and_selected_registers() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();
    app.export_memory_start_input = "0010".to_owned();
    app.export_memory_end_input = "001F".to_owned();
    app.export_registers = ExportRegisterSelection {
        accumulator: true,
        b: true,
        ..ExportRegisterSelection::default()
    };
    app.export_registers.c = false;
    app.export_flags = ExportFlagSelection {
        zero: true,
        carry: true,
        ..ExportFlagSelection::default()
    };

    let options = app.export_options();

    assert_eq!(options.memory_start, 0x0010);
    assert_eq!(options.memory_end, 0x001F);
    assert!(options.registers.contains(&ExportRegisterKind::Accumulator));
    assert!(options.registers.contains(&ExportRegisterKind::B));
    assert!(!options.registers.contains(&ExportRegisterKind::C));
    assert!(options.flags.contains(&ExportFlagKind::Zero));
    assert!(options.flags.contains(&ExportFlagKind::Carry));
    assert!(!options.flags.contains(&ExportFlagKind::Sign));
}

#[test]
fn text_export_options_include_all_session_sections_with_own_ranges() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportTabSelected(ExportTab::Text));
    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::ExportMemoryEndChanged("0101".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportMemoryStartChanged("0200".to_owned()));
    let _task = app.update(Message::ExportMemoryEndChanged("0202".to_owned()));

    let options = app.export_options();

    assert_eq!(options.text_sections.len(), 2);
    assert_eq!(options.text_sections[0].name, "Раздел 1");
    assert_eq!(options.text_sections[0].memory_start, 0x0100);
    assert_eq!(options.text_sections[0].memory_end, 0x0101);
    assert_eq!(options.text_sections[1].name, "Раздел 2");
    assert_eq!(options.text_sections[1].memory_start, 0x0200);
    assert_eq!(options.text_sections[1].memory_end, 0x0202);
}

#[test]
fn xlsx_export_options_include_all_session_pages_with_own_ranges() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_export_modal();

    let _task = app.update(Message::ExportMemoryStartChanged("0100".to_owned()));
    let _task = app.update(Message::ExportMemoryEndChanged("0101".to_owned()));
    let _task = app.update(Message::ToggleExportMemoryColumn(
        ExportMemoryColumn::Comment,
    ));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportMemoryStartChanged("0200".to_owned()));
    let _task = app.update(Message::ExportMemoryEndChanged("0202".to_owned()));

    let options = app.export_options();

    assert_eq!(options.xlsx_pages.len(), 2);
    assert_eq!(options.xlsx_pages[0].name, "Подпрограмма 1");
    assert_eq!(options.xlsx_pages[0].memory_start, 0x0100);
    assert_eq!(options.xlsx_pages[0].memory_end, 0x0101);
    assert!(options.xlsx_pages[0].include_comment_column);
    assert_eq!(options.xlsx_pages[1].name, "Подпрограмма 2");
    assert_eq!(options.xlsx_pages[1].memory_start, 0x0200);
    assert_eq!(options.xlsx_pages[1].memory_end, 0x0202);
    assert!(options.text_sections.is_empty());
}
