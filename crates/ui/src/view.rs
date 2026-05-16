use crate::app::{
    DesktopApp, MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID,
    MEMORY_OVERSCAN_ROWS, MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID,
    MEMORY_VALUE_INPUT_ID, Message, REGISTER_NAME_INPUT_ID, REGISTER_VALUE_INPUT_ID, register_name,
};
use iced::widget::{
    Column, Row, Space, Text, button, column, container, mouse_area, opaque, row, scrollable,
    stack, text, text_input,
};
use iced::{Background, Border, Color, Element, Font, Length, Padding, Theme, alignment};
use k580_core::{Cpu8080State, RegisterName, decode_opcode};

const UI_FONT: Font = Font::with_name("Segoe UI Variable");
const MONO_FONT: Font = Font::MONOSPACE;

const TOKYO_BG: Color = Color::from_rgb8(0x1A, 0x1B, 0x26);
const TOKYO_BOARD: Color = Color::from_rgb8(0x12, 0x13, 0x20);
const TOKYO_PANEL: Color = Color::from_rgb8(0x1F, 0x23, 0x35);
const TOKYO_SURFACE: Color = Color::from_rgb8(0x24, 0x28, 0x3B);
const TOKYO_SURFACE_2: Color = Color::from_rgb8(0x2F, 0x33, 0x4D);
const TOKYO_SURFACE_3: Color = Color::from_rgb8(0x36, 0x3B, 0x59);
const TOKYO_BORDER: Color = Color::from_rgb8(0x41, 0x48, 0x68);
const TOKYO_TEXT: Color = Color::from_rgb8(0xC0, 0xCA, 0xF5);
const TOKYO_MUTED: Color = Color::from_rgb8(0x56, 0x5F, 0x89);
const TOKYO_BLUE: Color = Color::from_rgb8(0x7A, 0xA2, 0xF7);
const TOKYO_CYAN: Color = Color::from_rgb8(0x7D, 0xCF, 0xFF);
const TOKYO_GREEN: Color = Color::from_rgb8(0x9E, 0xCE, 0x6A);
const TOKYO_YELLOW: Color = Color::from_rgb8(0xE0, 0xAF, 0x68);
const TOKYO_RED: Color = Color::from_rgb8(0xF7, 0x76, 0x8E);
const TOKYO_MAGENTA: Color = Color::from_rgb8(0xBB, 0x9A, 0xF7);

impl DesktopApp {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let main = row![self.schematic_panel(), self.side_panel()]
            .spacing(8)
            .height(Length::Fill);

