//! View layer for the desktop UI.
//!
//! The module is sliced into focused submodules so that no single file
//! grows past a comfortable reading size. Each submodule owns one panel
//! or one concern:
//!
//! - [`theme`]: colour swatches, fonts, and the `ui_text` / `mono_text`
//!   helpers everyone reuses.
//! - [`styles`]: container/text-input/button/scrollable style functions.
//! - [`widgets`]: small reusable widgets (legend frame, spinner text
//!   input, ↵ button).
//! - [`utils`]: tiny helpers shared by more than one panel.
//! - [`menu`]: top menu strip and the "Файл" dropdown.
//! - [`schematic`]: left-hand simulated CPU schematic.
//! - [`memory_list`]: virtualised memory list with the inline value
//!   editor.
//! - [`opcode_dropdown`]: floating opcode picker that drops out of a
//!   memory row.
//! - [`editors`]: right-hand side panel with the memory cell editor and
//!   the register editor.
//!
//! All submodules attach their `impl DesktopApp { fn ... }` blocks to the
//! same `DesktopApp` defined in `crate::app`, which keeps panel logic
//! near the markup that produces it.

mod editors;
mod icons;
mod memory_list;
mod menu;
mod opcode_dropdown;
mod schematic;
mod styles;
mod theme;
mod utils;
mod widgets;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use styles::{app_style, inset_style};
use theme::{TOKYO_TEXT, ui_text};

use crate::app::{DesktopApp, MenuId, Message};

/// Vertical offset of the floating menu dropdown from the top of the
/// app root. The menu bar is 34 px tall and sits flush with the top of
/// the window (root padding is now `top: 0`), with a 1-px hairline
/// directly below it. 34 px puts the dropdown's top border *on top of*
/// the divider hairline rather than 1 px below it — without this
/// overlap a thin horizontal sliver of plate showed through between
/// the divider line and the dropdown's top edge, breaking the
/// "frame hangs off the line" illusion the user flagged.
const MENU_DROPDOWN_TOP: f32 = 34.0;

/// Vertical offset of the halt-blocked notice overlay from the top of
/// the app root. Sits comfortably below the menu bar (34 px tall + 1 px
/// hairline) with a small gap so the framed message reads as a separate
/// floating element rather than glued to the bar.
const HALT_NOTICE_TOP: f32 = 48.0;

/// Horizontal offset of the floating menu dropdown from the app's left
/// edge, **per top-level menu**. Each value puts the dropdown's left
/// edge a few pixels to the *left* of the trigger label so the row
/// labels inside (which carry their own `4 + 10 = 14 px` of inner
/// padding before the glyph) line up under the first letter of the
/// trigger. Composition: `8 px root padding` + `17 px bar padding` +
/// `16 px cpu glyph` + `18 px gap` − `14 px dropdown inner inset` =
/// `45 px` for "Файл". "МП-Система" then sits another `~36 px`
/// (label width) + `18 px` gap further along the bar. Numbers are
/// approximate — text metrics shift with the OS font fallback — and
/// only need to land "near" the trigger, not dead-centre under it.
///
/// Exposed to the `menu` submodule so the bar's bottom hairline can
/// punch a hole under the open dropdown — see `menu_bar()`.
pub(super) const FILE_MENU_DROPDOWN_LEFT: f32 = 45.0;
pub(super) const MP_MENU_DROPDOWN_LEFT: f32 = 99.0;

impl DesktopApp {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let main = row![self.schematic_panel(), self.side_panel()]
            .spacing(8)
            .height(Length::Fill);

        // Root padding is per-side rather than a single `padding(8)`:
        // the menu bar must lie flush with the top of the window so
        // the visible vertical breathing room above the labels equals
        // the room below them (i.e. labels stay optically centred in
        // the 34-px bar). Side and bottom paddings remain at 8 px so
        // the schematic + side panel keep the same gutters they had
        // before. The bar itself does not need a top hairline because
        // the title bar's own bottom hairline (drawn inside
        // `menu_bar()`) already separates it from the schematic.
        let content = column![self.menu_bar(), main]
            .padding(iced::Padding {
                top: 0.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            })
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        let app_root: Element<'_, Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(app_style)
            .into();

