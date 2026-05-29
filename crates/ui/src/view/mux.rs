//! Multiplexer panel — the right column of the schematic plate.
//!
//! Carries the W/Z scratch pair, the general-purpose register grid
//! (B/C, D/E, H/L), and the SP/PC footer. Split out of `schematic.rs`
//! to keep that file under the 400-line workspace ceiling.

use iced::widget::{Space, column, container, mouse_area, row, text_input};
use iced::{Background, Color, Element, Length, Padding, Theme, alignment};
use k580_core::{Cpu8080State, RegisterName, decode_opcode};

use super::styles::{
    inline_value_input_style, mux_chip_style, mux_header_style, mux_panel_style, solid_style,
};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MUTED, TOKYO_SELECTION_BLUE, TOKYO_SURFACE,
    TOKYO_TEXT, mono_text, ui_text,
};
use super::utils::row_separator;
use crate::app::{Message, REGISTER_INLINE_INPUT_ID, RegisterInlineTarget, register_name};

const MUX_REGISTER_CELL_HEIGHT: f32 = 30.0;
const MUX_REGISTER_VALUE_WIDTH: f32 = 28.0;

#[derive(Clone, Copy)]
struct MuxEditState<'a> {
    selected: RegisterName,
    inline_target: Option<RegisterInlineTarget>,
    active_target: Option<RegisterInlineTarget>,
    hovered_target: Option<RegisterInlineTarget>,
    input_value: &'a str,
}

pub(super) struct MuxRegisterValues {
    pub(super) b: String,
    pub(super) c: String,
    pub(super) d: String,
    pub(super) e: String,
    pub(super) h: String,
    pub(super) l: String,
}