        let content = column![self.menu_bar(), main, self.status_bar()]
            .padding(8)
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(app_style)
            .into()
    }

    fn menu_bar(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        container(
            row![
                ui_text("Emulator KR580VM80A", 14, TOKYO_MAGENTA),
                menu_label("File"),
                menu_label("MP-System"),
                menu_label("View"),
                menu_label("Settings"),
                menu_label("Help"),
                menu_action("Open", Message::OpenSnapshot),
                menu_action("Save", Message::SaveSnapshot),
                menu_action("TXT", Message::ExportTxt),
                menu_action("XLSX", Message::ExportXlsx),
                menu_action("DOCX", Message::ExportDocx),
                menu_action("Step", Message::StepInstruction),
                menu_action("Tact", Message::StepTact),
                menu_action("Run", Message::Run),
                menu_action("Stop", Message::Stop),
                menu_action("Reset", Message::ResetCpu),
                menu_action("RAM", Message::ResetRam),
                Space::new().width(Length::Fill),
                mono_text(format!("PC {:04X}", cpu.pc), 13, TOKYO_BLUE),
                mono_text(format!("SP {:04X}", cpu.sp), 13, TOKYO_CYAN),
                mono_text(format!("T {}", cpu.cycle_count), 13, TOKYO_YELLOW),
                mono_text(
                    if cpu.halted { "HALT ON" } else { "HALT OFF" },
                    13,
                    if cpu.halted { TOKYO_RED } else { TOKYO_GREEN },
                ),
            ]
            .spacing(18)
            .align_y(alignment::Vertical::Center),
        )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(menu_bar_style)
        .into()
    }

    fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        let top_bus_row = row![
            schematic_readout("PSW", format!("{:04X}", cpu.flags.to_psw()), TOKYO_GREEN),
            flag_strip(cpu),
            Space::new().width(Length::Fill),
            schematic_readout(
                "Data Buffer",
                format!("{:02X}", cpu.memory.read(cpu.pc)),
                TOKYO_GREEN,
            ),
        ]
        .spacing(18)
        .align_y(alignment::Vertical::Center);

        let alu = container(
            column![
                ui_text("ALU", 34, TOKYO_TEXT).font(MONO_FONT),
                row![
                    compact_value("A", format!("{:02X}", cpu.registers.a), TOKYO_GREEN),
                    compact_value("HL", format!("{:04X}", cpu.registers.hl()), TOKYO_CYAN),
                ]
                .spacing(8),
            ]
            .align_x(alignment::Horizontal::Center)
            .spacing(10),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fixed(86.0))
        .style(alu_style);

        let core_left = column![
            row![
                functional_block(
                    "Accumulator",
                    format!("{:02X}", cpu.registers.a),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::A),
                ),
                functional_block(
                    "Buf. Reg 1",
                    format!("{:02X}", cpu.registers.b),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::B),
                ),
                schematic_readout(
                    "Reg. Flags",
                    format!("{:06b}", cpu.flags.to_psw()),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
            row![
                functional_block(
                    "Buf. Reg 2",
                    format!("{:02X}", cpu.registers.c),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::C),
                ),
                alu,
                schematic_readout(
                    "Instr. Reg",
                    format!("{:02X}", cpu.memory.read(cpu.pc)),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
        ]
        .spacing(14)
        .width(Length::FillPortion(3));

        let multiplexer = mux_panel(cpu, self.selected_register);

        let core_plane = row![core_left, multiplexer]
            .spacing(16)
            .height(Length::FillPortion(2));

        let low_control = row![
            cycle_tick_panel(cpu),
            Space::new().width(Length::Fill),
            column![
                ui_text("Decimal Adjust", 12, TOKYO_MUTED),
                ui_text("Schema", 12, TOKYO_MUTED),
            ]
            .align_x(alignment::Horizontal::Center),
            Space::new().width(Length::Fill),
            schematic_readout("Sync & Control Block", "CTRL", TOKYO_TEXT),
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);

        let devices_state = &self.snapshot.devices;
        let device_entries: [(&'static str, &dyn std::fmt::Debug); 5] = [
            ("MON", &devices_state.monitor.status),
            ("FDD", &devices_state.floppy.status),
            ("HDD", &devices_state.hdd.status),
            ("NET", &devices_state.network.status),
            ("PRN", &devices_state.printer.status),
        ];

        let devices = Row::with_children(
            device_entries
                .iter()
                .map(|(label, status)| square_device(label, &format!("{status:?}")))
                .chain(std::iter::once(Space::new().width(Length::Fill).into()))
                .chain(std::iter::once(schematic_readout(
                    "I/O Controller",
                    "I/O",
                    TOKYO_TEXT,
                ))),
        )
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let bottom = column![
            low_control,
            control_lamps(),
            bus_bar("Address Bus A0-A15", TOKYO_MUTED),
            bus_bar("Control Bus", TOKYO_MUTED),
            devices,
        ]
        .spacing(10);

        let content = column![
            bus_bar("External Data Bus D7-D0", TOKYO_MUTED),
            top_bus_row,
            bus_bar("Internal Data Bus", TOKYO_MUTED),
            core_plane,
            bottom,
        ]
        .spacing(16)
        .height(Length::Fill);

        container(content)
            .padding(18)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(board_style)
            .into()
    }

    fn side_panel(&self) -> Element<'_, Message> {
        column![
            self.memory_panel(),
            self.memory_editor_panel(),
            self.register_editor_panel(),
        ]
        .spacing(8)
        .width(Length::Fixed(330.0))
        .height(Length::Fill)
        .into()
    }

    fn memory_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;
        let selected = parse_hex_u16_preview(&self.memory_address_input);
        let render_start =
            (self.memory_scroll_first_row as usize).saturating_sub(MEMORY_OVERSCAN_ROWS);
        let render_end = (render_start + MEMORY_RENDER_ROWS).min(MEMORY_ADDRESS_COUNT);
        let mut rows: Column<'_, Message> = Column::new().spacing(0);

        if render_start > 0 {
            rows = rows.push(memory_spacer(render_start));
        }

        for address in render_start..render_end {
            let address = address as u16;
            rows = rows.push(memory_row(
                cpu,
                address,
                selected == Some(address),
                &self.memory_inline_value_input,
            ));
        }

        if render_end < MEMORY_ADDRESS_COUNT {
            rows = rows.push(memory_spacer(MEMORY_ADDRESS_COUNT - render_end));
        }

        let memory_scroll_reveal = self.memory_scroll_visible_ticks > 0;
        let scrollable_memory: Element<'_, Message> = scrollable(rows)
            .id(MEMORY_SCROLL_ID)
            .height(Length::Fill)
            .style(move |theme, status| scrollable_style(memory_scroll_reveal, theme, status))
            .on_scroll(|viewport| {
                Message::MemoryScrolled(viewport.absolute_offset().y, viewport.bounds().height)
            })
            .into();

        let memory_body: Element<'_, Message> = if let Some(address) = self.opcode_dropdown_address
        {
            let top = ((address as f32 * MEMORY_ROW_HEIGHT) - self.memory_scroll_offset).max(0.0);

            stack(vec![
                scrollable_memory,
                opcode_dropdown_overlay(
                    address,
                    &self.opcode_search_input,
                    self.opcode_scroll_visible_ticks > 0,
                    top,
                ),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            scrollable_memory
        };

        let body = column![memory_header(), memory_body]
            .spacing(8)
            .height(Length::Fill);

        legend_panel("Содержимое ячеек ОЗУ", body, Length::Fill)
    }

    fn memory_editor_panel(&self) -> Element<'_, Message> {
        let controls = row![
            spinner_text_input(
                "0000",
                &self.memory_address_input,
                Message::MemoryAddressChanged,
                Message::MemoryAddressNext,
                Message::MemoryAddressPrevious,
                Length::Fixed(96.0),
                Message::JumpMemoryAddress,
                MEMORY_ADDRESS_INPUT_ID,
                self.focused_input == Some(MEMORY_ADDRESS_INPUT_ID),
                self.hovered_input == Some(MEMORY_ADDRESS_INPUT_ID),
            ),
            text_input("00", &self.memory_value_input)
                .id(MEMORY_VALUE_INPUT_ID)
                .on_input(Message::MemoryValueChanged)
                .on_submit(Message::ApplyMemory)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(58.0))
                .style(input_style),
            enter_button(Message::ApplyMemory),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(controls)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel("Ячейка ОЗУ и ее значение", content, Length::Shrink)
    }

    fn register_editor_panel(&self) -> Element<'_, Message> {
        let editor = row![
            spinner_text_input(
                "A",
                &self.register_name_input,
                Message::RegisterNameChanged,
                Message::RegisterNext,
                Message::RegisterPrevious,
                Length::Fixed(62.0),
                Message::ApplyRegister,
                REGISTER_NAME_INPUT_ID,
                self.focused_input == Some(REGISTER_NAME_INPUT_ID),
                self.hovered_input == Some(REGISTER_NAME_INPUT_ID),
            ),
            text_input("00", &self.register_value_input)
                .id(REGISTER_VALUE_INPUT_ID)
                .on_input(Message::RegisterValueChanged)
                .on_submit(Message::ApplyRegister)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(58.0))
                .style(input_style),
            enter_button(Message::ApplyRegister),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(editor)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel("Регистр и его значение", content, Length::Shrink)
    }

    fn status_bar(&self) -> Element<'_, Message> {
        container(
            row![
                ui_text("Статус", 13, TOKYO_MUTED),
                mono_text(&self.status, 15, TOKYO_TEXT).width(Length::Fill),
            ]
            .spacing(12)
            .align_y(alignment::Vertical::Center),
        )
        .padding(12)
        .width(Length::Fill)
        .style(status_bar_style)
        .into()
    }
}

