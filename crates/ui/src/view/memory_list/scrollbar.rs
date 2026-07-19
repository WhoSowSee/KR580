use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer, widget,
};
use iced::{Background, Border, Element, Event, Length, Rectangle, Size};

use super::super::styles::memory_scrollbar_color;
use crate::app::Message;

const HIT_WIDTH: f32 = 8.0;
const THUMB_WIDTH: f32 = 4.0;
const THUMB_HEIGHT: f32 = 20.0;
const PRECISION_DRAG_DISTANCE: f32 = 12.0;

pub(super) fn memory_scrollbar(
    offset: f32,
    max_offset: f32,
    viewport_height: f32,
    reveal: bool,
) -> Element<'static, Message> {
    Element::new(MemoryScrollbar {
        offset,
        max_offset,
        viewport_height,
        reveal,
    })
}

struct MemoryScrollbar {
    offset: f32,
    max_offset: f32,
    viewport_height: f32,
    reveal: bool,
}

#[derive(Debug, Clone, Copy)]
struct DragOrigin {
    cursor_y: f32,
    handle_y: f32,
}

#[derive(Debug, Default)]
struct State {
    drag_origin: Option<DragOrigin>,
    track_hovered: bool,
}

impl Widget<Message, iced::Theme, iced::Renderer> for MemoryScrollbar {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(HIT_WIDTH), Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(HIT_WIDTH), Length::Fill)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let handle = handle_bounds(layout.bounds(), self.offset, self.max_offset);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position()
                    && handle.contains(position)
                {
                    state.drag_origin = Some(DragOrigin {
                        cursor_y: position.y,
                        handle_y: handle.y,
                    });
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(origin) = state.drag_origin {
                    let offset =
                        drag_target_offset(layout.bounds(), origin, position.y, self.max_offset);
                    if (offset - self.offset).abs() > f32::EPSILON {
                        shell.publish(Message::MemoryScrollbarDragged(
                            offset,
                            self.viewport_height,
                        ));
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag_origin.take().is_some() =>
            {
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }

        let track_hovered = cursor.is_over(layout.bounds());
        if state.track_hovered != track_hovered {
            state.track_hovered = track_hovered;
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let handle = handle_bounds(layout.bounds(), self.offset, self.max_offset);
        let state = tree.state.downcast_ref::<State>();
        let hovered = cursor.is_over(layout.bounds());
        let color = memory_scrollbar_color(self.reveal, hovered, state.drag_origin.is_some());

        renderer.fill_quad(
            renderer::Quad {
                bounds: handle,
                border: Border {
                    radius: 2.0.into(),
                    ..Border::default()
                },
                ..renderer::Quad::default()
            },
            Background::Color(color),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let handle = handle_bounds(layout.bounds(), self.offset, self.max_offset);

        if state.drag_origin.is_some() || cursor.is_over(handle) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

fn handle_bounds(bounds: Rectangle, offset: f32, max_offset: f32) -> Rectangle {
    let height = THUMB_HEIGHT.min(bounds.height);
    let travel = (bounds.height - height).max(0.0);
    let progress = if max_offset > 0.0 {
        (offset / max_offset).clamp(0.0, 1.0)
    } else {
        0.0
    };

    Rectangle {
        x: bounds.x + bounds.width - THUMB_WIDTH,
        y: bounds.y + travel * progress,
        width: THUMB_WIDTH,
        height,
    }
}

fn drag_target_offset(
    bounds: Rectangle,
    origin: DragOrigin,
    cursor_y: f32,
    max_offset: f32,
) -> f32 {
    let height = THUMB_HEIGHT.min(bounds.height);
    let travel = bounds.height - height;
    if travel <= 0.0 {
        return 0.0;
    }

    let pointer_delta = cursor_y - origin.cursor_y;
    let handle_y = origin.handle_y + precision_adjusted_drag_delta(pointer_delta);
    let progress = (handle_y - bounds.y) / travel;
    progress.clamp(0.0, 1.0) * max_offset
}

fn precision_adjusted_drag_delta(pointer_delta: f32) -> f32 {
    let distance = pointer_delta.abs();
    if distance >= PRECISION_DRAG_DISTANCE {
        return pointer_delta;
    }

    let progress = distance / PRECISION_DRAG_DISTANCE;
    let gain = progress * progress * (3.0 - 2.0 * progress);
    pointer_delta * gain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grabbing_off_center_does_not_move_thumb() {
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(8.0, 300.0));
        let max_offset = 1_000_000.0;
        let offset = 375_000.0;
        let handle = handle_bounds(bounds, offset, max_offset);
        let cursor_y = handle.y + handle.height * 0.25;
        let origin = DragOrigin {
            cursor_y,
            handle_y: handle.y,
        };

        assert!((drag_target_offset(bounds, origin, cursor_y, max_offset) - offset).abs() < 0.1);
    }

    #[test]
    fn minimal_drag_uses_precision_zone() {
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(8.0, 300.0));
        let max_offset = 1_000_000.0;
        let offset = 250_000.0;
        let handle = handle_bounds(bounds, offset, max_offset);
        let origin = DragOrigin {
            cursor_y: handle.y + handle.height * 0.5,
            handle_y: handle.y,
        };
        let target = drag_target_offset(bounds, origin, origin.cursor_y + 1.0, max_offset);
        let moved_handle = handle_bounds(bounds, target, max_offset);

        assert!(moved_handle.y > handle.y);
        assert!(moved_handle.y - handle.y < 0.03);
    }

    #[test]
    fn thumb_catches_pointer_and_tracks_one_to_one() {
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(8.0, 300.0));
        let max_offset = 1_000_000.0;
        let offset = 250_000.0;
        let handle = handle_bounds(bounds, offset, max_offset);
        let origin = DragOrigin {
            cursor_y: handle.y + handle.height * 0.5,
            handle_y: handle.y,
        };
        let pointer_delta = 50.0;
        let target =
            drag_target_offset(bounds, origin, origin.cursor_y + pointer_delta, max_offset);
        let moved_handle = handle_bounds(bounds, target, max_offset);

        assert!((moved_handle.y - handle.y - pointer_delta).abs() < 0.001);
        assert_eq!(
            precision_adjusted_drag_delta(PRECISION_DRAG_DISTANCE),
            PRECISION_DRAG_DISTANCE
        );
    }

    #[test]
    fn drag_clamps_at_track_ends() {
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(8.0, 300.0));
        let origin = DragOrigin {
            cursor_y: 100.0,
            handle_y: 100.0,
        };

        assert_eq!(drag_target_offset(bounds, origin, -1_000.0, 60_000.0), 0.0);
        assert_eq!(
            drag_target_offset(bounds, origin, 1_000.0, 60_000.0),
            60_000.0
        );
    }

    #[test]
    fn thumb_stays_inside_track() {
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(8.0, 300.0));

        assert_eq!(handle_bounds(bounds, 0.0, 100.0).y, 0.0);
        assert_eq!(handle_bounds(bounds, 100.0, 100.0).y, 280.0);
    }
}
