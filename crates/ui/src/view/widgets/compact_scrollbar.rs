use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer, widget,
};
use iced::widget::{Space, container};
use iced::{Background, Border, Element, Event, Length, Point, Rectangle, Size, alignment};

use super::super::styles::memory_scrollbar_color;
use crate::app::Message;

const HIT_WIDTH: f32 = 8.0;
const THUMB_WIDTH: f32 = 5.0;
const THUMB_HEIGHT: f32 = 28.0;
const PRECISION_DRAG_DISTANCE: f32 = 12.0;

pub(in crate::view) fn compact_scrollbar(
    offset: f32,
    max_offset: f32,
    reveal: bool,
    on_drag: impl Fn(f32) -> Message + 'static,
) -> Element<'static, Message> {
    if max_offset > 0.0 {
        container(Element::new(CompactScrollbar {
            offset,
            max_offset,
            reveal,
            forwarding_wheel: false,
            on_drag,
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .into()
    } else {
        Space::new().width(Length::Fill).height(Length::Fill).into()
    }
}

struct CompactScrollbar<F> {
    offset: f32,
    max_offset: f32,
    reveal: bool,
    forwarding_wheel: bool,
    on_drag: F,
}

#[derive(Debug, Clone, Copy)]
struct DragOrigin {
    cursor_y: f32,
    handle_y: f32,
}

#[derive(Debug, Default)]
struct State {
    drag_origin: Option<DragOrigin>,
    last_cursor_position: Option<Point>,
    track_hovered: bool,
}

impl<F: Fn(f32) -> Message> Widget<Message, iced::Theme, iced::Renderer> for CompactScrollbar<F> {
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
        let bounds = layout.bounds();
        self.forwarding_wheel = matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. }))
            && state.drag_origin.is_none()
            && cursor.is_over(handle_hit_bounds(bounds, self.offset, self.max_offset));
        if self.forwarding_wheel {
            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // iced 0.14 supplies the final cursor position to every event in a batch.
                let position = cursor
                    .position()
                    .map(|position| state.last_cursor_position.unwrap_or(position));
                if let Some(position) = position.filter(|position| bounds.contains(*position)) {
                    let hit = handle_hit_bounds(bounds, self.offset, self.max_offset);
                    let grabbed_thumb = hit.contains(position);
                    let origin = if grabbed_thumb {
                        DragOrigin {
                            cursor_y: position.y,
                            handle_y: hit.y,
                        }
                    } else {
                        centered_drag_origin(bounds, position.y)
                    };
                    state.drag_origin = Some(origin);
                    if !grabbed_thumb {
                        let offset =
                            drag_target_offset(bounds, origin, origin.cursor_y, self.max_offset);
                        shell.publish((self.on_drag)(offset));
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.last_cursor_position = Some(*position);
                if let Some(origin) = state.drag_origin {
                    let offset = drag_target_offset(bounds, origin, position.y, self.max_offset);
                    if (offset - self.offset).abs() > f32::EPSILON {
                        shell.publish((self.on_drag)(offset));
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => state.last_cursor_position = None,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag_origin.take().is_some() =>
            {
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }

        let track_hovered = cursor.is_over(bounds);
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
        // iced 0.14 Stack uses mouse_interaction to gate wheel input to lower layers.
        if self.forwarding_wheel {
            return mouse::Interaction::None;
        }

        let state = tree.state.downcast_ref::<State>();
        let hit = handle_hit_bounds(layout.bounds(), self.offset, self.max_offset);

        if state.drag_origin.is_some() || cursor.is_over(hit) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

fn handle_hit_bounds(bounds: Rectangle, offset: f32, max_offset: f32) -> Rectangle {
    Rectangle {
        x: bounds.x,
        width: bounds.width,
        ..handle_bounds(bounds, offset, max_offset)
    }
}

fn handle_bounds(bounds: Rectangle, offset: f32, max_offset: f32) -> Rectangle {
    let height = THUMB_HEIGHT.min(bounds.height);
    let travel = (bounds.height - height).max(0.0);
    let progress = (offset / max_offset).clamp(0.0, 1.0);

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

fn centered_drag_origin(bounds: Rectangle, cursor_y: f32) -> DragOrigin {
    let height = THUMB_HEIGHT.min(bounds.height);
    let handle_y = (cursor_y - height * 0.5).clamp(bounds.y, bounds.y + bounds.height - height);
    DragOrigin {
        cursor_y: handle_y + height * 0.5,
        handle_y,
    }
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
mod tests;
