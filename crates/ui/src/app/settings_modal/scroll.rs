use iced::advanced::widget;
use iced::advanced::widget::operation::{Operation, Outcome, Scrollable};
use iced::{Rectangle, Task, Vector};

use crate::app::{Message, SETTINGS_CONTENT_SCROLL_ID};

const SCROLL_END_EPSILON: f32 = 0.5;

pub(crate) fn scroll_hint_visibility(
    offset: f32,
    viewport_height: f32,
    content_height: f32,
) -> (bool, bool) {
    (
        offset > SCROLL_END_EPSILON,
        content_height - viewport_height - offset > SCROLL_END_EPSILON,
    )
}

pub(super) fn scroll_settings_content_by(delta: f32) -> Task<Message> {
    widget::operate(ScrollSettingsContent {
        delta,
        result: None,
    })
    .map(
        |(can_scroll_up, can_scroll_down)| Message::SettingsContentScrolled {
            can_scroll_up,
            can_scroll_down,
        },
    )
}

struct ScrollSettingsContent {
    delta: f32,
    result: Option<(bool, bool)>,
}

impl Operation<(bool, bool)> for ScrollSettingsContent {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<(bool, bool)>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if id != Some(&widget::Id::new(SETTINGS_CONTENT_SCROLL_ID)) {
            return;
        }
        let max_offset = (content_bounds.height - bounds.height).max(0.0);
        let offset = (translation.y + self.delta).clamp(0.0, max_offset);
        state.scroll_by(
            widget::operation::scrollable::AbsoluteOffset {
                x: 0.0,
                y: self.delta,
            },
            bounds,
            content_bounds,
        );
        self.result = Some(scroll_hint_visibility(
            offset,
            bounds.height,
            content_bounds.height,
        ));
    }

    fn finish(&self) -> Outcome<(bool, bool)> {
        self.result.map_or(Outcome::None, Outcome::Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Point, Size};

    #[test]
    fn scroll_hints_follow_viewport_boundaries() {
        assert_eq!(scroll_hint_visibility(0.0, 360.0, 440.0), (false, true));
        assert_eq!(scroll_hint_visibility(40.0, 360.0, 440.0), (true, true));
        assert_eq!(scroll_hint_visibility(80.0, 360.0, 440.0), (true, false));
        assert_eq!(scroll_hint_visibility(0.0, 360.0, 200.0), (false, false));
    }

    #[test]
    fn wheel_past_fractional_bottom_does_not_reverse_scroll() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));
        let content_bounds = Rectangle::new(Point::ORIGIN, Size::new(100.0, 200.6));
        let mut state = TestScrollState { offset: 100.6 };
        let mut operation = ScrollSettingsContent {
            delta: 40.0,
            result: None,
        };

        operation.scrollable(
            Some(&widget::Id::new(SETTINGS_CONTENT_SCROLL_ID)),
            bounds,
            content_bounds,
            Vector::new(0.0, 101.0),
            &mut state,
        );

        assert!((state.offset - 100.6).abs() <= 0.001);
        assert!(matches!(operation.finish(), Outcome::Some((true, false))));
    }

    struct TestScrollState {
        offset: f32,
    }

    impl Scrollable for TestScrollState {
        fn snap_to(&mut self, _offset: widget::operation::scrollable::RelativeOffset<Option<f32>>) {
        }

        fn scroll_to(
            &mut self,
            _offset: widget::operation::scrollable::AbsoluteOffset<Option<f32>>,
        ) {
        }

        fn scroll_by(
            &mut self,
            offset: widget::operation::scrollable::AbsoluteOffset,
            bounds: Rectangle,
            content_bounds: Rectangle,
        ) {
            self.offset = (self.offset + offset.y)
                .clamp(0.0, (content_bounds.height - bounds.height).max(0.0));
        }
    }
}