fn legend_panel<'a>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
    height: Length,
) -> Element<'a, Message> {
    const LEGEND_LINE_OFFSET: f32 = 9.0;

    let panel: Element<'a, Message> = container(content)
        .padding(Padding {
            top: 18.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        })
        .width(Length::Fill)
        .height(height)
        .style(panel_style)
        .into();
    let framed_panel: Element<'a, Message> = column![
        Space::new().height(Length::Fixed(LEGEND_LINE_OFFSET)),
        panel,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(height)
    .into();
    let legend: Element<'a, Message> = row![
        Space::new().width(Length::Fill),
        container(ui_text(title, 14, TOKYO_TEXT))
            .padding([0, 5])
            .style(legend_label_style),
        Space::new().width(Length::Fill),
    ]
    .width(Length::Fill)
    .into();

    stack(vec![framed_panel, legend])
        .width(Length::Fill)
        .height(height)
        .into()
}

fn memory_header() -> Element<'static, Message> {
    container(
        row![
            ui_text("Адрес", 12, TOKYO_MUTED)
                .width(Length::Fixed(76.0))
                .align_x(alignment::Horizontal::Center),
            ui_text("Значение", 12, TOKYO_MUTED)
                .width(Length::Fixed(94.0))
                .align_x(alignment::Horizontal::Center),
            ui_text("Команда", 12, TOKYO_MUTED)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        ]
        .spacing(6),
    )
    .padding(5)
    .width(Length::Fill)
    .style(transparent_style)
    .into()
}

