use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::app::Message;
use crate::app::shortcuts::{binding_from_event, message_for_action};
use crate::persistence::{ShortcutAction, ShortcutSettings};

pub(in crate::view) fn shortcut_capture<'a>(
    content: impl Into<Element<'a, Message>>,
    settings: &'a ShortcutSettings,
    action: ShortcutAction,
    active: bool,
) -> Element<'a, Message> {
    Element::new(ShortcutCapture {
        content: content.into(),
        settings,
        action,
        active,
    })
}

struct ShortcutCapture<'a> {
    content: Element<'a, Message>,
    settings: &'a ShortcutSettings,
    action: ShortcutAction,
    active: bool,
}

impl Widget<Message, iced::Theme, iced::Renderer> for ShortcutCapture<'_> {
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
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
        if self.active
            && let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                physical_key,
                modifiers,
                ..
            }) = event
            && binding_from_event(*physical_key, *modifiers)
                .is_some_and(|binding| self.settings.matches(self.action, binding))
        {
            shell.publish(message_for_action(self.action));
            shell.capture_event();
            return;
        }
        self.content.as_widget_mut().update(
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
        self.content.as_widget().draw(
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
        self.content.as_widget().mouse_interaction(
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
        self.content
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use iced::advanced::renderer::Headless;
    use iced::advanced::{Layout, Shell, clipboard, layout, widget};
    use iced::keyboard;
    use iced::keyboard::key::{Code, Physical};
    use iced::widget::text_input;
    use iced::{Event, Font, Size};

    use super::shortcut_capture;
    use crate::app::Message;
    use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings};

    #[test]
    fn printable_shortcut_is_captured_before_text_input() {
        let mut settings = ShortcutSettings::default();
        settings.assign(
            ShortcutAction::MemoryPatternSearch,
            ShortcutBinding::new(false, false, true, ShortcutKey::F),
        );
        let input = text_input("", "FF").on_input(Message::MemoryAddressChanged);
        let mut root =
            shortcut_capture(input, &settings, ShortcutAction::MemoryPatternSearch, true);
        let renderer = test_renderer();
        let mut tree = widget::Tree::new(&root);
        tree.children[0]
            .state
            .downcast_mut::<
                text_input::State<
                    <iced::Renderer as iced::advanced::text::Renderer>::Paragraph,
                >,
            >()
            .focus();
        let node = root.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(100.0, 30.0)),
        );
        let bounds = node.bounds();
        let cursor = iced::mouse::Cursor::Unavailable;
        let mut messages = Vec::new();
        let key = keyboard::Key::Character("f".into());
        let shortcut = Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::KeyF),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::ALT,
            text: Some("f".into()),
            repeat: false,
        });
        root.as_widget_mut().update(
            &mut tree,
            &shortcut,
            Layout::new(&node),
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &bounds,
        );

        assert!(matches!(
            messages.as_slice(),
            [Message::MemoryPatternSearch]
        ));
    }

    fn test_renderer() -> iced::Renderer {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("renderer runtime")
            .block_on(iced::Renderer::new(
                Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .expect("software renderer")
    }
}
