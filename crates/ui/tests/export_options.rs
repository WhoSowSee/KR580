use calamine::{Data, Reader, open_workbook_auto};
use k580_core::{Cpu8080State, Flags};
use k580_ui::persistence::{
    ExportFlagKind, ExportModel, ExportOptions, ExportRegisterKind, ExportXlsxPage, Exporters,
    Importers,
};
use std::path::PathBuf;

#[test]
fn export_options_filter_memory_range_and_registers() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x11;
    cpu.registers.w = 0x22;
    cpu.registers.z = 0x33;
    cpu.flags.zero = true;
    cpu.flags.carry = true;
    cpu.pc = 0x1234;
    cpu.memory.write(0x000F, 0xAA);
    cpu.memory.write(0x0010, 0xBB);
    cpu.memory.write(0x0011, 0xCC);

    let options = ExportOptions {
        memory_start: 0x0010,
        memory_end: 0x0010,
        registers: vec![
            ExportRegisterKind::W,
            ExportRegisterKind::Z,
            ExportRegisterKind::ProgramCounter,
        ],
        flags: vec![ExportFlagKind::Zero, ExportFlagKind::Carry],
        ..ExportOptions::default()
    };

    let model = ExportModel::from_cpu_with_options(&cpu, &options);

    assert_eq!(
        model.registers,
        vec![
            ("W".to_owned(), "22".to_owned()),
            ("Z".to_owned(), "33".to_owned()),
            ("PC".to_owned(), "1234".to_owned()),
        ]
    );
    assert_eq!(
        model.flags,
        vec![("Z".to_owned(), true), ("C".to_owned(), true)]
    );
    assert_eq!(model.memory, vec![(0x0010, 0xBB)]);
}

#[test]
fn default_export_flags_follow_visible_flag_strip_order() {
    let options = ExportOptions::default();

    assert_eq!(
        options.flags,
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
fn xlsx_options_set_sheet_name_and_optional_memory_columns() {
    let model = ExportModel {
        registers: vec![("A".to_owned(), "42".to_owned())],
        flags: vec![("Z".to_owned(), true)],
        memory: vec![(0x0000, 0x76)],
    };
    let options = ExportOptions {
        page_name: "Page:1".to_owned(),
        include_memory_command: true,
        include_comment_column: true,
        ..ExportOptions::default()
    };
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let xlsx = dir.join("state.xlsx");

    Exporters::write_xlsx_with_options(&xlsx, &model, &options).unwrap();

    let mut workbook = open_workbook_auto(&xlsx).unwrap();
    assert_eq!(workbook.sheet_names().first().unwrap(), "Page_1");
    let sheet = workbook.worksheet_range("Page_1").unwrap();
    let memory_header = sheet
        .rows()
        .find(|row| row.first().is_some_and(|cell| cell_text(cell) == "Address"))
        .unwrap();
    assert_eq!(cell_text(&memory_header[1]), "Value");
    assert_eq!(cell_text(&memory_header[2]), "Command");
    assert_eq!(cell_text(&memory_header[3]), "Comment");
    assert_eq!(Importers::read_xlsx(&xlsx).unwrap(), model);

    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn txt_sections_are_written_as_named_subprogram_blocks() {
    let first = ExportModel {
        registers: vec![("A".to_owned(), "11".to_owned())],
        flags: vec![("Z".to_owned(), true)],
        memory: vec![(0x0001, 0xAA)],
    };
    let second = ExportModel {
        registers: vec![("B".to_owned(), "22".to_owned())],
        flags: vec![("C".to_owned(), false)],
        memory: vec![(0x0100, 0xBB)],
    };

    let text = Exporters::to_text_sections(&[
        ("Подпрограмма 1".to_owned(), first),
        ("Подпрограмма 2".to_owned(), second),
    ]);

    assert!(text.starts_with("[Подпрограмма 1]\n[Registers]\nA=11\n"));
    assert!(text.contains("\n[Подпрограмма 2]\n[Registers]\nB=22\n"));
    assert_eq!(text.matches("[Registers]").count(), 2);
    assert_eq!(text.matches("[Memory]").count(), 2);
}

#[test]
fn xlsx_pages_are_written_as_separate_worksheets() {
    let first = ExportModel {
        registers: vec![("A".to_owned(), "11".to_owned())],
        flags: vec![("Z".to_owned(), true)],
        memory: vec![(0x0001, 0xAA)],
    };
    let second = ExportModel {
        registers: vec![("B".to_owned(), "22".to_owned())],
        flags: vec![("C".to_owned(), false)],
        memory: vec![(0x0100, 0xBB)],
    };
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let xlsx = dir.join("pages.xlsx");

    Exporters::write_xlsx_pages(
        &xlsx,
        &[
            ("Page:1".to_owned(), first, ExportOptions::default()),
            ("Page:2".to_owned(), second, ExportOptions::default()),
        ],
    )
    .unwrap();

    let mut workbook = open_workbook_auto(&xlsx).unwrap();
    assert_eq!(workbook.sheet_names(), &["Page_1", "Page_2"]);
    let first_sheet = workbook.worksheet_range("Page_1").unwrap();
    let second_sheet = workbook.worksheet_range("Page_2").unwrap();
    assert!(first_sheet.rows().any(|row| {
        row.first().is_some_and(|cell| cell_text(cell) == "0001")
            && row.get(1).is_some_and(|cell| cell_text(cell) == "AA")
    }));
    assert!(second_sheet.rows().any(|row| {
        row.first().is_some_and(|cell| cell_text(cell) == "0100")
            && row.get(1).is_some_and(|cell| cell_text(cell) == "BB")
    }));

    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn xlsx_sheet_names_and_selected_sheet_import_are_available() {
    let first = ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0100, 0xAA)],
    };
    let second = ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0200, 0xBB)],
    };
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let xlsx = dir.join("pages.xlsx");

    Exporters::write_xlsx_pages(
        &xlsx,
        &[
            ("Подпрограмма 1".to_owned(), first, ExportOptions::default()),
            (
                "Подпрограмма 2".to_owned(),
                second,
                ExportOptions::default(),
            ),
        ],
    )
    .unwrap();

    assert_eq!(
        Importers::xlsx_sheet_names(&xlsx).unwrap(),
        vec!["Подпрограмма 1", "Подпрограмма 2"]
    );
    assert_eq!(
        Importers::read_xlsx_sheet(&xlsx, "Подпрограмма 2")
            .unwrap()
            .memory,
        vec![(0x0200, 0xBB)]
    );

    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn txt_section_names_and_selected_section_import_are_available() {
    let first = ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0100, 0xAA)],
    };
    let second = ExportModel {
        registers: Vec::new(),
        flags: Vec::new(),
        memory: vec![(0x0200, 0xBB)],
    };
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("sections.txt");
    std::fs::write(
        &txt,
        Exporters::to_text_sections(&[
            ("Раздел 1".to_owned(), first),
            ("Раздел 2".to_owned(), second),
        ]),
    )
    .unwrap();

    assert_eq!(
        Importers::txt_section_names(&txt).unwrap(),
        vec!["Раздел 1", "Раздел 2"]
    );
    assert_eq!(
        Importers::read_txt_section(&txt, "Раздел 2")
            .unwrap()
            .memory,
        vec![(0x0200, 0xBB)]
    );

    std::fs::remove_file(txt).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn xlsx_page_options_keep_comment_column_per_page() {
    let page = ExportXlsxPage {
        name: "Sheet".to_owned(),
        memory_start: 0x0100,
        memory_end: 0x0101,
        include_memory_address: true,
        include_memory_value: true,
        include_memory_command: true,
        include_comment_column: true,
        registers: vec![ExportRegisterKind::Accumulator],
        flags: vec![ExportFlagKind::Zero],
    };

    let options = page.to_options();

    assert_eq!(options.page_name, "Sheet");
    assert_eq!(options.memory_start, 0x0100);
    assert!(options.include_comment_column);
    assert_eq!(options.registers, vec![ExportRegisterKind::Accumulator]);
    assert_eq!(options.flags, vec![ExportFlagKind::Zero]);
}