fn memory_spacer(rows: usize) -> Element<'static, Message> {
    Space::new()
        .width(Length::Fill)
        .height(Length::Fixed(rows as f32 * MEMORY_ROW_HEIGHT))
        .into()
}

fn memory_row<'a>(
    cpu: &Cpu8080State,
    address: u16,
    selected: bool,
    inline_value_input: &'a str,
) -> Element<'a, Message> {
    let value = cpu.memory.read(address);
    let command = decode_opcode(value)
        .map(|instruction| instruction.mnemonic)
        .unwrap_or_else(|_| "???".to_owned());
    let accent = if selected { TOKYO_BLUE } else { TOKYO_MUTED };

    let line: Element<'a, Message> = container(
        row![
            cell_button(
                mono_text(format!("{address:04X}"), 14, accent),
                Length::Fixed(76.0),
                Message::MemorySelected(address),
            ),
            memory_value_cell(value, address, selected, inline_value_input),
            command_cell_button(command, address),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center),
    )
    .padding(4)
    .height(Length::Fixed(MEMORY_ROW_HEIGHT - 1.0))
    .width(Length::Fill)
    .style(move |_theme| memory_row_container_style(selected))
    .into();

    column![line, row_separator()].spacing(0).into()
}

fn cell_button(
    content: Text<'static>,
    width: Length,
    message: Message,
) -> Element<'static, Message> {
    button(content.width(width).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(0)
        .width(width)
        .style(move |_theme, status| cell_button_style(status))
        .into()
}

fn memory_value_cell<'a>(
    value: u8,
    address: u16,
    selected: bool,
    inline_value_input: &'a str,
) -> Element<'a, Message> {
    if selected {
        text_input("00", inline_value_input)
            .id(MEMORY_INLINE_INPUT_ID)
            .on_input(move |value| Message::InlineMemoryValueChanged(address, value))
            .on_submit(Message::ApplyInlineMemoryValue(address))
            .font(MONO_FONT)
            .size(14)
            .padding(0)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fixed(94.0))
            .style(inline_value_input_style)
            .into()
    } else {
        value_cell_button(value, address)
    }
}

fn value_cell_button(value: u8, address: u16) -> Element<'static, Message> {
    button(
        mono_text(format!("{value:02X}"), 14, TOKYO_GREEN)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::MemorySelected(address))
    .padding(0)
    .width(Length::Fixed(94.0))
    .style(move |_theme, status| value_button_style(status))
    .into()
}

fn command_cell_button(command: String, address: u16) -> Element<'static, Message> {
    button(
        mono_text(command, 14, TOKYO_TEXT)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::OpcodeDropdownToggled(address))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| cell_button_style(status))
    .into()
}

fn row_separator() -> Element<'static, Message> {
    container(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_theme| solid_style(Color::from_rgba8(0x41, 0x48, 0x68, 0.26), 0.0))
        .into()
}

