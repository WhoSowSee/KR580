//! Custom widget operations that reconcile focus state from cursor
//! coordinates after every mouse press.
//!
//! Why we need this in iced 0.14.2:
//!
//! `text_input::update` (line 825) calls `shell.capture_event()` only on
//! the input that *actually contains* the click; the rest of the tree
//! still sees the same `ButtonPressed`. Each text_input that does NOT
//! contain the cursor sets its own `state.is_focused = None` at line
//! 725 — so within a flat container (a `row`, a `column`, …) iced
//! converges to one focused input on its own.
//!
//! Convergence breaks across stacked panels. Every editor in the side
//! panel is wrapped by `legend_panel`, which renders the title label
//! as `stack(framed_panel, legend)` so the legend cuts the border.
//! `stack::update` (stack.rs:262) iterates its layers in reverse (top
//! first) and bails out as soon as `shell.is_event_captured()` returns
//! true. The outer `column` (`column::update`, column.rs:272)
//! propagates events to every child unconditionally. The bad sequence:
//!
//! 1. User types in panel A's input (e.g. register_value). It now
//!    holds `state.is_focused = Some(_)`.
//! 2. User clicks panel B's input (e.g. memory_address). The column
//!    hands the event to panel A's stack first; A's framed_panel
//!    processes the click — A's text_input under the cursor (if any)
//!    captures, others clear themselves. The event continues
//!    captured-or-not to panel B's stack.
//! 3. Panel B's stack iterates `[framed_panel, legend]` in reverse:
//!    `legend` first. Legend doesn't capture, but the stack's
//!    post-update check sees the event is **still captured** (set by
//!    A's framed_panel a step earlier). The stack returns. **B's
//!    framed_panel never gets the event.**
//! 4. memory_address.is_focused is whatever it was before the click —
//!    typically `Some(_)` from earlier typing, since B's text_inputs
//!    were not visited. register_value.is_focused = Some too. We end
//!    up with two focused widgets, two carets, and the user sees the
//!    one they just clicked appear to "instantly reset" away.
//!
//! The fix is split across **two** operations on purpose:
//!
//! * [`find_focusable_at`] is a read-only scan that returns the id of
//!   the focusable whose bounds contain `point`, or `None` if no
//!   focusable claims the point.
//! * [`unfocus_except`] then walks the tree mutating state, clearing
//!   `is_focused` on everyone *except* the id we just identified.
//!
//! Why not do both in a single pass? Because the message handler that
//! dispatches the operation runs **after** `text_input::update` has
//! already processed the click in the freshly-clicked input. By then
//! the layout has been recomputed for the next frame, and the bounds
//! visited by the operation can drift by a pixel or two — enough that
//! a click in an already-focused input no longer falls inside its own
//! reported bounds. A single-pass operation that unconditionally
//! unfocuses every "miss" would then unfocus the very input that just
//! processed the click, dropping focus mid-edit.
//!
//! Splitting the work lets the handler treat "no focusable claims the
//! point" as a benign signal: the user clicked in dead space *or* the
//! layout drifted, and either way the safe action is to leave focus
//! alone. Only when we positively identify a hit do we issue the
//! unfocus pass — at which point clearing every *other* focusable is
//! unambiguously correct, because the hit acts as the anchor.
//!
//! `text_input::draw` (line 501) renders the caret straight off
//! `state.is_focused`, so a cosmetic shell-driven blue ring would
//! mask but not fix the underlying multi-focus state. The only
//! reliable fix is to clear stale state atomically — which is what
//! these operations do.

use iced::Point;
use iced::Rectangle;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Focusable, Operation, Outcome};

