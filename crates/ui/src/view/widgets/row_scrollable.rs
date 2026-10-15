use crate::app::Message;
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::widget::Scrollable;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

// iced 0.14 converts every wheel line to a fixed 60 logical pixels.
const ICED_LINE_HEIGHT: f32 = 60.0;

pub(in crate::view) fn dropdown_height(count: usize, full_rows: usize, row_height: f32) -> f32 {
    let preview = if count > full_rows {
        (row_height * 0.5).ceil()
    } else {
        0.0
    };
    count.min(full_rows) as f32 * row_height + preview
}

pub(in crate::view) fn row_scrollable<'a>(
    inner: Scrollable<'a, Message>,
    row_height: f32,
) -> Element<'a, Message> {
    Element::new(RowScrollable { inner, row_height })
}

struct RowScrollable<'a> {
    inner: Scrollable<'a, Message>,
    row_height: f32,
}

impl Widget<Message, iced::Theme, iced::Renderer> for RowScrollable<'_> {
    fn tag(&self) -> widget::tree::Tag {
        self.inner.tag()
    }

    fn state(&self) -> widget::tree::State {
        self.inner.state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.inner.children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.inner.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.inner.size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.inner.layout(tree, renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let event = match event {
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x, y },
            }) => Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines {
                    x: x * self.row_height / ICED_LINE_HEIGHT,
                    y: y * self.row_height / ICED_LINE_HEIGHT,
                },
            }),
            _ => event.clone(),
        };
        self.inner.update(
            tree, &event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.inner
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.inner
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.inner.operate(tree, layout, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, iced::Theme, iced::Renderer>> {
        self.inner
            .overlay(tree, layout, renderer, viewport, translation)
    }
}