        // If a top-level menu is open, lay its dropdown panel over the
        // app root via `stack`. The dropdown is wrapped in `opaque` so
        // clicks inside it don't leak through to the scrim underneath
        // — that scrim is what closes the menu on stray clicks, and
        // catching the dropdown's own clicks would dismiss it before
        // the actual menu item could process the press.
        let app_with_menu: Element<'_, Message> = if let Some(dropdown) = self.menu_dropdown() {
            // Per-menu horizontal offset: each top-level label sits at
            // a different x in the menu bar, and the dropdown should
            // land under its own trigger rather than under the first
            // one. `open_menu` is `Some(_)` whenever `menu_dropdown`
            // returned a panel, so the unwrap path is unreachable.
            let left = match self.open_menu {
                Some(MenuId::File) => FILE_MENU_DROPDOWN_LEFT,
                Some(MenuId::Mp) => MP_MENU_DROPDOWN_LEFT,
                None => FILE_MENU_DROPDOWN_LEFT,
            };
            stack![app_root, menu_dropdown_overlay(dropdown, left)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            app_root
        };

        // The halt-blocked notice (Variant A) sits above everything
        // else: when a step/run gesture is refused on a halted CPU,
        // the user gets a framed message floating at the top centre
        // of the window explaining how to unblock themselves. The
        // overlay is non-interactive — it carries no buttons — so the
        // surrounding scrim's mouse_area still catches clicks
        // anywhere on screen and routes them to the matching close
        // message. We still wrap the framed text in `opaque` so
        // pointer events do not leak through it; the visual frame
        // would otherwise pretend to be clickable.
        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.halt_notice.as_deref() {
                stack![app_with_menu, halt_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_menu
            };

        // One scrim covers both interactive overlays we have today
        // (opcode picker and menu dropdown). When either is open we
        // wrap the whole thing in a `mouse_area` whose press emits
        // the matching "close" message; clicks inside the dropdowns
        // themselves do not bubble up because the dropdowns sit
        // inside `opaque` wrappers that swallow pointer events.
        if self.opcode_dropdown_address.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::HideOpcodeDropdown)
                .into()
        } else if self.open_menu.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::MenuClosed)
                .into()
        } else {
            app_with_overlays
        }
    }
}

/// Pads the dropdown into the corner under its trigger using a pair of
/// `Space`s, then `opaque`-wraps it so the surrounding scrim's
/// `mouse_area` does not see clicks landing on the dropdown itself.
/// `left` is the per-menu horizontal offset picked by `view()`.
fn menu_dropdown_overlay(dropdown: Element<'_, Message>, left: f32) -> Element<'_, Message> {
    column![
        Space::new().height(Length::Fixed(MENU_DROPDOWN_TOP)),
        row![
            Space::new().width(Length::Fixed(left)),
            opaque(dropdown),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Floating notice anchored to the top centre of the window. Used for
/// the halt-blocked Variant A message — see `docs/ui_app.md` and the
/// `halt_notice` field on `DesktopApp`. The framed body uses
/// `inset_style` so the message reads as a discrete UI element with a
/// border on the dark schematic background. `opaque` wraps the body so
/// pointer events do not leak through the visible frame, but the
/// notice has no on-press of its own — clicks just do nothing
/// (consistent with passive notifications).
fn halt_notice_overlay(notice: &str) -> Element<'_, Message> {
    let body = container(ui_text(notice.to_owned(), 13, TOKYO_TEXT))
        .padding([8, 16])
        .style(inset_style);
    column![
        Space::new().height(Length::Fixed(HALT_NOTICE_TOP)),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