/// Builds an [`Operation`] that walks every focusable widget and
/// returns the id of the one whose bounds contain `point`, **without
/// mutating any state**. Returns `None` if no focusable claims the
/// point — either because the click landed in dead space (a panel
/// border, a label, the empty area between widgets) or because the
/// layout has drifted between the press event and the operation drain
/// (see the module-level note about why we split the reconciliation
/// in two).
///
/// Always paired with [`unfocus_except`] in the `MousePressed`
/// handler: the read-only scan first identifies who owns the click,
/// then a second pass clears everyone else. The handler treats a
/// `None` result as "leave focus alone" rather than "clear
/// everything", which is what makes repeated clicks inside the same
/// already-focused input safe — even if a layout race makes its
/// bounds momentarily fail to contain the click, no other operation
/// fires to wipe its `is_focused`.
pub(crate) fn find_focusable_at(point: Point) -> impl Operation<Option<Id>> {
    struct FindFocusableAt {
        point: Point,
        hit: Option<Id>,
    }

    impl Operation<Option<Id>> for FindFocusableAt {
        fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, _state: &mut dyn Focusable) {
            // Read-only: we never call `state.unfocus()` /
            // `state.focus()` here. Multiple focusables claiming the
            // point is impossible in practice (text_inputs do not
            // overlap), but if it ever happened the *last* one
            // visited would win — fine, since at that point any
            // single one is a defensible answer.
            if id.is_some() && bounds.contains(self.point) {
                self.hit = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            // Walk the entire tree. Pruning subtrees on bounds
            // mismatch would be a valid optimisation, but the focus
            // count is small (5 tracked inputs) so the simpler
            // unfiltered walk is fine.
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            // `Outcome::Some(None)` means "scan completed, no
            // focusable claimed the point". The handler reads this
            // and skips the follow-up unfocus pass.
            Outcome::Some(self.hit.clone())
        }
    }

    FindFocusableAt { point, hit: None }
}

/// Builds an [`Operation`] that walks every focusable widget and
/// returns the id of whichever one currently has `is_focused() ==
/// true`, or `None` if none of them do.
///
/// Differs from `iced::advanced::widget::operation::focusable::
/// find_focused` in one critical way: that built-in version produces
/// `Outcome::None` when nothing is focused, which causes the
/// `Task::map` continuation to silently drop the message. We need
/// the message to arrive *especially* when nothing is focused —
/// that's the entire signal we use to clear the cosmetic shell-border
/// tracker after Esc or a dead-space click.
///
/// Wrapping the answer in `Option<Id>` and always returning
/// `Outcome::Some(option)` makes the report unconditional, so the
/// `ResolveFocusedTracker` handler runs in both the "nothing focused"
/// and "something focused" cases and can branch on the value.
pub(crate) fn find_focused_optional() -> impl Operation<Option<Id>> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Option<Id>> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            // Last-writer-wins is fine: only one focusable should
            // ever report `is_focused() == true` at a time, and if
            // any layout race produces a transient pair, picking
            // either one for the cosmetic indicator is defensible.
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            // Always reportable: `Outcome::Some(None)` means "scan
            // completed, no focusable was focused".
            Outcome::Some(self.focused.clone())
        }
    }

    FindFocused { focused: None }
}

/// Builds an [`Operation`] that walks every focusable widget and
/// calls `state.unfocus()` on those whose id does **not** match
/// `except`. The widget identified by `except` is left alone —
/// `text_input::update` has already set `is_focused = Some(_)` on it
/// (or it was already focused from a prior interaction), and we
/// deliberately avoid calling `state.focus()` because iced's
/// implementation snaps the caret to the end of the value, which
/// would make repeated clicks inside the same input lose the user's
/// caret position.
///
/// Always preceded by [`find_focusable_at`] in the `MousePressed`
/// handler — see the module docs for why the work is split into two
/// passes.
pub(crate) fn unfocus_except(except: Id) -> impl Operation<()> {
    struct UnfocusExcept {
        except: Id,
    }

    impl Operation<()> for UnfocusExcept {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            // Match strictly on id. Bounds are intentionally ignored
            // here: by the time this operation runs we have *already*
            // committed to a hit, and any layout drift since the
            // initial scan is no longer relevant — we trust the id
            // we were handed.
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
