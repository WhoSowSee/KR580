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

use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Background, Border, Color, Element, Length};

use styles::{app_style, inset_style};
use theme::{TOKYO_BG, TOKYO_BORDER, TOKYO_TEXT, ui_text};

use crate::app::{DesktopApp, MenuId, Message, PendingAction};

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
        let scrimmed: Element<'_, Message> = if self.opcode_dropdown_address.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::HideOpcodeDropdown)
                .into()
        } else if self.open_menu.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::MenuClosed)
                .into()
        } else {
            app_with_overlays
        };

        // Confirmation modal sits above every other overlay: it is
        // the only chrome that explicitly disables the surrounding
        // UI while it is up. The backdrop is a full-window
        // semi-transparent fill wrapped in `opaque` so pointer
        // events cannot reach the menu bar, schematic, side panel,
        // or any of the input fields underneath — exactly what the
        // user asked for ("ничего не было кликабельным"). True
        // gaussian blur is not a primitive iced 0.14 exposes, so
        // the visual cue for "background suppressed" comes from the
        // dark semi-transparent overlay instead — the same pattern
        // every modal-using iced app reaches for.
        if let Some(action) = self.pending_action.as_ref() {
            stack![scrimmed, discard_modal_overlay(action)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            scrimmed
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

/// Renders the "unsaved changes" confirmation modal. The layout is
/// three layers stacked together:
///
/// 1. **Backdrop** — a full-window dark fill (`TOKYO_BOARD` at 70%
///    alpha) wrapped in `mouse_area` + `opaque`. The mouse_area
///    catches *every* click that misses the dialog so the user can
///    dismiss with Esc-style "click outside" without needing the
///    button; the opaque wrapper guarantees those clicks do not
///    pass through to the application underneath. Together they
///    deliver the "ничего не было кликабельным" requirement: any
///    click anywhere on the page either dismisses the modal or
///    activates a button on the modal itself, never anything
///    behind it.
/// 2. **Centred dialog** — a column with the title, the body
///    paragraph, and the two action buttons. Wrapped in a second
///    `opaque` so pointer events on the dialog do not bubble back
///    up to the backdrop's `mouse_area` (otherwise clicking inside
///    the dialog would dismiss it).
/// 3. **Spacer rows** above and below + `Space::with(Length::Fill)`
///    flanks on either side push the dialog to the geometric centre
///    of the window without needing absolute coordinates.
///
/// `action` is the queued gesture, used only for the dialog title so
/// the user sees which gesture they are confirming ("Открыть файл" /
/// "Новый файл" / "Импорт" / "Закрыть приложение"). The body
/// paragraph is the same for every variant — the unsaved-changes
/// warning carries the actionable information.
fn discard_modal_overlay(action: &PendingAction) -> Element<'_, Message> {
    let title = match action {
        PendingAction::OpenSnapshot => "Открыть файл",
        PendingAction::NewFile => "Новый файл",
        PendingAction::Import => "Импорт",
        PendingAction::CloseWindow => "Закрыть приложение",
    };

    // Backdrop: wraps the whole window in a darkened fill. The
    // `mouse_area` swallows clicks landing outside the dialog and
    // routes them to `CancelDiscard` — same gesture as clicking
    // "Отменить", so a click on dead space behaves the same way as
    // pressing the cancel button. `opaque` then prevents that
    // mouse_area from passing the event further down the tree.
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelDiscard);

    let cancel_button = button(
        container(ui_text("Отменить", 13, TOKYO_TEXT))
            .padding([6, 16]),
    )
    .on_press(Message::CancelDiscard)
    .style(modal_button_style);

    let confirm_button = button(
        container(ui_text("Закрыть", 13, TOKYO_TEXT))
            .padding([6, 16]),
    )
    .on_press(Message::ConfirmDiscard)
    .style(modal_button_style);

    let buttons = row![
        Space::new().width(Length::Fill),
        cancel_button,
        Space::new().width(Length::Fixed(8.0)),
        confirm_button,
    ]
    .width(Length::Fill);

    let body = container(
        column![
            ui_text(title.to_owned(), 16, TOKYO_TEXT),
            Space::new().height(Length::Fixed(8.0)),
            ui_text(
                "Несохранённые изменения будут потеряны.".to_owned(),
                13,
                TOKYO_TEXT,
            ),
            Space::new().height(Length::Fixed(16.0)),
            buttons,
        ]
        .width(Length::Fixed(360.0)),
    )
    .padding(16)
    .style(modal_dialog_style);

    // Centre the dialog. `Length::Fill` spacers above/below and on
    // both sides push the framed body to the middle of the
    // window — works for any window size without picking absolute
    // pixel coordinates.
    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // The backdrop sits underneath the centred dialog — both stacked
    // together so the dark fill spans the whole window while the
    // dialog only takes its content size.
    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Semi-transparent dark overlay for the modal backdrop. Iced 0.14
/// has no native gaussian blur primitive, so we approximate the
/// "blur the background" intent with a darkening fill — the standard
/// pattern modal dialogs use across desktop UI when blur is not
/// available. 70% alpha on `TOKYO_BOARD` lets just enough of the
/// schematic bleed through that the user remembers what they were
/// doing while still reading the surrounding chrome as suppressed.
fn modal_backdrop_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: 0.07,
            g: 0.07,
            b: 0.13,
            a: 0.70,
        })),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..iced::widget::container::Style::default()
    }
}

/// Framed body of the modal dialog. Solid surface (no transparency
/// — the backdrop already provides the contrast) with a 1-px border
/// so the dialog reads as a discrete element floating above the
/// suppressed background. 8 px corner radius matches the rest of
/// the bubble chrome.
fn modal_dialog_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BG)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

/// "Отменить" / "Закрыть" — both modal buttons share the same neutral
/// chrome. The user explicitly asked the destructive twin to look the
/// same as the cancel button: visual weight should not push them
/// toward either choice. Surface-tone fill, neutral border, subtle
/// hover/press feedback — same shape as the editor `↵` button.
fn modal_button_style(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use crate::view::theme::{TOKYO_SURFACE, TOKYO_SURFACE_2};
    use iced::widget::button;
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_2,
        button::Status::Hovered => TOKYO_SURFACE,
        _ => TOKYO_BG,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}
