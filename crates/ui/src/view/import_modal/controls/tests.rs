use super::super::{DIALOG_WIDTH, ImportModalViewState, import_modal_overlay};
use super::*;
use iced::advanced::{Layout, Shell, clipboard, layout, renderer::Headless, widget};
use iced::{Event, Point, Rectangle, Size, Vector, mouse};
use std::collections::BTreeMap;

#[test]
fn target_options_match_the_selector_height_and_center_their_text() {
    let renderer = renderer();
    for label in [
        "Подпрограмма 1",
        "A very long worksheet name that must remain on one line",
    ] {
        let mut option = dropdown_option(label.to_owned(), true);
        let mut tree = widget::Tree::new(&option);
        let node = option.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(FIELD_WIDTH, 100.0)),
        );
        assert_eq!(node.size().height, ROW_HEIGHT);
        let row = Layout::new(&node);
        let label = row.children().next().unwrap().children().next().unwrap();
        assert!((label.bounds().center_y() - row.bounds().center_y()).abs() < 0.01);
    }
}

#[test]
fn target_selector_stays_close_to_the_footer() {
    let renderer = renderer();
    let options = vec!["Worksheet 1".to_owned(), "Worksheet 2".to_owned()];
    let mut root = modal(&options, false, Lang::En);
    let mut tree = widget::Tree::new(&root);
    let node = root.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(1060.0, 600.0)),
    );
    let mut bounds = BTreeMap::new();
    collect_bounds(Layout::new(&node), Vec::new(), &mut bounds);
    let target = bounds
        .values()
        .find(|rect| rect.width == FIELD_WIDTH && rect.height == ROW_HEIGHT)
        .unwrap();
    let footer = bounds
        .values()
        .find(|rect| rect.width == DIALOG_WIDTH && (20.0..ROW_HEIGHT).contains(&rect.height))
        .unwrap();
    let gap = footer.y - target.y - target.height;
    assert!(
        (8.0..=20.0).contains(&gap),
        "unexpected selector/footer gap: {gap}"
    );
}

#[test]
fn target_popup_keeps_the_dialog_and_existing_widget_states_stationary() {
    let renderer = renderer();
    for lang in [Lang::Ru, Lang::En] {
        for count in [1, 2, 8] {
            let options: Vec<_> = (0..count)
                .map(|index| {
                    format!(
                        "Long worksheet name with a full suffix that must survive selection {index}"
                    )
                })
                .collect();
            for size in [
                Size::new(580.0, 300.0),
                Size::new(1060.0, 600.0),
                Size::new(1280.0, 800.0),
            ] {
                let mut tree = {
                    let root = modal(&options, false, lang);
                    widget::Tree::new(&root)
                };
                let mut baseline: Option<(BTreeMap<_, _>, Vec<_>)> = None;
                for open in [false, true, false, true, false] {
                    let mut root = modal(&options, open, lang);
                    tree.diff(&root);
                    let node = root.as_widget_mut().layout(
                        &mut tree,
                        &renderer,
                        &layout::Limits::new(Size::ZERO, size),
                    );
                    let mut bounds = BTreeMap::new();
                    collect_bounds(Layout::new(&node), Vec::new(), &mut bounds);
                    let mut states = Vec::new();
                    collect_states(&tree, &mut states);
                    if let Some((before_bounds, before_states)) = &baseline {
                        for (path, before) in before_bounds {
                            assert_eq!(bounds.get(path), Some(before), "layout moved: {path:?}");
                        }
                        for state in before_states {
                            assert!(states.contains(state), "existing widget state was replaced");
                        }
                    } else {
                        baseline = Some((bounds, states));
                    }
                    let mut popup = root.as_widget_mut().overlay(
                        &mut tree,
                        Layout::new(&node),
                        &renderer,
                        &Rectangle::with_size(size),
                        Vector::ZERO,
                    );
                    assert_eq!(popup.is_some(), open);
                    if let Some(popup) = popup.as_mut() {
                        let node = popup.as_overlay_mut().layout(&renderer, size);
                        let mut bounds = BTreeMap::new();
                        collect_bounds(Layout::new(&node), Vec::new(), &mut bounds);
                        let panel = bounds
                            .values()
                            .find(|rect| {
                                rect.width == FIELD_WIDTH
                                    && rect.height
                                        == count.min(2) as f32 * ROW_HEIGHT
                                            + if count > 2 { 17.0 } else { 0.0 }
                                            + 8.0
                            })
                            .expect("full-height popup");
                        assert!(panel.y >= 0.0 && panel.y + panel.height <= size.height);
                    }
                }
            }
        }
    }
}

