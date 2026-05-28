//! Top-of-window menu strip plus the floating dropdown panels.
//!
//! The bar itself is a flat `row` of clickable labels; "Файл" and
//! "МП-Система" each toggle a dropdown panel that the root `view`
//! overlays on top of the rest of the UI. iced 0.14 has no first-class
//! menu widget, so we lean on `mouse_area` + `button` and a tiny bit of
//! state in `DesktopApp` (`open_menu`) to drive visibility. Every
//! dropdown entry is a flat action — submenus would just add a second
//! click on top of the format selection that the OS file picker already
//! handles, and on the МП-Система side every entry is a single
//! emulator command anyway.
//!
//! The bar doubles as the custom title bar: the empty zone between the
//! menu triggers and the caption buttons is wrapped in a `mouse_area`
//! that fires `Message::WindowDragStart`, handing the press off to the
//! OS so the borderless window can be dragged like the native chrome.
//! The minimise / maximise / close buttons sit on the far right and
//! dispatch their respective `Window*` messages.

use iced::widget::{Space, button, column, container, mouse_area, row, svg};
use iced::{Element, Length, alignment};

use super::icons;
use super::menu_dropdowns::{
    FILE_DROPDOWN_WIDTH, MENU_ICON_SIZE, MP_DROPDOWN_WIDTH, file_dropdown, mp_dropdown,
};
use super::menu_labels::inactive_category_labels;
use super::styles::{
    caption_button_style, close_caption_button_style, menu_bar_divider_style, menu_bar_style,
};
use super::theme::{TOKYO_MAGENTA, TOKYO_TEXT, ui_text};
use crate::app::{DesktopApp, MenuId, Message};

/// Edge length of the SVG glyph rendered inside each caption button
/// (minimise / maximise / close). 14 px gives the same optical weight
/// as the native Windows caption at 100 % DPI without crowding the
/// 28 px button surface.
const CAPTION_ICON_SIZE: f32 = 14.0;

/// Edge length of the close-button glyph. The close cross is two
/// diagonal strokes, which carry less visual weight than the
/// horizontal stroke of the minimise glyph or the four-sided square
/// of the maximise glyph at the same nominal size — Lucide's `x`
/// looks noticeably smaller next to its siblings when all three are
/// painted at 14 px. Bumping the close glyph to 16 px restores the
/// optical balance so the three caption buttons read as the same
/// "weight" in the bar.
const CAPTION_CLOSE_ICON_SIZE: f32 = 16.0;

/// Outer width of every caption button. Native Windows captions sit at
/// 46 px on the title bar, but the menu bar here is only 34 px tall
/// and we want a square hit target — 28 px is the largest square that
/// still leaves a couple of pixels of breathing room above and below
/// the glyph inside the bar.
const CAPTION_BUTTON_WIDTH: f32 = 32.0;
const CAPTION_BUTTON_HEIGHT: f32 = 24.0;

impl DesktopApp {
    pub(super) fn menu_bar(&self) -> Element<'_, Message> {
        // The drag handle is a `mouse_area` filling all the empty
        // space between the menu triggers and the caption buttons.
        // `on_press` fires `WindowDragStart`, which dispatches the
        // OS-native drag loop. The handle itself draws nothing (it
        // wraps a `Space::with_width(Length::Fill)`), so the bar
        // visually reads as one contiguous surface even though the
        // middle band is interactive.
        let drag_handle: Element<'_, Message> = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::WindowDragStart)
        .into();

        let caption_buttons = row![
            caption_button(
                icons::window_minimize(),
                Message::WindowMinimize,
                CaptionKind::Neutral,
            ),
            caption_button(
                if self.window_maximized {
                    icons::window_restore()
                } else {
                    icons::window_maximize()
                },
                Message::WindowToggleMaximize,
                CaptionKind::Neutral,
            ),
            caption_button(
                icons::window_close(),
                Message::WindowCloseRequested,
                CaptionKind::Close
            ),
        ]
        .spacing(2)
        .align_y(alignment::Vertical::Center);

        let cpu_icon = svg(icons::cpu())
            .width(Length::Fixed(MENU_ICON_SIZE))
            .height(Length::Fixed(MENU_ICON_SIZE))
            .style(|_theme, _status| svg::Style {
                color: Some(TOKYO_TEXT),
            });
        // The cpu glyph doubles as the bar's "show / hide menu"
        // toggle. Wrapping it in a `mouse_area` (instead of a
        // `button`) keeps the visual chrome unchanged — no hover
        // tint, no surrounding surface — so the brand mark still
        // reads as part of the title bar rather than a third caption
        // button. `Pointer` interaction makes the cursor signal that
        // the icon is clickable. The press fires
        // `MenuCategoriesToggled`, which flips the visibility flag
        // and (on the hide half) collapses any open dropdown.
        let cpu_toggle: Element<'_, Message> = mouse_area(cpu_icon)
            .on_press(Message::MenuCategoriesToggled)
            .interaction(iced::mouse::Interaction::Pointer)
            .into();

