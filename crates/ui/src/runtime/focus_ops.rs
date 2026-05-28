//! Custom widget operations for post-click focus reconciliation.
//!
//! Iced 0.14 only converges focus inside flat containers — across
//! stacked panels sibling text_inputs end up with stale
//! `is_focused = Some(_)`. The two-pass split below is what unfocuses
//! them: a single-pass walk would also unfocus the input that just
//! processed the click when layout drifts a pixel, so we only mutate
//! state after `find_focusable_at` confirms a hit.

use iced::Point;
use iced::Rectangle;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Focusable, Operation, Outcome};

pub(crate) fn find_focusable_at(point: Point) -> impl Operation<Option<Id>> {
    struct FindFocusableAt {
        point: Point,
        hit: Option<Id>,
    }

    impl Operation<Option<Id>> for FindFocusableAt {
        fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, _state: &mut dyn Focusable) {
            if id.is_some() && bounds.contains(self.point) {
                self.hit = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            Outcome::Some(self.hit.clone())
        }
    }

    FindFocusableAt { point, hit: None }
}

/// Like iced's built-in `find_focused`, but always reports —
/// built-in returns `Outcome::None` when nothing is focused, and
/// `Task::map` then silently drops the focus-clear message.
pub(crate) fn find_focused_optional() -> impl Operation<Option<Id>> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Option<Id>> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            Outcome::Some(self.focused.clone())
        }
    }

    FindFocused { focused: None }
}

/// Deliberately avoids calling `state.focus()` on the kept widget —
/// iced snaps the caret to the end of the field on `focus()`.
pub(crate) fn unfocus_except(except: Id) -> impl Operation<()> {
    struct UnfocusExcept {
        except: Id,
    }

    impl Operation<()> for UnfocusExcept {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            let is_target = matches!(id, Some(id) if id == &self.except);
            if !is_target {
                state.unfocus();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<()> {
            Outcome::Some(())
        }
    }

    UnfocusExcept { except }
}
