use iced::advanced::{clipboard, renderer::Headless};
use iced::widget::{button, opaque, responsive, scrollable, stack};
use iced::{Point, window};

use super::*;

#[test]
fn minimal_drag_uses_precision_zone() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(8.0, 300.0));
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
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(8.0, 300.0));
    let max_offset = 1_000_000.0;
    let offset = 250_000.0;
    let handle = handle_bounds(bounds, offset, max_offset);
    let origin = DragOrigin {
        cursor_y: handle.y + handle.height * 0.5,
        handle_y: handle.y,
    };
    for pointer_delta in [PRECISION_DRAG_DISTANCE, 50.0] {
        let target =
            drag_target_offset(bounds, origin, origin.cursor_y + pointer_delta, max_offset);
        let moved_handle = handle_bounds(bounds, target, max_offset);

        assert!((moved_handle.y - handle.y - pointer_delta).abs() < 0.001);
    }
}

#[test]
fn drag_clamps_at_track_ends() {
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(8.0, 300.0));
    let origin = DragOrigin {
        cursor_y: 100.0,
        handle_y: 100.0,
    };

    for (cursor_y, expected_offset, expected_top) in
        [(-1_000.0, 0.0, 0.0), (1_000.0, 60_000.0, 280.0)]
    {
        let offset = drag_target_offset(bounds, origin, cursor_y, 60_000.0);

        assert_eq!(offset, expected_offset);
        assert_eq!(handle_bounds(bounds, offset, 60_000.0).y, expected_top);
    }
}

#[test]
fn full_width_thumb_grab_does_not_activate_underlying_option() {
    for x in [194.0, 198.0] {
        let mut scene = TestScene::new();
        let grab = Point::new(x, 110.0);
        scene.mouse(mouse::Event::ButtonPressed(mouse::Button::Left), grab);
        scene.mouse(mouse::Event::CursorMoved { position: grab }, grab);
        assert!(scene.messages.is_empty());

        let moved = Point::new(x, 150.0);
        scene.mouse(mouse::Event::CursorMoved { position: moved }, moved);
        scene.mouse(mouse::Event::ButtonReleased(mouse::Button::Left), moved);

        assert!(matches!(
            scene.messages.as_slice(),
            [Message::OpcodeScrollbarDragged(_)]
        ));
    }
}

#[test]
fn rail_clicks_do_not_activate_underlying_option() {
    for release in [Point::new(194.0, 250.0), Point::new(180.0, 250.0)] {
        let mut scene = TestScene::new();
        scene.mouse(
            mouse::Event::ButtonPressed(mouse::Button::Left),
            Point::new(194.0, 250.0),
        );
        scene.mouse(mouse::Event::CursorMoved { position: release }, release);
        scene.mouse(mouse::Event::ButtonReleased(mouse::Button::Left), release);
        assert!(scene.messages.is_empty());

        let option = Point::new(180.0, 250.0);
        scene.mouse(mouse::Event::ButtonPressed(mouse::Button::Left), option);
        scene.mouse(mouse::Event::ButtonReleased(mouse::Button::Left), option);
        assert!(matches!(
            scene.messages.as_slice(),
            [Message::OpcodeSelected(0x1234, 0x00)]
        ));
    }
}

#[test]
fn wheel_over_thumb_matches_native_scrollable() {
    for delta in [
        mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        mouse::ScrollDelta::Pixels { x: 0.0, y: -12.5 },
    ] {
        for position in [
            Point::new(100.0, 10.0),
            Point::new(194.0, 10.0),
            Point::new(198.0, 10.0),
            Point::new(194.0, 250.0),
        ] {
            let mut native = TestScene::with_root(native_scrollable());
            native.mouse(mouse::Event::WheelScrolled { delta }, position);
            let [Message::OpcodeScrolled(expected)] = native.messages.as_slice() else {
                panic!("native scrollable did not report its offset");
            };

            for responsive_thumb in [false, true] {
                let mut wrapped = TestScene::with_root(scrollable_with_thumb(responsive_thumb));
                wrapped.mouse(mouse::Event::WheelScrolled { delta }, position);
                assert!(
                    matches!(wrapped.messages.as_slice(), [Message::OpcodeScrolled(actual)] if actual == expected),
                    "cursor {position:?}, delta {delta:?}: {:?}",
                    wrapped.messages
                );

                let thumb = Point::new(198.0, 10.0);
                wrapped.event(
                    Event::Window(window::Event::RedrawRequested(iced::time::Instant::now())),
                    thumb,
                );
                let layout = Layout::new(&wrapped.layout);
                let viewport = layout.bounds();
                assert_eq!(
                    wrapped.root.as_widget().mouse_interaction(
                        &wrapped.tree,
                        layout,
                        mouse::Cursor::Available(thumb),
                        &viewport,
                        &wrapped.renderer,
                    ),
                    mouse::Interaction::Pointer
                );
            }
        }
    }
}

#[test]
fn wheel_does_not_pass_through_covering_overlay() {
    let root = stack![
        scrollable_with_thumb(true),
        opaque(Space::new().width(Length::Fill).height(Length::Fill)),
    ]
    .into();
    let mut scene = TestScene::with_root(root);
    scene.mouse(
        mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        },
        Point::new(198.0, 10.0),
    );

    assert!(scene.messages.is_empty());
}

fn native_scrollable() -> Element<'static, Message> {
    scrollable(Space::new().width(Length::Fill).height(1_300.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .on_scroll(|viewport| Message::OpcodeScrolled(viewport.absolute_offset().y))
        .into()
}

fn scrollable_with_thumb(responsive_thumb: bool) -> Element<'static, Message> {
    let thumb = if responsive_thumb {
        responsive(|size| {
            compact_scrollbar(
                0.0,
                1_300.0 - size.height,
                false,
                Message::OpcodeScrollbarDragged,
            )
        })
        .into()
    } else {
        compact_scrollbar(0.0, 1_000.0, false, Message::OpcodeScrollbarDragged)
    };

    stack![native_scrollable(), thumb].into()
}

struct TestScene {
    root: Element<'static, Message>,
    tree: widget::Tree,
    layout: layout::Node,
    renderer: iced::Renderer,
    messages: Vec<Message>,
}

impl TestScene {
    fn new() -> Self {
        let root = stack![
            button(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .on_press(Message::OpcodeSelected(0x1234, 0x00)),
            compact_scrollbar(
                375_000.0,
                1_000_000.0,
                false,
                Message::OpcodeScrollbarDragged
            ),
        ]
        .into();
        Self::with_root(root)
    }

    fn with_root(mut root: Element<'static, Message>) -> Self {
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("renderer runtime is available")
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .expect("software renderer is available");
        let mut tree = widget::Tree::new(&root);
        let layout = root.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(200.0, 300.0)),
        );

        Self {
            root,
            tree,
            layout,
            renderer,
            messages: Vec::new(),
        }
    }

    fn mouse(&mut self, event: mouse::Event, position: Point) {
        self.event(Event::Mouse(event), position);
    }

    fn event(&mut self, event: Event, position: Point) {
        let layout = Layout::new(&self.layout);
        let viewport = layout.bounds();
        let mut shell = Shell::new(&mut self.messages);
        self.root.as_widget_mut().update(
            &mut self.tree,
            &event,
            layout,
            mouse::Cursor::Available(position),
            &self.renderer,
            &mut clipboard::Null,
            &mut shell,
            &viewport,
        );
    }
}