fn opcode_dropdown_overlay<'a>(
    address: u16,
    search: &'a str,
    reveal: bool,
    top: f32,
) -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            opaque(opcode_dropdown(address, search, reveal)),
            Space::new().width(Length::Fixed(24.0)),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn opcode_dropdown<'a>(address: u16, search: &'a str, reveal: bool) -> Element<'a, Message> {
    let mut options = Column::new().spacing(0);

    for choice in filtered_opcode_choices(search) {
        options = options.push(opcode_option(address, choice));
    }

    let content = column![
        text_input("Поиск: hex или мнемоника", search)
            .on_input(Message::OpcodeSearchChanged)
            .font(MONO_FONT)
            .size(13)
            .padding(6)
            .width(Length::Fill)
            .style(input_borderless_style),
        row_separator(),
        scrollable(options)
            .height(Length::Fixed(172.0))
            .style(move |theme, status| scrollable_style(reveal, theme, status))
            .on_scroll(|_| Message::OpcodeScrolled),
    ]
    .spacing(4);

    container(content)
        .padding(6)
        .width(Length::Fixed(226.0))
        .style(opcode_dropdown_style)
        .into()
}

fn opcode_option(address: u16, choice: OpcodeChoice) -> Element<'static, Message> {
    button(
        row![
            mono_text(format!("{:02X}", choice.value), 13, TOKYO_GREEN).width(Length::Fixed(34.0)),
            mono_text(choice.mnemonic, 13, TOKYO_TEXT).width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::OpcodeSelected(address, choice.value))
    .padding(5)
    .width(Length::Fill)
    .style(move |_theme, status| opcode_option_style(status))
    .into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OpcodeChoice {
    value: u8,
    mnemonic: String,
}

impl OpcodeChoice {
    fn new(value: u8) -> Self {
        let mnemonic = decode_opcode(value)
            .map(|instruction| instruction.mnemonic)
            .unwrap_or_else(|_| "UNDOC".to_owned());

        Self { value, mnemonic }
    }
}

fn filtered_opcode_choices(search: &str) -> Vec<OpcodeChoice> {
    let search = search.trim().to_ascii_uppercase();

    (0..=u8::MAX)
        .map(OpcodeChoice::new)
        .filter(|choice| {
            search.is_empty()
                || format!("{:02X} {}", choice.value, choice.mnemonic)
                    .to_ascii_uppercase()
                    .contains(&search)
        })
        .collect()
}

fn menu_label(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 13, TOKYO_TEXT).into()
}

fn menu_action(label: &'static str, message: Message) -> Element<'static, Message> {
    button(ui_text(label, 12, TOKYO_MUTED))
        .on_press(message)
        .padding(4)
        .style(move |_theme, status| menu_button_style(status))
        .into()
}

fn schematic_readout(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 12, TOKYO_MUTED),
            mono_text(value, 20, accent),
        ]
        .spacing(4)
        .align_x(alignment::Horizontal::Center),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(70.0))
    .style(schematic_block_style)
    .into()
}

fn compact_value(
    label: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
) -> Element<'static, Message> {
    container(
        column![
            ui_text(label, 11, TOKYO_MUTED),
            mono_text(value, 18, accent),
        ]
        .spacing(2),
    )
    .padding(8)
    .width(Length::Fill)
    .style(inset_style)
    .into()
}

