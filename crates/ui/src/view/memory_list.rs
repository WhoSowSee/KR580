//! Virtualised list of the full 64 KiB address space.
//!
//! Builds the scrollable rows, the column header, the inline value editor
//! that lives on the currently selected row, and the surrounding
//! `legend_panel` frame. The opcode dropdown is composed in here when the
//! user opens it.

use iced::widget::{
    Column, Space, Text, column, container, mouse_area, row, scrollable, stack, text_input,
};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, decode_opcode};

use super::opcode_dropdown::opcode_dropdown_overlay;
use super::styles::{
    inline_value_input_style, memory_row_container_style, scrollable_style, solid_style,
    transparent_style,
};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MUTED, TOKYO_RED, TOKYO_TEXT, mono_text, ui_text,
};
use super::widgets::legend_panel;
use crate::app::{
    DesktopApp, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS, MEMORY_RENDER_ROWS,
    MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, Message,
};
use crate::i18n::Key;

const VALUE_GLYPH_WIDTH: f32 = 28.0;

impl DesktopApp {
    pub(super) fn memory_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;
        let (view_start, view_count) = self.memory_view();
        let selected = parse_hex_u16_preview(&self.memory_address_input);
        let render_start =
            (self.memory_scroll_first_row as usize).saturating_sub(MEMORY_OVERSCAN_ROWS);
        let render_end = (render_start + MEMORY_RENDER_ROWS).min(view_count);
        let mut rows: Column<'_, Message> = Column::new().spacing(0);
        let inline_placeholder = self.input_placeholder(MEMORY_INLINE_INPUT_ID, "00");

        if render_start > 0 {
            rows = rows.push(memory_spacer(render_start));
        }

        for row in render_start..render_end {
            let address = view_start + row as u16;
            // PC sits one byte past the HLT opcode after halt;
            // halted row = `pc == addr+1` AND byte == 0x76.
            let halted_here =
                cpu.halted && address.wrapping_add(1) == cpu.pc && cpu.memory.read(address) == 0x76;
            rows = rows.push(memory_row(
                cpu,
                address,
                selected == Some(address),
                selected == Some(address.saturating_add(1)),
                halted_here,
                &self.memory_inline_value_input,
                inline_placeholder,
            ));
        }

        if render_end < view_count {
            rows = rows.push(memory_spacer(view_count - render_end));
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
            let top = (((address.saturating_sub(view_start) as f32) * MEMORY_ROW_HEIGHT)
                - self.memory_scroll_offset)
                .max(0.0);

            stack(vec![
                scrollable_memory,
                opcode_dropdown_overlay(
                    address,
                    &self.opcode_search_input,
                    self.opcode_highlight_index,
                    self.opcode_scroll_visible_ticks > 0,
                    top,
                    self.lang,
                ),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            scrollable_memory
        };

        let body = column![memory_header(self.lang), memory_body]
            .spacing(8)
            .height(Length::Fill);

        legend_panel(self.lang.t(Key::MemoryListTitle), body, Length::Fill)
    }
}

fn memory_header(lang: crate::i18n::Lang) -> Element<'static, Message> {
    container(
        row![
            ui_text(lang.t(Key::ColumnAddress).to_owned(), 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text(lang.t(Key::ColumnValue).to_owned(), 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text(lang.t(Key::ColumnCommand).to_owned(), 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
        ]
        .spacing(0),
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
    next_selected: bool,
    halted_here: bool,
    inline_value_input: &'a str,
    inline_placeholder: &'a str,
) -> Element<'a, Message> {
    let value = cpu.memory.read(address);
    // Mirror the in-progress inline edit on the selected row so the
    // command column updates live; others decode the stored byte.
    let preview_value = if selected {
        parse_hex_u8_preview(inline_value_input).unwrap_or(value)
    } else {
        value
    };
    let command = decode_opcode(preview_value)
        .map(|instruction| instruction.mnemonic)
        .unwrap_or_else(|_| "-".to_owned());
    let accent = if halted_here {
        TOKYO_RED
    } else if selected {
        TOKYO_BLUE
    } else {
        TOKYO_MUTED
    };

    // Cells fill the full row height; clicks on the bottom-edge
    // pixel land on the cell above (separator is purely cosmetic).
    let cells_row: Element<'a, Message> = container(
        row![
            address_cell(address, accent),
            memory_value_cell(
                value,
                address,
                selected,
                inline_value_input,
                inline_placeholder,
            ),
            command_cell(command, address),
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center),
    )
    .height(Length::Fixed(MEMORY_ROW_HEIGHT))
    .width(Length::Fill)
    .style(move |_theme| memory_row_container_style(selected, halted_here))
    .into();

    // Cosmetic 1-px divider over the cells row. Hidden when this/next
    // row is selected or halted.
    let separator_overlay: Element<'a, Message> = if selected || next_selected || halted_here {
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(MEMORY_ROW_HEIGHT))
            .into()
    } else {
        column![
            Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(MEMORY_ROW_HEIGHT - 1.0)),
            container(Space::new())
                .height(Length::Fixed(1.0))
                .width(Length::Fill)
                .style(|_theme| solid_style(iced::Color::from_rgba8(0x41, 0x48, 0x68, 0.26), 0.0)),
        ]
        .into()
    };

    stack![cells_row, separator_overlay]
        .width(Length::Fill)
        .height(Length::Fixed(MEMORY_ROW_HEIGHT))
        .into()
}

