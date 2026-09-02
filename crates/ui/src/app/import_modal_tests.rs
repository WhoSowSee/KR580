use super::{DesktopApp, ImportFileFormat, ImportModalFocus};
use crate::app::Message;
use crate::persistence::{ExportModel, ExportOptions, Exporters};
use iced::{Event, window};
use std::path::PathBuf;

#[test]
fn import_opens_without_picker_and_can_switch_to_export() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let _task = app.update(Message::OpenMonitor);

    let _task = app.update(Message::Import);

    assert!(app.import_modal_open);
    assert!(!app.monitor_open);
    assert_eq!(app.import_modal_focus, ImportModalFocus::Browse);
    assert!(app.import_file_path.is_none());
    assert!(app.import_target_options.is_empty());

    let _task = app.update(Message::Export);
    assert!(!app.import_modal_open);
    assert!(app.export_modal_open);
    assert!(!app.monitor_open);

    let _task = app.update(Message::CancelExport);

    assert!(!app.export_modal_open);
    assert!(!app.monitor_open);
}

#[test]
fn tab_cycles_import_modal_focus_in_both_directions() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.open_import_modal();

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.import_modal_focus, ImportModalFocus::Cancel);
    assert!(app.import_modal_keyboard_focus_visible);

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.import_modal_focus, ImportModalFocus::Browse);

    let _task = app.update(Message::FocusCycle { backward: true });
    assert_eq!(app.import_modal_focus, ImportModalFocus::Cancel);
}

#[test]
fn confirm_focus_is_available_after_a_valid_file_is_loaded() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("focus-import.txt");
    std::fs::write(&path, Exporters::to_text(&model_at(0x0100))).unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());
    assert_eq!(app.import_modal_focus, ImportModalFocus::Confirm);

    let _task = app.update(Message::FocusCycle { backward: false });
    assert_eq!(app.import_modal_focus, ImportModalFocus::Browse);
    let _task = app.update(Message::FocusCycle { backward: true });
    assert_eq!(app.import_modal_focus, ImportModalFocus::Confirm);

    std::fs::remove_file(path).ok();
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
fn unsupported_import_file_keeps_the_modal_open_with_local_error() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let path = unique_temp_file("replace-import.txt");
    std::fs::write(&path, Exporters::to_text(&model_at(0x0100))).unwrap();

    app.open_import_modal();
    app.load_import_file(path.clone());
    app.load_import_file(PathBuf::from("program.bin"));

    assert!(app.import_modal_open);
    assert!(app.import_file_path.is_none());
    assert!(app.import_file_display.is_empty());
    assert!(app.import_file_format.is_none());
    assert!(app.import_target_options.is_empty());
    assert!(app.import_target_input.is_empty());
    assert_eq!(app.import_modal_focus, ImportModalFocus::Browse);
    assert_eq!(
        app.import_error.as_deref(),
        Some("Формат файла не поддерживается – используйте файл .txt или .xlsx")
    );
    std::fs::remove_file(path).ok();
}

#[test]
fn import_modal_owns_file_drag_hover_and_drop_events() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let main = window::Id::unique();
    let path = unique_temp_file("dropped-import.txt");
    std::fs::write(&path, Exporters::to_text(&model_at(0x0200))).unwrap();
    app.main_window_id = Some(main);
    app.open_import_modal();

    let hovered = Event::Window(window::Event::FileHovered(path.clone()));
    let _task = app.handle_file_drag_event(&hovered, main);
    assert!(app.import_file_drag_hovered);
    assert!(!app.file_drag_hovered);

    let left = Event::Window(window::Event::FilesHoveredLeft);
    let _task = app.handle_file_drag_event(&left, main);
    assert!(!app.import_file_drag_hovered);

    let dropped = Event::Window(window::Event::FileDropped(path.clone()));
    let _task = app.handle_file_drag_event(&dropped, main);
    assert!(!app.import_file_drag_hovered);
    assert_eq!(app.import_file_path, Some(path.clone()));
    assert_eq!(app.import_file_format, Some(ImportFileFormat::Text));
    assert!(app.error_notice.is_none());

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
