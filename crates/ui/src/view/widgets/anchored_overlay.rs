use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

pub(crate) fn anchored_overlay<'a, Message: Clone + 'a>(
    anchor: impl Into<Element<'a, Message>>,
    panel: impl Into<Element<'a, Message>>,
    open: bool,
    gap: f32,
    dismiss: Message,
) -> Element<'a, Message> {
    Element::new(AnchoredOverlay {
        anchor: anchor.into(),
        panel: panel.into(),
        open,
        gap,
        dismiss,
    })
}

struct AnchoredOverlay<'a, Message> {
    anchor: Element<'a, Message>,
    panel: Element<'a, Message>,
    open: bool,
    gap: f32,
    dismiss: Message,
}

impl<Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for AnchoredOverlay<'_, Message> {
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.anchor),
            widget::Tree::new(&self.panel),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.anchor.as_widget(), self.panel.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
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
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, iced::Theme, iced::Renderer>> {
        let mut children = tree.children.iter_mut();
        let anchor = self.anchor.as_widget_mut().overlay(
            children.next().expect("anchor tree"),
            layout,
            renderer,
            viewport,
            translation,
        );
        let panel = self.open.then(|| {
            overlay::Element::new(Box::new(PanelOverlay {
                position: layout.position() + translation,
                anchor_bounds: layout.bounds(),
                panel: &mut self.panel,
                tree: children.next().expect("panel tree"),
                gap: self.gap,
                dismiss: self.dismiss.clone(),
            }))
        });

        if anchor.is_some() || panel.is_some() {
            Some(overlay::Group::with_children(anchor.into_iter().chain(panel).collect()).overlay())
        } else {
            None
        }
    }
}

struct PanelOverlay<'a, 'b, Message> {
    position: Point,
    anchor_bounds: Rectangle,
    panel: &'b mut Element<'a, Message>,
    tree: &'b mut widget::Tree,
    gap: f32,
    dismiss: Message,
}

impl<Message: Clone> overlay::Overlay<Message, iced::Theme, iced::Renderer>
    for PanelOverlay<'_, '_, Message>
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let space_below =
            (bounds.height - self.position.y - self.anchor_bounds.height - self.gap).max(0.0);
        let space_above = (self.position.y - self.gap).max(0.0);
        let available_height = space_below.max(space_above);
        let width = self.anchor_bounds.width.min(bounds.width);
        let panel = self.panel.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, available_height)).width(width),
        );
        let panel_size = panel.size();
        let x = self
            .position
            .x
            .clamp(0.0, (bounds.width - panel_size.width).max(0.0));
        let y = if panel_size.height <= space_below || space_below >= space_above {
            self.position.y + self.anchor_bounds.height + self.gap
        } else {
            (self.position.y - panel_size.height - self.gap).max(0.0)
        };

        layout::Node::with_children(panel_size, vec![panel]).move_to(Point::new(x, y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let viewport = layout.bounds();
        let panel_layout = layout.children().next().expect("panel layout");
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) && !cursor.is_over(panel_layout.bounds())
            && !cursor.is_over(self.anchor_bounds)
        {
            shell.publish(self.dismiss.clone());
        }
        self.panel.as_widget_mut().update(
            self.tree,
            event,
            panel_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();
        self.panel.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout.children().next().expect("panel layout"),
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let viewport = layout.bounds();
        self.panel.as_widget().mouse_interaction(
            self.tree,
            layout.children().next().expect("panel layout"),
            cursor,
            &viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.panel.as_widget_mut().operate(
            self.tree,
            layout.children().next().expect("panel layout"),
            renderer,
            operation,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'a, Message, iced::Theme, iced::Renderer>> {
        let viewport = layout.bounds();
        self.panel.as_widget_mut().overlay(
            self.tree,
            layout.children().next().expect("panel layout"),
            renderer,
            &viewport,
            Vector::ZERO,
        )
    }
}
