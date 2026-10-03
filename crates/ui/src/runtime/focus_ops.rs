// iced 0.14 stacked focus needs two passes to preserve the clicked input and its caret.
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

/// Reports an explicit empty focus result because iced 0.14 otherwise drops the clear message.
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

/// iced 0.14 focus() moves the caret; preserve focus without refocusing the kept input.
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