/// Three framed subgroups: W/Z scratch registers, the two-column
/// B/C-D/E-H/L general-purpose grid, and the stack/program-counter
/// footer.
pub(super) fn mux_panel<'a>(
    cpu: &Cpu8080State,
    selected: RegisterName,
    inline_target: Option<RegisterInlineTarget>,
    active_target: Option<RegisterInlineTarget>,
    hovered_target: Option<RegisterInlineTarget>,
    input_value: &'a str,
    values: MuxRegisterValues,
) -> Element<'a, Message> {
    let edit_state = MuxEditState {
        selected,
        inline_target,
        active_target,
        hovered_target,
        input_value,
    };

    let scratch_group = container(
        column![
            mux_section_caption("Регистры временного хранения"),
            mux_static_pair("W", cpu.registers.w, "Z", cpu.registers.z),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(mux_chip_style);

    let general_group = container(
        column![
            mux_section_caption("Регистры общего назначения (РОН)"),
            mux_register_pair(
                RegisterName::B,
                values.b,
                RegisterName::C,
                values.c,
                edit_state,
            ),
            row_separator(),
            mux_register_pair(
                RegisterName::D,
                values.d,
                RegisterName::E,
                values.e,
                edit_state,
            ),
            row_separator(),
            mux_register_pair(
                RegisterName::H,
                values.h,
                RegisterName::L,
                values.l,
                edit_state,
            ),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(mux_chip_style);

    let pointer_group = container(
        column![
            mux_readout_row("Указатель стека (УС)", format!("{:04X}", cpu.sp)),
            row_separator(),
            mux_readout_row("Счётчик команд (СК)", format!("{:04X}", cpu.pc)),
            row_separator(),
            mux_readout_row(
                "Инкремент-декремент",
                format!(
                    "+{}",
                    decode_opcode(cpu.memory.read(cpu.pc))
                        .map(|info| info.size)
                        .unwrap_or(1)
                ),
            ),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(mux_chip_style);

    let table = column![scratch_group, general_group, pointer_group].spacing(6);

    container(
        column![
            container(
                ui_text("Мультиплексор", 14, TOKYO_MUTED).align_x(alignment::Horizontal::Center),
            )
            .height(Length::Fixed(18.0))
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
            table,
        ]
        .spacing(2),
    )
    .padding(Padding {
        top: 4.0,
        right: 8.0,
        bottom: 6.0,
        left: 8.0,
    })
    .width(Length::FillPortion(1))
    .style(mux_panel_style)
    .into()
}

/// Centred muted-text divider that splits the chip group into
/// scratch / general-purpose / footer subblocks. `align_x(Center)`
/// is applied to both the inner `Text` and the container so the
/// caption stays centred regardless of how iced rounds the inner
/// text bounding box against the outer width.
fn mux_section_caption(label: &'static str) -> Element<'static, Message> {
    container(ui_text(label, 11, TOKYO_MUTED).align_x(alignment::Horizontal::Center))
        .padding([3, 8])
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .style(mux_header_style)
        .into()
}

fn mux_static_pair(
    left_label: &'static str,
    left_value: u8,
    right_label: &'static str,
    right_value: u8,
) -> Element<'static, Message> {
    row![
        mux_static_cell(left_label, left_value),
        mux_column_separator(),
        mux_static_cell(right_label, right_value),
    ]
    .spacing(0)
    .height(Length::Fixed(MUX_REGISTER_CELL_HEIGHT))
    .into()
}

fn mux_static_cell(label: &'static str, value: u8) -> Element<'static, Message> {
    container(
        row![
            ui_text(label, 13, TOKYO_MUTED),
            Space::new().width(Length::Fill),
            mono_text(format!("{value:02X}"), 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .height(Length::Fixed(30.0))
    .into()
}

/// Single row inside the SP / PC footer group. The group owns the
/// frame; rows stay borderless and are split by 1-px separators so the
/// footer reads as one subblock instead of three rounded chips.
fn mux_readout_row(label: &'static str, value: String) -> Element<'static, Message> {
    container(
        row![
            ui_text(label, 12, TOKYO_MUTED),
            Space::new().width(Length::Fill),
            mono_text(value, 16, TOKYO_GREEN),
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .into()
}

fn mux_register_pair(
    left: RegisterName,
    left_value: String,
    right: RegisterName,
    right_value: String,
    edit_state: MuxEditState<'_>,
) -> Element<'_, Message> {
    row![
        mux_register_cell(left, left_value, edit_state),
        mux_column_separator(),
        mux_register_cell(right, right_value, edit_state),
    ]
    .spacing(0)
    .height(Length::Fixed(30.0))
    .into()
}

fn mux_register_cell(
    register: RegisterName,
    value: String,
    edit_state: MuxEditState<'_>,
) -> Element<'_, Message> {
    let target = RegisterInlineTarget::Mux(register);
    let is_selected = if edit_state.active_target.is_some() {
        edit_state.active_target == Some(target)
    } else {
        register == edit_state.selected
    };
    let editing = edit_state.inline_target == Some(target);
    let hovered = edit_state.hovered_target == Some(target);

    // Selected register name uses TOKYO_BLUE, idle uses TOKYO_MUTED —
    // matches the memory-row address column. Byte stays TOKYO_GREEN.
    let label_color = if is_selected { TOKYO_BLUE } else { TOKYO_MUTED };

    let value: Element<'_, Message> = if editing {
        text_input("00", edit_state.input_value)
            .id(REGISTER_INLINE_INPUT_ID)
            .on_input(move |value| Message::InlineRegisterValueChanged(target, value))
            .on_submit(Message::ApplyInlineRegisterValue(target))
            .font(MONO_FONT)
            .size(16)
            .padding(0)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fixed(MUX_REGISTER_VALUE_WIDTH))
            .style(inline_value_input_style)
            .into()
    } else {
        mouse_area(
            container(mono_text(value, 16, TOKYO_GREEN))
                .width(Length::Fixed(MUX_REGISTER_VALUE_WIDTH))
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::RegisterEnter(target))
        .on_double_click(Message::RegisterEnter(target))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
    };

    let body = container(
        row![
            ui_text(register_name(register), 13, label_color),
            Space::new().width(Length::Fill),
            value,
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8)
        .width(Length::Fill),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .height(Length::Fixed(MUX_REGISTER_CELL_HEIGHT))
    .style(move |theme| {
        mux_register_cell_style(theme, is_selected || hovered || editing, is_selected)
    });

    let area = mouse_area(body)
        .on_enter(Message::RegisterHoverStarted(target))
        .on_exit(Message::RegisterHoverEnded(target))
        .interaction(iced::mouse::Interaction::Pointer);

    if editing {
        area.on_press(Message::RegisterEnter(target))
            .on_double_click(Message::RegisterEnter(target))
            .into()
    } else {
        area.on_press(Message::RegisterSelected(target))
            .on_double_click(Message::RegisterEnter(target))
            .into()
    }
}

fn mux_register_cell_style(_theme: &Theme, active: bool, selected: bool) -> container::Style {
    let background = if selected {
        Some(TOKYO_SELECTION_BLUE)
    } else if active {
        Some(TOKYO_SURFACE)
    } else {
        None
    };

    container::Style {
        background: background.map(Background::Color),
        text_color: Some(TOKYO_TEXT),
        border: iced::Border {
            radius: 0.0.into(),
            width: if active { 1.0 } else { 0.0 },
            color: if active {
                mux_grid_line_color()
            } else {
                Color::TRANSPARENT
            },
        },
        ..container::Style::default()
    }
}

fn mux_column_separator() -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(MUX_REGISTER_CELL_HEIGHT))
        .style(|_theme| solid_style(mux_grid_line_color(), 0.0))
        .into()
}

fn mux_grid_line_color() -> Color {
    Color::from_rgba8(0x41, 0x48, 0x68, 0.26)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_cell_hover_restores_grid_line_without_accent_border() {
        let style = mux_register_cell_style(&Theme::TokyoNight, true, false);

        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.color, mux_grid_line_color());
    }
}
