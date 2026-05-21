//! Custom widget operation that reconciles focus state from cursor
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
//! `reconcile_focus_at(point)` fixes this at the source: a global
//! `event::listen_with` for `mouse::Event::ButtonPressed(Left)` fires
//! a message regardless of capture status; the handler runs this
//! operation, which authoritatively assigns focus to whatever
//! focusable bounds contain the point and clears every other
//! focusable. No race, no stale state, no flash, no dependency on
//! iced's per-widget propagation.
//!
//! `text_input::draw` (line 501) renders the caret straight off
//! `state.is_focused`, so a cosmetic shell-driven blue ring would
//! mask but not fix the underlying multi-focus state. The only
//! reliable fix is to clear stale state atomically — which is what
//! this operation does.

use iced::Point;
use iced::Rectangle;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Focusable, Operation, Outcome};

/// Builds an [`Operation`] that walks every focusable widget and calls
/// `state.unfocus()` on those whose bounds do **not** contain `point`,
/// leaving any focusable whose bounds **do** contain the point alone.
/// Returns the id of the focusable that contains the point (or `None`
/// if the click missed every focusable, or hit an unkeyed one).
///
/// This is the authoritative cleanup we run on every
/// `mouse::Event::ButtonPressed(Left)` from a global
/// `event::listen_with` subscription. The reason we need it instead
/// of trusting iced's own per-widget propagation is the column→stack
/// capture race described at the module level: when the user clicks
/// panel B after panel A captured an earlier press in the same batch,
/// panel B's `framed_panel` never sees the event because B's
/// `stack::update` returns early on `shell.is_event_captured()`. The
/// text_inputs inside panel B then keep whatever `is_focused` they
/// had before the click — typically a stale `Some(_)` — and the user
/// sees two carets or focus appears to jump back to a
/// previously-edited input.
///
/// Crucially, this operation does **not** call `state.focus()` on the
/// hit widget. iced's `text_input::State::focus` calls
/// `move_cursor_to_end()` (line 1520 of text_input.rs), which would
/// yank the caret to the end of the value on every click — unusable
/// for any field longer than one character. We rely on the fact that
/// `text_input::update` (line 725) has *already* set
/// `state.is_focused = Some(_)` on the hit widget by the time this
/// operation runs, since iced processes the click event before
/// draining the operation queue. Our job is purely to clear the
/// stale `Some(_)`s on widgets that the click did not land on.
pub(crate) fn reconcile_focus_at(point: Point) -> impl Operation<Option<Id>> {
    struct ReconcileFocusAt {
        point: Point,
        hit: Option<Id>,
    }

    impl Operation<Option<Id>> for ReconcileFocusAt {
        fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
            if bounds.contains(self.point) {
                // The click landed inside this focusable. Leave its
                // state alone — iced has already set `is_focused =
                // Some(_)` for us in `text_input::update`. Capture
                // the id so the caller can update its cosmetic
                // tracker. Unkeyed focusables (like the opcode-search
                // input that has no static id) cannot be tracked, so
                // we leave `hit` as `None` for them.
                if let Some(id) = id {
                    self.hit = Some(id.clone());
                }
            } else {
                // The click did not land inside this focusable. Clear
                // any stale `is_focused = Some(_)` it may still hold
                // from an earlier interaction. This is the exact same
                // state mutation iced *would* have performed in
                // `text_input::update` (line 725) if the column→stack
                // capture race had not aborted the traversal early.
                state.unfocus();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Id>>)) {
            // Walk the entire tree. Skipping any subtree (even one
            // that we are confident does not contain the cursor)
            // would let stale focus state survive in widgets we never
            // visited.
            operate(self);
        }

        fn finish(&self) -> Outcome<Option<Id>> {
            // Always emit `Some(...)` so the resulting `Task` resolves
            // to a deterministic value and the message handler always
            // gets to run. The inner `Option<Id>` is `None` when the
            // click missed every focusable (e.g. clicked on a panel
            // border or the empty space between widgets), which the
            // handler treats as "drop the cosmetic indicator".
            Outcome::Some(self.hit.clone())
        }
    }

    ReconcileFocusAt { point, hit: None }
}