fn flag_strip(cpu: &Cpu8080State) -> Element<'static, Message> {
    let dots = [
        ("Z", cpu.flags.zero),
        ("S", cpu.flags.sign),
        ("P", cpu.flags.parity),
        ("C", cpu.flags.carry),
        ("AC", cpu.flags.auxiliary_carry),
    ];

    Row::with_children(
        dots.into_iter()
            .map(|(label, active)| flag_dot(label, active)),
    )
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn flag_dot(label: &'static str, active: bool) -> Element<'static, Message> {
    column![
        mono_text("●", 18, if active { TOKYO_RED } else { TOKYO_TEXT })
            .align_x(alignment::Horizontal::Center),
        ui_text(label, 10, TOKYO_TEXT).align_x(alignment::Horizontal::Center),
    ]
    .spacing(2)
    .width(Length::Fixed(28.0))
    .into()
}

fn mux_panel(cpu: &Cpu8080State, selected: RegisterName) -> Element<'static, Message> {
    let pair = |a: RegisterName, b: RegisterName| {
        row![
            mux_register(a, cpu.registers.get(a), selected),
            mux_register(b, cpu.registers.get(b), selected),
        ]
        .spacing(0)
    };

    container(
        column![
            container(ui_text("Multiplexer", 12, TOKYO_TEXT))
                .padding(7)
                .width(Length::Fill)
                .style(mux_header_style),
            pair(RegisterName::A, RegisterName::B),
            pair(RegisterName::C, RegisterName::D),
            pair(RegisterName::E, RegisterName::H),
            row![
                mux_register(RegisterName::L, cpu.registers.l, selected),
                compact_value("SP", format!("{:04X}", cpu.sp), TOKYO_GREEN),
            ]
            .spacing(0),
            compact_value("PC", format!("{:04X}", cpu.pc), TOKYO_GREEN),
        ]
        .spacing(0),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .style(mux_panel_style)
    .into()
}

fn mux_register(
    register: RegisterName,
    value: u8,
    selected: RegisterName,
) -> Element<'static, Message> {
    let is_selected = register == selected;
    let accent = if is_selected {
        TOKYO_MAGENTA
    } else {
        TOKYO_BLUE
    };

    button(column![
        ui_text(register_name(register), 11, TOKYO_BLUE),
        mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
    ])
    .on_press(Message::RegisterSelected(register))
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(58.0))
    .style(move |_theme, status| mux_button_style(status, accent, is_selected))
    .into()
}

fn cycle_tick_panel(cpu: &Cpu8080State) -> Element<'static, Message> {
    container(
        column![
            row![
                ui_text("Cycle", 12, TOKYO_MUTED),
                mono_text(cpu.cycle_count.to_string(), 14, TOKYO_GREEN),
            ]
            .spacing(10),
            row![
                ui_text("Tick", 12, TOKYO_MUTED),
                mono_text(
                    cpu.tact_phase
                        .map(|phase| phase.to_string())
                        .unwrap_or_else(|| "1".to_owned()),
                    14,
                    TOKYO_GREEN,
                ),
            ]
            .spacing(18),
        ]
        .spacing(6),
    )
    .padding(10)
    .style(schematic_block_style)
    .into()
}

fn square_device(label: &'static str, value: &str) -> Element<'static, Message> {
    container(
        column![
            mono_text(label, 12, TOKYO_TEXT),
            ui_text(value.to_owned(), 10, TOKYO_MUTED),
        ]
        .align_x(alignment::Horizontal::Center)
        .spacing(2),
    )
    .padding(7)
    .width(Length::Fixed(52.0))
    .height(Length::Fixed(44.0))
    .style(schematic_block_style)
    .into()
}

fn control_lamps() -> Element<'static, Message> {
    const LAMPS: [&str; 11] = [
        "F2", "F1", "SYNC", "READY", "WAIT", "HOLD", "INT", "INTE", "DBIN", "WR", "HLDA",
    ];

    Row::with_children(LAMPS.into_iter().map(control_lamp))
        .spacing(7)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn control_lamp(label: &'static str) -> Element<'static, Message> {
    column![
        ui_text(label, 8, TOKYO_MUTED).align_x(alignment::Horizontal::Center),
        mono_text("●", 14, TOKYO_RED).align_x(alignment::Horizontal::Center),
    ]
    .width(Length::Fixed(34.0))
    .spacing(2)
    .into()
}

fn functional_block(
    title: impl Into<String>,
    value: impl Into<String>,
    accent: Color,
    message: Message,
) -> Element<'static, Message> {
    button(
        column![
            ui_text(title, 12, TOKYO_MUTED),
            mono_text(value, 24, accent),
        ]
        .align_x(alignment::Horizontal::Center)
        .spacing(4),
    )
    .on_press(message)
    .padding(10)
    .width(Length::Fill)
    .style(move |_theme, status| capsule_button_style(status, accent, false))
    .into()
}

fn bus_bar(label: impl Into<String>, accent: Color) -> Element<'static, Message> {
    row![
        ui_text(label, 12, TOKYO_MUTED),
        container(Space::new())
            .height(Length::Fixed(4.0))
            .width(Length::Fill)
            .style(move |_theme| solid_style(accent, 0.0)),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn spinner_text_input<'a>(
    placeholder: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    up: Message,
    down: Message,
    width: Length,
    on_submit: Message,
    id: &'static str,
    focused: bool,
    hovered: bool,
) -> Element<'a, Message> {
    let shell: Element<'a, Message> = container(
        row![
            text_input(placeholder, value)
                .id(id)
                .on_input(on_input)
                .on_submit(on_submit)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(input_borderless_style),
            column![step_button("▲", up), step_button("▼", down),].spacing(0),
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center),
    )
    .width(width)
    .style(move |theme| input_shell_style(theme, focused, hovered))
    .into();

    mouse_area(shell)
        .on_enter(Message::SpinnerHovered { id, hovered: true })
        .on_exit(Message::SpinnerHovered { id, hovered: false })
        .into()
}

