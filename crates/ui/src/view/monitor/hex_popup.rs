use crate::backend::MonitorState;
use iced::widget::{
    Space, column, container, mouse_area, opaque, responsive, row, scrollable, stack,
};
use iced::{Element, Length};

use crate::app::{HexStreamFilter, MONITOR_HEX_SCROLL_ID, Message};
use crate::i18n::{Key, Lang};
use crate::view::icons;
use crate::view::theme::{mono_text, tokyo_muted, tokyo_text, ui_text};
use crate::view::widgets::compact_scrollbar;

use super::styles::{HEX_GROUP, dialog_style, framebuffer_style, popup_backdrop_style};
use super::{HexPopupViewState, icon_button};

const HEX_TEXT_SIZE: u32 = 12;
const HEX_ROW_HEIGHT: f32 = HEX_TEXT_SIZE as f32 * 1.3;
const HEX_ROW_SPACING: f32 = 2.0;
const HEX_CONTENT_PADDING: f32 = 12.0;

pub(super) fn hex_popup_overlay<'a>(
    state: &'a MonitorState,
    hex: HexPopupViewState,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(popup_backdrop_style),
    )
    .on_press(Message::ToggleMonitorHexPopup);

    let kept = filtered_hex_bytes(&state.hex_buffer, hex.filter);

    let byte_count = kept.len();
    let row_count = byte_count.div_ceil(HEX_GROUP);
    let content_height = row_count as f32 * HEX_ROW_HEIGHT
        + row_count.saturating_sub(1) as f32 * HEX_ROW_SPACING
        + HEX_CONTENT_PADDING * 2.0;
    let mut col = column![].spacing(HEX_ROW_SPACING);
    for (chunk_idx, chunk) in kept.chunks(HEX_GROUP).enumerate() {
        let offset = chunk_idx * HEX_GROUP;
        let hex: String = chunk
            .iter()
            .map(|(_, b)| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        col = col.push(
            row![
                mono_text(format!("{offset:04X}"), HEX_TEXT_SIZE, tokyo_muted()),
                Space::new().width(Length::Fixed(12.0)),
                mono_text(hex, HEX_TEXT_SIZE, tokyo_text()),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .height(Length::Fixed(HEX_ROW_HEIGHT)),
        );
    }
    let stream: Element<'_, Message> = scrollable(container(col).padding(HEX_CONTENT_PADDING))
        .id(MONITOR_HEX_SCROLL_ID)
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .on_scroll(|viewport| Message::MonitorHexScrolled(viewport.absolute_offset().y))
        .into();
    let scrollbar: Element<'_, Message> = responsive(move |size| {
        compact_scrollbar(
            hex.scroll_offset,
            (content_height - size.height).max(0.0),
            hex.reveal_scrollbar,
            Message::MonitorHexScrollbarDragged,
        )
    })
    .into();
    let body: Element<'_, Message> = stack![stream, scrollbar]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    let (filter_icon, filter_hint) = match hex.filter {
        HexStreamFilter::All => (icons::binary(), Key::MonitorHexFilterAll),
        HexStreamFilter::Graphics => (icons::line_squiggle(), Key::MonitorHexFilterGraphics),
        HexStreamFilter::Text => (icons::text_cursor(), Key::MonitorHexFilterText),
    };

    let header = row![
        ui_text(lang.t(Key::MonitorHexBuffer), 14, tokyo_text()),
        Space::new().width(Length::Fixed(16.0)),
        ui_text(format!("{byte_count} B"), 12, tokyo_muted()),
        Space::new().width(Length::Fill),
        icon_button(
            filter_icon,
            Message::CycleMonitorHexFilter,
            lang.t(filter_hint),
            None,
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Message::ToggleMonitorHexPopup,
            lang.t(Key::MonitorClose),
            Some("Esc".to_owned()),
            false,
        ),
    ]
    .align_y(iced::alignment::Vertical::Center);

    let panel = container(
        column![
            header,
            Space::new().height(Length::Fixed(8.0)),
            container(body)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(framebuffer_style),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(430.0))
    .height(Length::Fixed(480.0))
    .style(dialog_style);

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(panel),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Mirrors `MonitorDevice::output_byte` to classify each recorded byte
/// as part of a graphics or text command; if that protocol changes,
/// this must follow.
fn filtered_hex_bytes(buffer: &[u8], filter: HexStreamFilter) -> Vec<(usize, u8)> {
    if matches!(filter, HexStreamFilter::All) {
        return buffer.iter().copied().enumerate().collect();
    }

    enum Phase {
        Idle,
        Text { left: u8 },
        Graphics { left: u8 },
    }

    let mut phase = Phase::Idle;
    let mut out = Vec::with_capacity(buffer.len());

    for (idx, &byte) in buffer.iter().enumerate() {
        let is_graphics = match phase {
            Phase::Idle => {
                if byte & 0x80 == 0 {
                    phase = Phase::Text { left: 1 };
                    false
                } else {
                    phase = Phase::Graphics { left: 2 };
                    true
                }
            }
            Phase::Text { left } => {
                phase = if left <= 1 {
                    Phase::Idle
                } else {
                    Phase::Text { left: left - 1 }
                };
                false
            }
            Phase::Graphics { left } => {
                phase = if left <= 1 {
                    Phase::Idle
                } else {
                    Phase::Graphics { left: left - 1 }
                };
                true
            }
        };

        let keep = match filter {
            HexStreamFilter::All => true,
            HexStreamFilter::Graphics => is_graphics,
            HexStreamFilter::Text => !is_graphics,
        };
        if keep {
            out.push((idx, byte));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{HexStreamFilter, filtered_hex_bytes};

    #[test]
    fn filter_all_keeps_every_byte_with_original_offsets() {
        let buf = [0xFF_u8, 10, 20, 0x40, 0x41, 0x80, 0, 0];
        let kept = filtered_hex_bytes(&buf, HexStreamFilter::All);
        assert_eq!(
            kept,
            vec![
                (0, 0xFF),
                (1, 10),
                (2, 20),
                (3, 0x40),
                (4, 0x41),
                (5, 0x80),
                (6, 0),
                (7, 0),
            ],
        );
    }

    #[test]
    fn filter_graphics_keeps_only_graphics_command_bytes() {
        let buf = [0xFF_u8, 10, 20, 0x40, 0x41, 0x80, 0, 0];
        let kept = filtered_hex_bytes(&buf, HexStreamFilter::Graphics);
        assert_eq!(
            kept,
            vec![(0, 0xFF), (1, 10), (2, 20), (5, 0x80), (6, 0), (7, 0),],
        );
    }

    #[test]
    fn filter_text_keeps_only_text_command_bytes() {
        let buf = [0xFF_u8, 10, 20, 0x40, 0x41, 0x80, 0, 0];
        let kept = filtered_hex_bytes(&buf, HexStreamFilter::Text);
        assert_eq!(kept, vec![(3, 0x40), (4, 0x41)]);
    }

    #[test]
    fn filter_handles_partial_command_at_end_of_stream() {
        let buf = [0x40_u8, 0x41, 0xFF, 7];
        let gfx = filtered_hex_bytes(&buf, HexStreamFilter::Graphics);
        let txt = filtered_hex_bytes(&buf, HexStreamFilter::Text);
        assert_eq!(gfx, vec![(2, 0xFF), (3, 7)]);
        assert_eq!(txt, vec![(0, 0x40), (1, 0x41)]);
    }
}