        // The category strip (Файл / МП-Система / Вид / Настройки /
        // Справка) is built into a `Vec` so we can fold it out of the
        // bar entirely when the user has toggled it off. Always
        // present in the layout would be the wrong approach: leaving
        // an empty 18-px-spaced gap where the labels used to sit
        // would still consume bar real estate and read as "the menu
        // is broken" rather than "the menu is hidden". Building the
        // bar from a vector lets us drop those entries on demand
        // without disturbing the cpu icon, drag handle, or caption
        // buttons that bracket them.
        let mut bar_children: Vec<Element<'_, Message>> = Vec::with_capacity(8);
        bar_children.push(cpu_toggle);
        if self.menu_categories_visible {
            bar_children.push(menu_trigger(
                "Файл",
                MenuId::File,
                self.open_menu == Some(MenuId::File),
            ));
            bar_children.push(menu_trigger(
                "МП-Система",
                MenuId::Mp,
                self.open_menu == Some(MenuId::Mp),
            ));
            for label in inactive_category_labels() {
                bar_children.push(menu_label(label));
            }
        }
        bar_children.push(drag_handle);
        bar_children.push(caption_buttons.into());

        let bar = container(
            iced::widget::Row::with_children(bar_children)
                .spacing(18)
                .align_y(alignment::Vertical::Center),
        )
        // Asymmetric horizontal padding so the cpu brand mark on the
        // left sits the same distance from the window edge as the
        // close cross does on the right. The right-hand caption
        // buttons add an internal ~9 px between the button edge and
        // the 14 px glyph stroke on top of the 2 px container padding,
        // putting the rightmost stroke at ~11 px from the window edge.
        // The cpu glyph on the left has no surrounding button, so we
        // bake that ~9 px back into the container padding to keep the
        // two ends optically symmetric. Row spacing of 18 px then
        // takes care of the gap between cpu and "Файл" — it matches
        // the gap between every other top-level label.
        //
        // The 11/2 split is the result of the user asking for slightly
        // tighter title-bar margins than the original 17/8: shifted
        // both sides inward by 6 px, so the symmetric "≈11 px from
        // the window edge" rule still holds. `FILE_MENU_DROPDOWN_LEFT`
        // / `MP_MENU_DROPDOWN_LEFT` were lowered by the same 6 px in
        // `view/mod.rs` so the floating dropdowns continue to land
        // under their triggers — they are absolute X coordinates from
        // the window edge, so any change to this padding has to be
        // mirrored there or the dropdown drifts off-axis from the
        // label that opened it.
        .padding(iced::Padding::ZERO.left(11).right(2))
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(menu_bar_style);

        // The menu bar drops the rounded bubble border, but the user
        // still wants a visual seam between the title bar and the
        // schematic plate underneath. A 1-pixel hairline spanning the
        // full window width does that without bringing the bubble
        // back. The bar's fixed 34-px height keeps the vertical
        // breathing room around the labels symmetric — the hairline
        // sits *outside* that 34-px frame, so it doesn't eat into the
        // bottom half of the centered glyphs.
        //
        // While a top-level menu is open the hairline gets a hole
        // punched through it under the dropdown's footprint: a
        // floating panel overlays the bar from above, and its top
        // border would otherwise cross the line and draw a visible
        // double-stroke at their intersection (the user flagged
        // exactly this). We split the divider into [left segment]
        // [transparent gap matching the dropdown's x-range] [right
        // segment]. The gap is widened by `DIVIDER_GAP_BLEED` on
        // either side so micro-sub-pixel rendering shifts can't paint
        // a dot of hairline at the dropdown's left/right edges. Total
        // height stays 1 px so the layout offsets in `view::view()`
        // (which assume "bar + 1 px") stay valid.
        //
        // Coordinate-system note: the dropdown's `FILE_/MP_…_LEFT`
        // constants are absolute X from the window edge (the overlay
        // is `stack`ed *outside* the root padding). The divider lives
        // *inside* the root container, whose left padding is
        // `ROOT_PADDING_LEFT`. So in divider-local coordinates the
        // dropdown starts at `dropdown_left - ROOT_PADDING_LEFT`.
        // Without this subtraction the gap drifts left by exactly
        // the root padding, which is what the user saw.
        // The bleed is how far each line *extends past* the
        // dropdown's outer edge into the frame's footprint. Negative
        // values shrink the gap below `gap_width`, pushing the line
        // endpoints under the frame's 1 px border. The dropdown's
        // opaque fill paints over the overlap so the seam reads as
        // "line meets frame" without a visible sliver of plate.
        //
        // -6 px on each side comfortably overshoots the frame's 1 px
        // stroke plus the sub-pixel layout rounding iced applies when
        // it splits the row between `Fixed` and `Fill` segments. Lower
        // magnitudes (-1, -2, -4) still left a hairline gap on the
        // user's display, suggesting the right `Fill` segment ends up
        // a few pixels further out than the math predicts.
        const DIVIDER_GAP_BLEED: f32 = -6.0;
        const ROOT_PADDING_LEFT: f32 = 8.0;
        let divider: Element<'_, Message> = match self.open_menu {
            None => container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(menu_bar_divider_style)
                .into(),
            Some(menu) => {
                let (dropdown_left, gap_width) = match menu {
                    MenuId::File => (super::FILE_MENU_DROPDOWN_LEFT, FILE_DROPDOWN_WIDTH),
                    MenuId::Mp => (super::MP_MENU_DROPDOWN_LEFT, MP_DROPDOWN_WIDTH),
                };
                let gap_local_left = (dropdown_left - ROOT_PADDING_LEFT).max(0.0);
                let left_segment_width = (gap_local_left - DIVIDER_GAP_BLEED).max(0.0);
                let gap_total_width = gap_width + DIVIDER_GAP_BLEED * 2.0;
                let line_segment = |w: Length| {
                    container(Space::new())
                        .width(w)
                        .height(Length::Fixed(1.0))
                        .style(menu_bar_divider_style)
                };
                row![
                    line_segment(Length::Fixed(left_segment_width)),
                    Space::new().width(Length::Fixed(gap_total_width)),
                    line_segment(Length::Fill),
                ]
                .height(Length::Fixed(1.0))
                .into()
            }
        };
        column![bar, divider].into()
    }

    /// Builds the floating dropdown for whichever top-level menu is
    /// currently open, or `None` when the menu bar is at rest.
    /// Composed as a `column` so the root `view` can stack it on top
    /// of the rest of the UI at a fixed offset (just below the menu
    /// bar) without disturbing layout.
    pub(super) fn menu_dropdown(&self) -> Option<Element<'_, Message>> {
        match self.open_menu? {
            MenuId::File => Some(file_dropdown()),
            // `mp_dropdown` принимает `halted`, чтобы знать, рисовать
            // ли пункт «Сбросить флаг HLT» активным или серым:
            // когда halt-флипфлоп уже выключен, сбрасывать нечего,
            // и пункт читается как недоступный (см. `menu_item` с
            // `enabled=false` ниже).
            MenuId::Mp => Some(mp_dropdown(self.snapshot.cpu.halted)),
        }
    }
}