fn step_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(mono_text(label, 9, TOKYO_TEXT).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(1)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(17.0))
        .style(move |_theme, status| step_button_style(status))
        .into()
}

fn enter_button(message: Message) -> Element<'static, Message> {
    icon_button("↵", message, TOKYO_GREEN)
}

fn icon_button(label: &'static str, message: Message, accent: Color) -> Element<'static, Message> {
    button(mono_text(label, 14, accent).align_x(alignment::Horizontal::Center))
        .on_press(message)
        .padding(6)
        .style(move |_theme, status| capsule_button_style(status, accent, false))
        .into()
}

fn ui_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(UI_FONT).size(size).color(color)
}

fn mono_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(MONO_FONT).size(size).color(color)
}

fn parse_hex_u16_preview(input: &str) -> Option<u16> {
    u16::from_str_radix(
        input
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X"),
        16,
    )
    .ok()
}

fn app_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BG), 0.0, 0.0, Color::TRANSPARENT)
}

fn menu_bar_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_PANEL), 6.0, 1.0, TOKYO_BORDER)
}

fn board_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BOARD), 8.0, 1.0, TOKYO_BORDER)
}

fn panel_style(theme: &Theme) -> container::Style {
    board_style(theme)
}

fn inset_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 6.0, 1.0, TOKYO_BORDER)
}

fn schematic_block_style(_theme: &Theme) -> container::Style {
    surface_style(
        Some(Color::from_rgba8(0x24, 0x26, 0x3A, 0.92)),
        6.0,
        1.0,
        TOKYO_BORDER,
    )
}

fn alu_style(_theme: &Theme) -> container::Style {
    surface_style(
        Some(Color::from_rgb8(0x25, 0x27, 0x3D)),
        6.0,
        1.5,
        TOKYO_MAGENTA,
    )
}

fn mux_header_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE_2), 0.0, 0.0, TOKYO_BORDER)
}

fn mux_panel_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 6.0, 1.0, TOKYO_BORDER)
}

fn legend_label_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_BG), 0.0, 0.0, Color::TRANSPARENT)
}

fn transparent_style(_theme: &Theme) -> container::Style {
    surface_style(None, 0.0, 0.0, Color::TRANSPARENT)
}

fn status_bar_style(_theme: &Theme) -> container::Style {
    surface_style(Some(TOKYO_SURFACE), 7.0, 1.0, TOKYO_BORDER)
}

fn input_shell_style(_theme: &Theme, focused: bool, hovered: bool) -> container::Style {
    // Mirror the right-hand text input borders so the spinner blends in.
    // Focused beats hovered because once a user has tabbed/clicked into the
    // shell, the focus ring should win even if the cursor is still over it.
    let border_color = if focused {
        TOKYO_BLUE
    } else if hovered {
        TOKYO_CYAN
    } else {
        TOKYO_BORDER
    };

    surface_style(Some(TOKYO_BG), 6.0, 1.0, border_color)
}

fn opcode_dropdown_style(_theme: &Theme) -> container::Style {
    surface_style(
        Some(Color::from_rgb8(0x19, 0x1B, 0x2A)),
        7.0,
        1.0,
        TOKYO_BORDER,
    )
}

