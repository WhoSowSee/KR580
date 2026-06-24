use super::{DesktopApp, ImportFileFormat, ImportModalFocus};
use crate::app::{MEMORY_SCROLL_VISIBLE_TICKS, Message};
use crate::persistence::{ExportModel, ExportOptions, Exporters};
use std::path::PathBuf;

#[test]
fn import_opens_modal_instead_of_file_dialog_path() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    let _task = app.update(Message::Import);

    assert!(app.import_modal_open);
    assert_eq!(app.import_modal_focus, ImportModalFocus::Browse);
    assert!(app.import_file_path.is_none());
    assert!(app.import_target_options.is_empty());
}

#[test]
fn esc_closes_import_modal_without_focus_clear_step() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    app.open_import_modal();
    app.import_modal_focus = ImportModalFocus::Target;
    app.import_target_dropdown_open = true;
    let _task = app.update(Message::EscPressed);

    assert!(!app.import_modal_open);
    assert!(!app.import_target_dropdown_open);
}

#[test]
fn import_target_dropdown_reveals_scrollbar_then_hides_until_scrolled() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    app.open_import_modal();
    app.import_target_options = vec!["1".to_owned(), "2".to_owned(), "3".to_owned()];
    app.import_target_input = "1".to_owned();
    let _task = app.update(Message::ImportTargetDropdownToggled);

    assert!(app.import_target_dropdown_open);
    assert_eq!(
        app.import_target_scroll_visible_ticks,
        MEMORY_SCROLL_VISIBLE_TICKS
    );
    for _ in 0..MEMORY_SCROLL_VISIBLE_TICKS {
        let _task = app.update(Message::Tick);
    }
    assert_eq!(app.import_target_scroll_visible_ticks, 0);

    let _task = app.update(Message::ImportTargetScrolled);

    assert_eq!(
        app.import_target_scroll_visible_ticks,
        MEMORY_SCROLL_VISIBLE_TICKS
    );
}

#[test]
fn loading_xlsx_import_file_populates_sheet_targets() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("import-pages.xlsx");
    Exporters::write_xlsx_pages(
        &path,
        &[
            (
                "Подпрограмма 1".to_owned(),
                model_at(0x0100),
                ExportOptions::default(),
            ),
            (
                "Подпрограмма 2".to_owned(),
                model_at(0x0200),
                ExportOptions::default(),
            ),
        ],
    )
    .unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());

    assert_eq!(app.import_file_path, Some(path.clone()));
    assert_eq!(app.import_file_format, Some(ImportFileFormat::Xlsx));
    assert_eq!(
        app.import_target_options,
        vec!["Подпрограмма 1".to_owned(), "Подпрограмма 2".to_owned()]
    );
    assert_eq!(app.import_target_input, "Подпрограмма 1");
    assert_eq!(app.import_modal_focus, ImportModalFocus::Target);
    assert!(app.import_error.is_none());
    std::fs::remove_file(path).ok();
}

#[test]
fn loading_txt_import_file_populates_section_targets_when_present() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("import-sections.txt");
    std::fs::write(
        &path,
        Exporters::to_text_sections(&[
            ("Раздел 1".to_owned(), model_at(0x0100)),
            ("Раздел 2".to_owned(), model_at(0x0200)),
        ]),
    )
    .unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());

    assert_eq!(app.import_file_format, Some(ImportFileFormat::Text));
    assert_eq!(
        app.import_target_options,
        vec!["Раздел 1".to_owned(), "Раздел 2".to_owned()]
    );
    assert_eq!(app.import_target_input, "Раздел 1");
    std::fs::remove_file(path).ok();
}

#[test]
fn confirming_xlsx_import_applies_selected_sheet() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("confirm-pages.xlsx");
    Exporters::write_xlsx_pages(
        &path,
        &[
            (
                "Подпрограмма 1".to_owned(),
                model_at(0x0100),
                ExportOptions::default(),
            ),
            (
                "Подпрограмма 2".to_owned(),
                model_at(0x0200),
                ExportOptions::default(),
            ),
        ],
    )
    .unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());
    app.import_target_input = "Подпрограмма 2".to_owned();
    let _task = app.confirm_import();

    assert!(!app.import_modal_open);
    assert_eq!(app.snapshot.cpu.memory.read(0x0100), 0x00);
    assert_eq!(app.snapshot.cpu.memory.read(0x0200), 0xAA);
    std::fs::remove_file(path).ok();
}

#[test]
fn confirming_plain_txt_import_applies_whole_file_without_targets() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("confirm-plain.txt");
    std::fs::write(&path, Exporters::to_text(&model_at(0x0300))).unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());
    let _task = app.confirm_import();

    assert!(app.import_target_options.is_empty());
    assert_eq!(app.snapshot.cpu.memory.read(0x0300), 0xAA);
    std::fs::remove_file(path).ok();
}

#[test]
fn confirming_malformed_txt_import_sets_localized_status() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("malformed-import.txt");
    std::fs::write(&path, "this is not a KR580 export").unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());
    let _task = app.confirm_import();

    assert_eq!(app.status, "Не удалось прочитать файл – проверьте формат");
    assert!(
        app.error_notice
            .as_deref()
            .is_some_and(|notice| notice.contains(&app.status))
    );
    std::fs::remove_file(path).ok();
}

fn model_at(address: u16) -> ExportModel {
    ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(address, 0xAA)],
    }
}

fn unique_temp_file(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{nanos}-{name}"))
}