/// A non-clickable label for the menus we have not wired up yet (Вид,
/// Настройки, Справка). They keep their place in the bar so the visual
/// rhythm matches the reference emulator's chrome, even though clicking
/// them is currently a no-op.
fn menu_label(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 13, TOKYO_TEXT).into()
}

/// A clickable top-level menu label. The label itself is wrapped in a
/// `mouse_area` with `Pointer` interaction so the cursor signals
/// affordance, and the press dispatches `MenuToggled` for the menu we
/// own. When the menu is open the label tints to magenta so the user
/// can see at a glance which dropdown is currently visible — mirroring
/// the convention used by every native menu bar.
fn menu_trigger(label: &'static str, menu: MenuId, active: bool) -> Element<'static, Message> {
    let color = if active { TOKYO_MAGENTA } else { TOKYO_TEXT };
    mouse_area(ui_text(label, 13, color))
        .on_press(Message::MenuToggled(menu))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

/// Picks which of the two caption-button styles to use. `Neutral` is
/// the calm chrome used by the minimise / maximise glyphs; `Close`
/// flares red on hover to mirror the destructive affordance every
/// native window manager paints on the close box.
#[derive(Clone, Copy)]
enum CaptionKind {
    Neutral,
    Close,
}

/// Builds one caption button (minimise / maximise / close) for the
/// custom title bar. The body is a centered SVG glyph painted in the
/// regular text colour — the surrounding button style provides the
/// hover/press surface tint, and `CaptionKind` decides whether that
/// tint flares red (close) or stays neutral (the other two). `kind`
/// also picks the glyph size: the diagonal `x` of the close button
/// gets `CAPTION_CLOSE_ICON_SIZE` instead of `CAPTION_ICON_SIZE` so
/// it carries the same optical weight as the horizontal/square
/// strokes of its neighbours. The button has no border and a fixed
/// size so the three glyphs line up regardless of which one is
/// currently rendered for the toggle.
fn caption_button(
    icon: svg::Handle,
    action: Message,
    kind: CaptionKind,
) -> Element<'static, Message> {
    let glyph_size = match kind {
        CaptionKind::Neutral => CAPTION_ICON_SIZE,
        CaptionKind::Close => CAPTION_CLOSE_ICON_SIZE,
    };
    let glyph = svg(icon)
        .width(Length::Fixed(glyph_size))
        .height(Length::Fixed(glyph_size))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });

    let body = container(glyph)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    button(body)
        .on_press(action)
        .padding(0)
        .width(Length::Fixed(CAPTION_BUTTON_WIDTH))
        .height(Length::Fixed(CAPTION_BUTTON_HEIGHT))
        .style(move |_theme, status| match kind {
            CaptionKind::Neutral => caption_button_style(status),
            CaptionKind::Close => close_caption_button_style(status),
        })
        .into()
}