#[test]
fn scrolling_target_popup_selects_the_full_name_of_the_last_visible_option() {
    let renderer = renderer();
    let options: Vec<_> = (0..8)
        .map(|index| format!("Full worksheet name with a long non-visible suffix {index}"))
        .collect();
    let mut control = anchored_overlay(
        target_anchor(&options[0], false),
        dropdown(&options, Some(0)),
        true,
        DROPDOWN_ANCHOR_GAP,
        Message::ImportTargetDropdownToggled,
    );
    let mut tree = widget::Tree::new(&control);
    let viewport = Rectangle::with_size(Size::new(FIELD_WIDTH, 200.0));
    let anchor = control.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    let mut popup = control
        .as_widget_mut()
        .overlay(
            &mut tree,
            Layout::new(&anchor),
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .unwrap();
    let node = popup.as_overlay_mut().layout(&renderer, viewport.size());
    let cursor = mouse::Cursor::Available(Point::new(
        30.0,
        ROW_HEIGHT + DROPDOWN_ANCHOR_GAP + DROPDOWN_PANEL_PADDING + ROW_HEIGHT * 2.25,
    ));
    let mut messages = Vec::new();
    for event in [
        mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
        },
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        popup.as_overlay_mut().update(
            &Event::Mouse(event),
            Layout::new(&node),
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
        );
    }
    assert!(
        matches!(messages.as_slice(), [Message::ImportTargetSelected(name)] if name == &options[7])
    );
    messages.clear();
    popup.as_overlay_mut().update(
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(10.0, 180.0)),
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
    );
    assert!(matches!(
        messages.as_slice(),
        [Message::ImportTargetDropdownToggled]
    ));
}

#[test]
fn target_popup_scrolls_by_rows_with_a_wheel_and_pixels_with_a_touchpad() {
    let renderer = renderer();
    let options: Vec<_> = (0..8).map(|index| format!("Worksheet {index}")).collect();
    let mut control = anchored_overlay(
        target_anchor(&options[0], false),
        dropdown(&options, Some(0)),
        true,
        DROPDOWN_ANCHOR_GAP,
        Message::ImportTargetDropdownToggled,
    );
    let mut tree = widget::Tree::new(&control);
    let viewport = Rectangle::with_size(Size::new(FIELD_WIDTH, 200.0));
    let anchor = control.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    let mut popup = control
        .as_widget_mut()
        .overlay(
            &mut tree,
            Layout::new(&anchor),
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .unwrap();
    let node = popup.as_overlay_mut().layout(&renderer, viewport.size());
    let inside = Point::new(30.0, ROW_HEIGHT + DROPDOWN_ANCHOR_GAP + ROW_HEIGHT);
    let outside = Point::new(10.0, 180.0);
    let lines = |y| mouse::ScrollDelta::Lines { x: 0.0, y };
    let pixels = |y| mouse::ScrollDelta::Pixels { x: 0.0, y };
    for (delta, position, expected) in [
        (lines(-1.0), inside, ROW_HEIGHT),
        (lines(-1.0), inside, ROW_HEIGHT * 2.0),
        (lines(1.0), inside, ROW_HEIGHT),
        (lines(-100.0), inside, 187.0),
        (lines(1.0), inside, 153.0),
        (lines(100.0), inside, 0.0),
        (lines(-1.0), outside, 0.0),
        (pixels(-12.25), inside, 12.0),
        (pixels(-12.25), inside, 25.0),
        (pixels(4.25), inside, 20.0),
        (pixels(100.0), inside, 0.0),
    ] {
        let mut messages = Vec::new();
        popup.as_overlay_mut().update(
            &Event::Mouse(mouse::Event::WheelScrolled { delta }),
            Layout::new(&node),
            mouse::Cursor::Available(position),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
        );
        assert!(messages.is_empty());
        let mut offset = ScrollOffset(None);
        popup
            .as_overlay_mut()
            .operate(Layout::new(&node), &renderer, &mut offset);
        assert_eq!(offset.0, Some(expected), "unexpected scroll for {delta:?}");
    }
}

struct ScrollOffset(Option<f32>);

impl widget::Operation for ScrollOffset {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn widget::operation::Scrollable,
    ) {
        assert!(self.0.replace(translation.y).is_none());
    }
}

fn modal(options: &[String], open: bool, lang: Lang) -> Element<'_, Message> {
    import_modal_overlay(ImportModalViewState {
        focus: ImportModalFocus::Target,
        keyboard_focus_visible: false,
        file_drag_hovered: false,
        file_display: "fixture.xlsx",
        format: Some(ImportFileFormat::Xlsx),
        loading: false,
        target_input: &options[0],
        target_options: options,
        target_dropdown_open: open,
        target_highlight: Some(0),
        error: None,
        lang,
    })
}

fn collect_bounds(
    layout: Layout<'_>,
    path: Vec<usize>,
    bounds: &mut BTreeMap<Vec<usize>, Rectangle>,
) {
    bounds.insert(path.clone(), layout.bounds());
    for (index, child) in layout.children().enumerate() {
        let mut path = path.clone();
        path.push(index);
        collect_bounds(child, path, bounds);
    }
}

fn collect_states(tree: &widget::Tree, states: &mut Vec<*const ()>) {
    if let widget::tree::State::Some(state) = &tree.state {
        states.push(state.as_ref() as *const dyn std::any::Any as *const ());
    }
    for child in &tree.children {
        collect_states(child, states);
    }
}

fn renderer() -> iced::Renderer {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            13.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap()
}
