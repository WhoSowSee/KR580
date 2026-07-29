use iced::widget::{Space, column, container, row, stack};
use iced::{Background, Color, Element, Length, Point, Size};

use super::styles::status_tooltip_style;
use super::theme::{tokyo_text, ui_text};
use crate::app::Message;

const HINT_WIDTH: f32 = 140.0;
const HINT_HEIGHT: f32 = 24.0;
const EDGE_INSET: f32 = 8.0;
const CURSOR_OFFSET: f32 = 16.0;
const FLIPPED_GAP: f32 = 8.0;

pub(super) fn with_file_drop_hover<'a>(
    base: Element<'a, Message>,
    active: bool,
    cursor: Option<Point>,
    viewport: Size,
    hint: &'static str,
) -> Element<'a, Message> {
    if !active {
        return base;
    }
    let overlay: Element<'a, Message> = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(file_drop_hover_style)
        .into();
    let layers = stack![base, overlay]
        .width(Length::Fill)
        .height(Length::Fill);
    let Some(cursor) = cursor else {
        return layers.into();
    };
    let hint_position = file_drop_hint_position(cursor, viewport);
    let hint =
        container(ui_text(hint, 12, tokyo_text()).align_x(iced::alignment::Horizontal::Center))
            .width(Length::Fixed(HINT_WIDTH))
            .height(Length::Fixed(HINT_HEIGHT))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(status_tooltip_style);
    let hint_layer: Element<'a, Message> = column![
        Space::new().height(Length::Fixed(hint_position.y)),
        row![Space::new().width(Length::Fixed(hint_position.x)), hint].width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    layers.push(hint_layer).into()
}

fn file_drop_hint_position(cursor: Point, viewport: Size) -> Point {
    let max_x = (viewport.width - HINT_WIDTH - EDGE_INSET).max(EDGE_INSET);
    let max_y = (viewport.height - HINT_HEIGHT - EDGE_INSET).max(EDGE_INSET);
    let right = cursor.x + CURSOR_OFFSET;
    let below = cursor.y + CURSOR_OFFSET;
    let x = if right <= max_x {
        right.max(EDGE_INSET)
    } else {
        (cursor.x - HINT_WIDTH - FLIPPED_GAP).clamp(EDGE_INSET, max_x)
    };
    let y = if below <= max_y {
        below.max(EDGE_INSET)
    } else {
        (cursor.y - HINT_HEIGHT - FLIPPED_GAP).clamp(EDGE_INSET, max_y)
    };
    Point::new(x, y)
}

fn file_drop_hover_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.18,
            ..Color::BLACK
        })),
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{file_drop_hint_position, file_drop_hover_style};
    use iced::{Background, Point, Size};

    #[test]
    fn hover_scrim_is_subtle_and_translucent() {
        let Some(Background::Color(color)) = file_drop_hover_style(&iced::Theme::Dark).background
        else {
            panic!("file-drop hover must use a color scrim");
        };
        assert_eq!(color.a, 0.18);
    }

    #[test]
    fn hint_follows_the_cursor_with_a_small_offset() {
        assert_eq!(
            file_drop_hint_position(Point::new(300.0, 200.0), Size::new(1180.0, 720.0)),
            Point::new(316.0, 216.0)
        );
    }

    #[test]
    fn hint_flips_inside_the_viewport_near_bottom_right_corner() {
        assert_eq!(
            file_drop_hint_position(Point::new(1170.0, 710.0), Size::new(1180.0, 720.0)),
            Point::new(1022.0, 678.0)
        );
    }
}
