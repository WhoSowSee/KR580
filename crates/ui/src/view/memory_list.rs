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
    MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text,
};
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
    inline_value_input: &'a str,
) -> Element<'a, Message> {
    let value = cpu.memory.read(address);
    // For the selected row mirror whatever byte the user is currently
    // typing into the inline editor; for any other row decode the byte
    // that is actually stored in memory. This makes the "Команда"
    // column update live as the user types instead of waiting for
    // Enter to commit the write.
    let preview_value = if selected {
        parse_hex_u8_preview(inline_value_input).unwrap_or(value)
    } else {
        value
    };
    let command = decode_opcode(preview_value)
        .map(|instruction| instruction.mnemonic)
        .unwrap_or_else(|_| "-".to_owned());
    let accent = if selected { TOKYO_BLUE } else { TOKYO_MUTED };

    // The cells fill the entire row height (including the strip where
    // the 1-pixel separator is painted). Each cell's `mouse_area` is
    // `Length::Fill`, so clicks on the bottom-edge pixel land on
    // whichever cell is directly above them — the separator itself is
    // a purely cosmetic overlay.
    let cells_row: Element<'a, Message> = container(
        row![
            address_cell(address, accent),
            memory_value_cell(value, address, selected, inline_value_input),
            command_cell(command, address),
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center),
    )
    .height(Length::Fixed(MEMORY_ROW_HEIGHT))
    .width(Length::Fill)
    .style(move |_theme| memory_row_container_style(selected))
    .into();

    // The 1-pixel divider between rows used to live in its own
    // `mouse_area` underneath the row. That worked but added a third
    // hit-tested element per row, and a separate dispatch path for
    // separator clicks. Now the cells_row owns the full
    // `MEMORY_ROW_HEIGHT`, and we just paint a 1-pixel line on top of
    // it via a non-interactive `container`. `container` does not
    // capture pointer events, so a click landing on the separator
    // pixel falls straight through to whichever cell `mouse_area` sits
    // beneath it — the cell takes the gesture and routes it through
    // its own `on_press` / `on_double_click`. When the current row or
    // the row below it is selected we hide the line entirely so the
    // rounded highlight doesn't bump into a horizontal stripe.
    let separator_overlay: Element<'a, Message> = if selected || next_selected {
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
                .style(|_theme| solid_style(
                    iced::Color::from_rgba8(0x41, 0x48, 0x68, 0.26),
                    0.0
                )),
        ]
        .into()
    };

    stack![cells_row, separator_overlay]
        .width(Length::Fill)
        .height(Length::Fixed(MEMORY_ROW_HEIGHT))
        .into()
}

/// Wraps a centred text in a `mouse_area` that fires `on_press` on a
/// single click and `on_double_click` on a double click, with a fixed
/// `FillPortion(1)` width so all three columns line up. The reason we
/// don't use `button` for the address/command cells: `button::update`
/// captures `ButtonPressed`, and `mouse_area::update` returns early on
/// `shell.is_event_captured()` — so wrapping a row of buttons in an
/// outer `mouse_area` would make the outer one blind to every click.
/// Switching to `mouse_area` per cell lets each cell route its own
/// gestures (single click, double click) independently while still
/// leaving the press uncaptured for sibling listeners.
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
        Message::MemoryEnter(address),
    ))
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .into()
}

fn command_cell(command: String, address: u16) -> Element<'static, Message> {
    container(cell_mouse_area(
        mono_text(command, 14, TOKYO_TEXT),
        Message::OpcodeDropdownToggled(address),
        Message::MemoryEnter(address),
    ))
    .width(Length::FillPortion(1))
    .height(Length::Fill)
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
    container(cell_mouse_area(
        mono_text(format!("{value:02X}"), 14, TOKYO_GREEN),
        // Single click on the value column unambiguously means "let me
        // type here", so it goes straight to `MemoryEnter` (select +
        // focus the inline editor). Double-click does the same thing,
        // which makes the gesture forgiving — clicking twice quickly
        // still ends up in editing mode instead of falling back to
        // bare selection.
        Message::MemoryEnter(address),
        Message::MemoryEnter(address),
    ))
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