/// `mouse_area`-wrapped cell that fires `on_press`/`on_double_click`
/// with `FillPortion(1)`. Not a `button` because `button::update`
/// captures `ButtonPressed` and an outer `mouse_area` would then go
/// blind to every click.
fn cell_mouse_area<'a>(
    label: Text<'a>,
    on_press: Message,
    on_double_click: Message,
) -> Element<'a, Message> {
    let body = container(
        label
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(alignment::Vertical::Center);
    mouse_area(body)
        .on_press(on_press)
        .on_double_click(on_double_click)
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

fn address_cell(address: u16, accent: iced::Color) -> Element<'static, Message> {
    container(cell_mouse_area(
        mono_text(format!("{address:04X}"), 14, accent),
        Message::MemorySelected(address),
        Message::MemoryReplace(address),
    ))
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .into()
}

/// Two stacked listeners: an inner `mouse_area` over the mnemonic
/// glyph fires `OpcodeDropdownToggled`, the outer one fires
/// `MemorySelected`. `mouse_area::update` ignores presses an inner
/// widget already captured, so each gesture reaches one listener.
fn command_cell(command: String, address: u16) -> Element<'static, Message> {
    let glyph: Element<'static, Message> = mouse_area(mono_text(command, 14, TOKYO_TEXT))
        .on_press(Message::OpcodeDropdownToggled(address))
        .on_double_click(Message::MemoryReplace(address))
        .interaction(iced::mouse::Interaction::Pointer)
        .into();

    let body = container(glyph)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    let cell = mouse_area(body)
        .on_press(Message::MemorySelected(address))
        .on_double_click(Message::MemoryReplace(address));

    container(cell)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
}

fn memory_value_cell<'a>(
    value: u8,
    address: u16,
    selected: bool,
    inline_value_input: &'a str,
    inline_placeholder: &'a str,
) -> Element<'a, Message> {
    if selected {
        let editor: Element<'a, Message> = container(
            text_input(inline_placeholder, inline_value_input)
                .id(MEMORY_INLINE_INPUT_ID)
                .on_input(move |value| Message::InlineMemoryValueChanged(address, value))
                .on_submit(Message::ApplyInlineMemoryValue(address))
                .font(MONO_FONT)
                .size(14)
                .padding(0)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(VALUE_GLYPH_WIDTH))
                .style(inline_value_input_style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into();

        let body = mouse_area(editor)
            .on_press(Message::MemorySelected(address))
            .on_double_click(Message::MemoryReplace(address))
            .interaction(iced::mouse::Interaction::Pointer);

        container(body)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    } else {
        value_cell_button(value, address)
    }
}

/// Single-click edits in place; double-click starts with an empty replacement field.
fn value_cell_button(value: u8, address: u16) -> Element<'static, Message> {
    let glyph: Element<'static, Message> =
        mouse_area(mono_text(format!("{value:02X}"), 14, TOKYO_GREEN))
            .on_press(Message::MemoryEnter(address))
            .on_double_click(Message::MemoryReplace(address))
            .interaction(iced::mouse::Interaction::Text)
            .into();

    let body = mouse_area(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::MemorySelected(address))
    .on_double_click(Message::MemoryReplace(address))
    .interaction(iced::mouse::Interaction::Pointer);

    container(body)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
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

fn parse_hex_u8_preview(input: &str) -> Option<u8> {
    u8::from_str_radix(
        input
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X"),
        16,
    )
    .ok()
}
