//! Floating opcode picker that drops out of a memory row.

use iced::widget::{Space, button, column, container, opaque, row, scrollable, stack, text_input};
use iced::{Element, Length, alignment};

use super::styles::{input_borderless_style, opcode_dropdown_style, opcode_option_style};
use super::theme::{MONO_FONT, mono_text, tokyo_green, tokyo_text};
use super::utils::row_separator;
use super::widgets::compact_scrollbar;
use crate::app::{
    Message, OPCODE_LIST_HEIGHT, OPCODE_OPTION_HEIGHT, OPCODE_SCROLL_ID, OPCODE_SEARCH_INPUT_ID,
    OpcodeChoice, filtered_opcode_choices,
};
use crate::i18n::{Key, Lang};

pub(super) const OPCODE_DROPDOWN_HEIGHT: f32 = 224.0;

pub(super) fn opcode_dropdown_overlay<'a>(
    address: u16,
    search: &'a str,
    highlighted: usize,
    scroll_offset: f32,
    reveal: bool,
    top: f32,
    lang: Lang,
) -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fixed(top)),
        row![
            Space::new().width(Length::Fill),
            opaque(opcode_dropdown(
                address,
                search,
                highlighted,
                scroll_offset,
                reveal,
                lang,
            )),
            Space::new().width(Length::Fixed(24.0)),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn opcode_dropdown<'a>(
    address: u16,
    search: &'a str,
    highlighted: usize,
    scroll_offset: f32,
    reveal: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let choices = filtered_opcode_choices(search);
    let max_offset = choices.len() as f32 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT;
    let options = column(
        choices
            .into_iter()
            .enumerate()
            .map(|(index, choice)| opcode_option(address, choice, index == highlighted)),
    );

    let options = scrollable(options)
        .id(OPCODE_SCROLL_ID)
        .height(Length::Fixed(OPCODE_LIST_HEIGHT))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .on_scroll(|viewport| Message::OpcodeScrolled(viewport.absolute_offset().y));
    let options = stack![
        options,
        compact_scrollbar(
            scroll_offset,
            max_offset,
            reveal,
            Message::OpcodeScrollbarDragged,
        ),
    ]
    .width(Length::Fill)
    .height(Length::Fixed(OPCODE_LIST_HEIGHT));

    let content = column![
        text_input(lang.t(Key::OpcodeSearchPlaceholder), search)
            .id(OPCODE_SEARCH_INPUT_ID)
            .on_input(Message::OpcodeSearchChanged)
            .font(MONO_FONT)
            .size(13)
            .padding(6)
            .width(Length::Fill)
            .style(input_borderless_style),
        row_separator(),
        options,
    ]
    .spacing(4);

    container(content)
        .padding(6)
        .width(Length::Fixed(226.0))
        .style(opcode_dropdown_style)
        .into()
}

fn opcode_option(
    address: u16,
    choice: OpcodeChoice,
    highlighted: bool,
) -> Element<'static, Message> {
    button(
        row![
            mono_text(format!("{:02X}", choice.value), 13, tokyo_green())
                .width(Length::Fixed(34.0)),
            mono_text(choice.mnemonic, 13, tokyo_text()).width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::OpcodeSelected(address, choice.value))
    .padding(5)
    .width(Length::Fill)
    .height(Length::Fixed(OPCODE_OPTION_HEIGHT))
    .style(move |_theme, status| opcode_option_style(status, highlighted))
    .into()
}

#[cfg(test)]
mod tests {
    use iced::advanced::{Layout, Shell, clipboard, layout, mouse, renderer::Headless, widget};
    use iced::{Event, Point, Rectangle, Size, Vector};

    use super::*;

    #[test]
    fn scrollbar_drag_and_wheel_work_after_search_focus() {
        let after_tab = 10.0 * OPCODE_OPTION_HEIGHT - OPCODE_LIST_HEIGHT;
        for (offset, highlighted) in [(0.0, 0), (after_tab, 9)] {
            let mut scene = PickerScene::new(offset, highlighted);
            scene.operate(&mut widget::operation::scrollable::scroll_to::<()>(
                OPCODE_SCROLL_ID.into(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: offset }.into(),
            ));
            scene.operate(&mut widget::operation::focusable::focus::<()>(
                OPCODE_SEARCH_INPUT_ID.into(),
            ));

            let mut geometry = ScrollGeometry::default();
            scene.operate(&mut geometry);
            let bounds = geometry.bounds.expect("opcode scroll bounds");
            let grab = Point::new(
                bounds.x + bounds.width - 2.0,
                bounds.y + bounds.height * offset / geometry.max_offset + 5.0,
            );
            scene.event(
                Event::Window(iced::window::Event::RedrawRequested(
                    iced::time::Instant::now(),
                )),
                grab,
            );
            let moved = Point::new(grab.x, grab.y + 60.0);
            scene.messages.clear();
            scene.event(
                Event::Mouse(mouse::Event::CursorMoved { position: grab }),
                moved,
            );
            scene.event(
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                moved,
            );
            scene.event(
                Event::Mouse(mouse::Event::CursorMoved { position: moved }),
                moved,
            );
            scene.event(
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                moved,
            );

            let [Message::OpcodeScrollbarDragged(target)] = scene.messages.as_slice() else {
                panic!("offset {offset}: {:?}", scene.messages);
            };
            let target = *target;
            assert!(target > offset);
            scene.operate(&mut widget::operation::scrollable::scroll_to::<()>(
                OPCODE_SCROLL_ID.into(),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target }.into(),
            ));
            scene.messages.clear();
            scene.event(
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -12.5 },
                }),
                grab,
            );
            assert!(matches!(
                scene.messages.as_slice(),
                [Message::OpcodeScrolled(actual)] if *actual == target + 12.5
            ));
        }
    }

    #[derive(Default)]
    struct ScrollGeometry {
        bounds: Option<Rectangle>,
        max_offset: f32,
    }

    impl widget::Operation for ScrollGeometry {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            _state: &mut dyn widget::operation::Scrollable,
        ) {
            if id == Some(&widget::Id::new(OPCODE_SCROLL_ID)) {
                self.bounds = Some(bounds);
                self.max_offset = content_bounds.height - bounds.height;
            }
        }
    }

    struct PickerScene {
        root: Element<'static, Message>,
        tree: widget::Tree,
        layout: layout::Node,
        renderer: iced::Renderer,
        messages: Vec<Message>,
    }

    impl PickerScene {
        fn new(offset: f32, highlighted: usize) -> Self {
            let renderer = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("renderer runtime")
                .block_on(iced::Renderer::new(
                    iced::Font::DEFAULT,
                    13.0.into(),
                    Some("tiny-skia"),
                ))
                .expect("software renderer");
            let mut root =
                opcode_dropdown_overlay(0x1234, "", highlighted, offset, true, 0.0, Lang::En);
            let mut tree = widget::Tree::new(&root);
            let layout = root.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(330.0, 400.0)),
            );
            Self {
                root,
                tree,
                layout,
                renderer,
                messages: Vec::new(),
            }
        }

        fn operate(&mut self, operation: &mut dyn widget::Operation) {
            self.root.as_widget_mut().operate(
                &mut self.tree,
                Layout::new(&self.layout),
                &self.renderer,
                operation,
            );
        }

        fn event(&mut self, event: Event, position: Point) {
            let layout = Layout::new(&self.layout);
            self.root.as_widget_mut().update(
                &mut self.tree,
                &event,
                layout,
                mouse::Cursor::Available(position),
                &self.renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut self.messages),
                &layout.bounds(),
            );
        }
    }
}