#[test]
fn exporters_write_stable_direct_files() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x42;
    cpu.memory.write(0, 0x76);
    let model = ExportModel::from_cpu(&cpu);
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("state.txt");
    let xlsx = dir.join("state.xlsx");

    Exporters::write_txt(&txt, &model).unwrap();
    Exporters::write_xlsx(&xlsx, &model).unwrap();

    let text = std::fs::read_to_string(&txt).unwrap();
    assert!(text.contains("[Registers]"));
    assert!(text.contains("A=42"));
    assert!(std::fs::metadata(&xlsx).unwrap().len() > 0);

    std::fs::remove_file(txt).ok();
    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

#[test]
fn importers_round_trip_txt_and_xlsx() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0xA5;
    cpu.registers.b = 0x12;
    cpu.registers.c = 0x34;
    cpu.pc = 0x1234;
    cpu.sp = 0xABCD;
    cpu.flags = Flags {
        sign: true,
        zero: false,
        auxiliary_carry: true,
        parity: false,
        carry: true,
    };
    cpu.memory.write(0, 0x76);
    cpu.memory.write(1, 0xC3);
    cpu.memory.write(2, 0x00);
    cpu.memory.write(3, 0x10);
    cpu.cycle_count = 4242;

    let model = ExportModel::from_cpu(&cpu);
    let dir = unique_temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("state.txt");
    let xlsx = dir.join("state.xlsx");

    Exporters::write_txt(&txt, &model).unwrap();
    Exporters::write_xlsx(&xlsx, &model).unwrap();

    let from_txt = Importers::read_txt(&txt).unwrap();
    assert_eq!(from_txt, model);

    let from_xlsx = Importers::read_xlsx(&xlsx).unwrap();
    assert_eq!(from_xlsx, model);

    let mut restored = Cpu8080State::default();
    from_txt.apply_to(&mut restored).unwrap();
    assert_eq!(restored.registers, cpu.registers);
    assert_eq!(restored.flags, cpu.flags);
    assert_eq!(restored.pc, cpu.pc);
    assert_eq!(restored.sp, cpu.sp);
    assert_eq!(restored.cycle_count, cpu.cycle_count);
    assert_eq!(restored.memory.read(0), 0x76);
    assert_eq!(restored.memory.read(3), 0x10);

    std::fs::remove_file(txt).ok();
    std::fs::remove_file(xlsx).ok();
    std::fs::remove_dir(dir).ok();
}

fn unique_temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("k580-export-options-{nanos}"))
}

fn cell_text(cell: &Data) -> &str {
    match cell {
        Data::String(value) => value,
        _ => "",
    }
}
