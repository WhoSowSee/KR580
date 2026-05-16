//! Virtualised list of the full 64 KiB address space.
//!
//! Builds the scrollable rows, the column header, the inline value editor
//! that lives on the currently selected row, and the surrounding
//! `legend_panel` frame. The opcode dropdown is composed in here when the
//! user opens it.

use iced::widget::{
    Column, Space, Text, button, column, container, mouse_area, row, scrollable, stack, text_input,
};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, decode_opcode};

use super::opcode_dropdown::opcode_dropdown_overlay;
use super::styles::{
    cell_button_style, inline_value_input_style, memory_row_container_style, scrollable_style,
    transparent_style, value_button_style,
};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text,
};
use super::utils::row_separator;
use super::widgets::legend_panel;
use crate::app::{
    DesktopApp, MEMORY_ADDRESS_COUNT, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, Message,
};

impl DesktopApp {
    pub(super) fn memory_panel(&self) -> Element<'_, Message> {
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
                selected == Some(address.saturating_add(1)),
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
}

fn memory_header() -> Element<'static, Message> {
    container(
        row![
            ui_text("Адрес", 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text("Значение", 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text("Команда", 12, TOKYO_MUTED)
                .width(Length::FillPortion(1))
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
    next_selected: bool,
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
                Length::FillPortion(1),
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

    let line: Element<'a, Message> = mouse_area(line)
        .on_press(Message::MemorySelected(address))
        .into();

    // Hide the divider when this row or the row immediately below it is
    // selected, so the rounded highlight has clear margins above and
    // below instead of running into a horizontal line.
    let separator: Element<'a, Message> = if selected || next_selected {
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .into()
    } else {
        row_separator()
    };

    column![line, separator].spacing(0).into()
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
        container(
            text_input("00", inline_value_input)
                .id(MEMORY_INLINE_INPUT_ID)
                .on_input(move |value| Message::InlineMemoryValueChanged(address, value))
                .on_submit(Message::ApplyInlineMemoryValue(address))
                .font(MONO_FONT)
                .size(14)
                .padding(0)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(inline_value_input_style),
        )
        .width(Length::FillPortion(1))
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
    .width(Length::FillPortion(1))
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
    .width(Length::FillPortion(1))
    .style(move |_theme, status| cell_button_style(status))
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