fn solid_style(color: Color, radius: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

fn surface_style(
    background: Option<Color>,
    radius: f32,
    border_width: f32,
    border_color: Color,
) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: background.map(Background::Color),
        border: Border {
            radius: radius.into(),
            width: border_width,
            color: border_color,
        },
        ..container::Style::default()
    }
}

fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => TOKYO_BLUE,
        text_input::Status::Hovered => TOKYO_CYAN,
        text_input::Status::Active | text_input::Status::Disabled => TOKYO_BORDER,
    };

    text_input::Style {
        background: Background::Color(TOKYO_BG),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_TEXT,
        selection: TOKYO_MAGENTA,
    }
}

fn input_borderless_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_TEXT,
        selection: TOKYO_MAGENTA,
    }
}

fn inline_value_input_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: TOKYO_MUTED,
        placeholder: TOKYO_MUTED,
        value: TOKYO_GREEN,
        selection: TOKYO_MAGENTA,
    }
}

fn capsule_button_style(status: button::Status, accent: Color, selected: bool) -> button::Style {
    let active = is_button_active(status);
    let background = if selected {
        Color::from_rgba8(0xBB, 0x9A, 0xF7, 0.28)
    } else if active {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };
    let border_color = if active || selected {
        accent
    } else {
        TOKYO_BORDER
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: if selected { 1.5 } else { 1.0 },
            color: border_color,
        },
        ..button::Style::default()
    }
}

fn is_button_active(status: button::Status) -> bool {
    matches!(status, button::Status::Hovered | button::Status::Pressed)
}

fn flat_button_style(text_color: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color,
        border: Border::default(),
        ..button::Style::default()
    }
}

fn menu_button_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_2
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn step_button_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 3.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}

fn mux_button_style(status: button::Status, accent: Color, selected: bool) -> button::Style {
    let active = is_button_active(status);
    let background = if selected {
        Color::from_rgba8(0xBB, 0x9A, 0xF7, 0.45)
    } else if active {
        TOKYO_SURFACE_3
    } else {
        TOKYO_SURFACE
    };
    let border_color = if selected || active {
        accent
    } else {
        TOKYO_BORDER
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

fn cell_button_style(_status: button::Status) -> button::Style {
    flat_button_style(TOKYO_TEXT)
}

fn value_button_style(_status: button::Status) -> button::Style {
    flat_button_style(TOKYO_GREEN)
}

fn opcode_option_style(status: button::Status) -> button::Style {
    let background = if is_button_active(status) {
        TOKYO_SURFACE_3
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}

fn memory_row_container_style(selected: bool) -> container::Style {
    let background = if selected {
        Some(Background::Color(Color::from_rgba8(0x7A, 0xA2, 0xF7, 0.18)))
    } else {
        None
    };

    container::Style {
        background,
        text_color: Some(TOKYO_TEXT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

fn scrollable_style(reveal: bool, theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    const SCROLLER_HOVER: Color = Color::from_rgb(
        0x9A as f32 / 255.0,
        0xA5 as f32 / 255.0,
        0xCE as f32 / 255.0,
    );
    const SCROLLER_DRAG: Color = Color::from_rgb(
        0xC0 as f32 / 255.0,
        0xCA as f32 / 255.0,
        0xF5 as f32 / 255.0,
    );

    let mut style = scrollable::default(theme, status);
    style.vertical_rail.background = None;
    style.vertical_rail.border = Border::default();
    style.horizontal_rail.background = None;
    style.horizontal_rail.border = Border::default();

    let interacting = matches!(
        status,
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: true,
            ..
        } | scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | scrollable::Status::Dragged { .. },
    );

    let scroller_override = match status {
        scrollable::Status::Dragged { .. } => Some(SCROLLER_DRAG),
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: true,
            ..
        }
        | scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } => Some(SCROLLER_HOVER),
        _ => None,
    };

    if let Some(color) = scroller_override {
        style.vertical_rail.scroller.background = Background::Color(color);
        style.horizontal_rail.scroller.background = Background::Color(color);
    }

    if !reveal && !interacting {
        style.vertical_rail.scroller.background = Background::Color(Color::TRANSPARENT);
        style.horizontal_rail.scroller.background = Background::Color(Color::TRANSPARENT);
    }

    style
}
