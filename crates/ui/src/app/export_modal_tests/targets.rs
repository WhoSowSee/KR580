use super::*;

#[test]
fn text_tab_uses_separate_section_list_for_export_target() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTabSelected(ExportTab::Text));
    let _task = app.update(Message::ExportTargetChanged("Раздел X".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Раздел X");
    assert!(
        app.export
            .export_text_sections
            .iter()
            .any(|target| target.name == "Раздел X")
    );
    assert!(
        !app.export
            .export_xlsx_pages
            .iter()
            .any(|target| target.name == "Раздел X")
    );
    assert!(!app.export.export_target_dropdown.is_open());
}

#[test]
fn adding_existing_export_target_without_open_dropdown_keeps_dropdown_closed() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Подпрограмма 2");
    assert!(
        app.export
            .export_xlsx_pages
            .iter()
            .any(|target| target.name == "Подпрограмма 2")
    );
    assert!(!app.export.export_target_dropdown.is_open());
    assert_eq!(app.export.export_target_dropdown.highlight(), None);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::Page);
}

#[test]
fn adding_existing_export_target_with_open_dropdown_keeps_dropdown_open() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetDropdownToggled);
    let _task = app.update(Message::ExportTargetAdd);

    assert_eq!(app.export_target_input(), "Подпрограмма 2");
    assert!(
        app.export
            .export_xlsx_pages
            .iter()
            .any(|target| target.name == "Подпрограмма 2")
    );
    assert!(app.export.export_target_dropdown.is_open());
    assert_eq!(app.export.export_target_dropdown.highlight(), Some(1));
    assert_eq!(
        app.export.export_modal_focus,
        ExportModalFocus::TargetDropdown
    );
}

#[test]
fn deleting_export_target_falls_back_to_remaining_session_entry() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetChanged("Лист 2".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportTargetDelete);

    assert_eq!(app.export_target_input(), "Подпрограмма 1");
    assert!(
        !app.export
            .export_xlsx_pages
            .iter()
            .any(|target| target.name == "Лист 2")
    );
    assert!(!app.export.export_target_dropdown.is_open());
    assert_eq!(app.export.export_target_dropdown.highlight(), None);
    assert_eq!(app.export.export_modal_focus, ExportModalFocus::Page);
}

#[test]
fn deleting_export_target_with_open_dropdown_keeps_dropdown_open() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();

    let _task = app.update(Message::ExportTargetChanged("Лист 2".to_owned()));
    let _task = app.update(Message::ExportTargetAdd);
    let _task = app.update(Message::ExportTargetDropdownToggled);
    let _task = app.update(Message::ExportTargetDelete);

    assert_eq!(app.export_target_input(), "Подпрограмма 1");
    assert!(
        !app.export
            .export_xlsx_pages
            .iter()
            .any(|target| target.name == "Лист 2")
    );
    assert!(app.export.export_target_dropdown.is_open());
    assert_eq!(app.export.export_target_dropdown.highlight(), Some(0));
    assert_eq!(
        app.export.export_modal_focus,
        ExportModalFocus::TargetDropdown
    );
}

#[test]
fn export_options_parse_range_and_selected_registers() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.apply_language(Lang::Ru);
    app.open_export_modal();
    app.export.export_memory_start_input = "0010".to_owned();
    app.export.export_memory_end_input = "001F".to_owned();
    app.export.export_registers = ExportRegisterSelection {
        accumulator: true,
        b: true,
        ..ExportRegisterSelection::default()
    };
    app.export.export_registers.c = false;
    app.export.export_flags = ExportFlagSelection {
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
    app.apply_language(Lang::Ru);
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
    app.apply_language(Lang::Ru);
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
