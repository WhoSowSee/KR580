use crate::backend::MonitorState;
use iced::widget::{Space, column, container, mouse_area, opaque, row, scrollable, stack};
use iced::{Element, Length};

use crate::app::{HexStreamFilter, Message};
use crate::i18n::{Key, Lang};
use crate::view::icons;
use crate::view::styles::scrollable_style;
use crate::view::theme::{TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};

use super::icon_button;
use super::styles::{HEX_GROUP, dialog_style, framebuffer_style, popup_backdrop_style};

pub(super) fn hex_popup_overlay<'a>(
    state: &'a MonitorState,
    filter: HexStreamFilter,
    reveal: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(popup_backdrop_style),
    )
    .on_press(Message::ToggleMonitorHexPopup);

    let kept = filtered_hex_bytes(&state.hex_buffer, filter);

    let mut col = column![].spacing(2);
    for (chunk_idx, chunk) in kept.chunks(HEX_GROUP).enumerate() {
        let offset = chunk_idx * HEX_GROUP;
        let hex: String = chunk
            .iter()
            .map(|(_, b)| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        col = col.push(
            row![
                mono_text(format!("{offset:04X}"), 12, TOKYO_MUTED),
                Space::new().width(Length::Fixed(12.0)),
                mono_text(hex, 12, TOKYO_TEXT),
            ]
            .align_y(iced::alignment::Vertical::Center),
        );
    }
    let body: Element<'_, Message> = scrollable(container(col).padding(12))
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(|_| Message::MonitorHexScrolled)
        .style(move |theme, status| scrollable_style(reveal, theme, status))
        .into();

    let (filter_icon, filter_hint) = match filter {
        HexStreamFilter::All => (icons::binary(), Key::MonitorHexFilterAll),
        HexStreamFilter::Graphics => (icons::line_squiggle(), Key::MonitorHexFilterGraphics),
        HexStreamFilter::Text => (icons::text_cursor(), Key::MonitorHexFilterText),
    };

    let header = row![
        ui_text(lang.t(Key::MonitorHexBuffer), 14, TOKYO_TEXT),
        Space::new().width(Length::Fixed(16.0)),
        ui_text(format!("{} B", kept.len()), 12, TOKYO_MUTED),
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
