//! Virtualised list of the full 64 KiB address space.
//!
//! Builds the scrollable rows, the column header, the inline value editor
//! that lives on the currently selected row, and the surrounding
//! `legend_panel` frame. The opcode dropdown is composed in here when the
//! user opens it.

use iced::widget::{Column, Space, column, container, row, scrollable, stack};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, decode_opcode};

mod cells;
mod operands;
use cells::{address_cell, command_cell, memory_value_cell};
use operands::classify_operands;

pub(crate) use operands::{operand_jump_target, operand_port_number};

use super::opcode_dropdown::{OPCODE_DROPDOWN_HEIGHT, opcode_dropdown_overlay};
use super::styles::{memory_row_container_style, scrollable_style, solid_style, transparent_style};
use super::theme::{
    tokyo_blue, tokyo_cyan, tokyo_green, tokyo_magenta, tokyo_muted, tokyo_red, tokyo_subtle_line,
    tokyo_yellow, ui_text,
};
use super::widgets::legend_panel;
use crate::app::{
    DesktopApp, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS, MEMORY_RENDER_ROWS,
    MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, Message,
};
use crate::i18n::Key;

impl DesktopApp {
    pub(super) fn memory_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;
        let (view_start, view_count) = self.memory_view();
        let selected = parse_hex_u16_preview(&self.memory_address_input);
        let render_start =
            (self.memory_scroll_first_row as usize).saturating_sub(MEMORY_OVERSCAN_ROWS);
        let render_end = (render_start + MEMORY_RENDER_ROWS).min(view_count);
        let rendered_start = view_start.wrapping_add(render_start as u16);
        let rendered_count = render_end - render_start;
        let operand_kinds = classify_operands(rendered_start, rendered_count, &cpu.memory);
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
                MemoryRowVisuals {
                    selected: selected == Some(address),
                    next_selected: selected == Some(address.saturating_add(1)),
                    halted_here,
                    operand_highlighting: self.memory_operand_highlighting,
                    is_address_operand: operand_kinds.addresses.contains(&address),
                    is_data_operand: operand_kinds.data.contains(&address),
                    is_port_operand: operand_kinds.ports.contains(&address),
                },
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
            let row_top = (((address.saturating_sub(view_start) as f32) * MEMORY_ROW_HEIGHT)
                - self.memory_scroll_offset)
                .max(0.0);
            let top = opcode_dropdown_top(row_top, self.memory_viewport_height);

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
            ui_text(lang.t(Key::ColumnAddress).to_owned(), 12, tokyo_muted())
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text(lang.t(Key::ColumnValue).to_owned(), 12, tokyo_muted())
                .width(Length::FillPortion(1))
                .align_x(alignment::Horizontal::Center),
            ui_text(lang.t(Key::ColumnCommand).to_owned(), 12, tokyo_muted())
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

fn opcode_dropdown_top(row_top: f32, viewport_height: f32) -> f32 {
    if viewport_height <= 0.0 {
        return row_top;
    }

    let row_bottom = row_top + MEMORY_ROW_HEIGHT;
    let space_below = viewport_height - row_top;
    let max_top = (viewport_height - OPCODE_DROPDOWN_HEIGHT).max(0.0);

    if space_below < OPCODE_DROPDOWN_HEIGHT {
        (row_bottom - OPCODE_DROPDOWN_HEIGHT).clamp(0.0, max_top)
    } else {
        row_top.min(max_top)
    }
}

struct MemoryRowVisuals {
    selected: bool,
    next_selected: bool,
    halted_here: bool,
    operand_highlighting: bool,
    is_address_operand: bool,
    is_data_operand: bool,
    is_port_operand: bool,
}

fn memory_row<'a>(
    cpu: &Cpu8080State,
    address: u16,
    visuals: MemoryRowVisuals,
    inline_value_input: &'a str,
    inline_placeholder: &'a str,
) -> Element<'a, Message> {
    let value = cpu.memory.read(address);
    // Mirror the in-progress inline edit on the selected row so the
    // command column updates live; others decode the stored byte.
    let preview_value = if visuals.selected {
        parse_hex_u8_preview(inline_value_input).unwrap_or(value)
    } else {
        value
    };
    let command = decode_opcode(preview_value)
        .map(|instruction| instruction.mnemonic)
        .unwrap_or_else(|_| "-".to_owned());
    let accent = if visuals.halted_here {
        tokyo_red()
    } else if visuals.selected {
        tokyo_blue()
    } else {
        tokyo_muted()
    };
    let value_color = if !visuals.operand_highlighting {
        tokyo_green()
    } else if visuals.is_port_operand {
        tokyo_magenta()
    } else if visuals.is_address_operand {
        tokyo_yellow()
    } else if visuals.is_data_operand {
        tokyo_cyan()
    } else {
        tokyo_green()
    };

    // Cells fill the full row height; clicks on the bottom-edge
    // pixel land on the cell above (separator is purely cosmetic).
    let cells_row: Element<'a, Message> = container(
        row![
            address_cell(address, accent),
            memory_value_cell(
                value,
                address,
                visuals.selected,
                value_color,
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
    .style(move |_theme| memory_row_container_style(visuals.selected, visuals.halted_here))
    .into();

    // Cosmetic 1-px divider over the cells row. Hidden when this/next
    // row is selected or halted.
    let separator_overlay: Element<'a, Message> =
        if visuals.selected || visuals.next_selected || visuals.halted_here {
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
                    .style(|_theme| solid_style(tokyo_subtle_line(), 0.0)),
            ]
            .into()
        };

    stack![cells_row, separator_overlay]
        .width(Length::Fill)
        .height(Length::Fixed(MEMORY_ROW_HEIGHT))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_dropdown_opens_down_when_it_fits() {
        assert_eq!(opcode_dropdown_top(32.0, 320.0), 32.0);
    }

    #[test]
    fn opcode_dropdown_opens_up_when_bottom_would_clip() {
        assert_eq!(
            opcode_dropdown_top(260.0, 320.0),
            260.0 + MEMORY_ROW_HEIGHT - OPCODE_DROPDOWN_HEIGHT
        );
    }

    #[test]
    fn opcode_dropdown_top_clamps_to_viewport_top() {
        assert_eq!(opcode_dropdown_top(80.0, 140.0), 0.0);
    }

    #[test]
    fn opcode_dropdown_top_clamps_to_viewport_bottom() {
        assert_eq!(opcode_dropdown_top(500.0, 320.0), 96.0);
    }
}
