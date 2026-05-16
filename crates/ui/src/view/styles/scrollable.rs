//! Scrollable rail style. Auto-hides the scroller when the user is not
//! interacting with the rail, except for a brief reveal window driven
//! by the `reveal` flag (used to flash the scrollbar after a programmatic
//! jump).

use iced::widget::scrollable;
use iced::{Background, Border, Color, Theme};

pub(crate) fn scrollable_style(
    reveal: bool,
    theme: &Theme,
    status: scrollable::Status,
) -> scrollable::Style {
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
