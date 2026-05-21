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

use styles::app_style;

use crate::app::{DesktopApp, Message};

/// Vertical offset of the floating menu dropdown from the top of the
/// app root. The menu bar is 34 px tall and the root container has 8 px
/// of padding around its edge, so 42 px puts the dropdown flush with
/// the bar's bottom border.
const MENU_DROPDOWN_TOP: f32 = 42.0;

/// Horizontal offset of the floating menu dropdown from the app's left
/// edge. Tuned so the dropdown sits just under the "Файл" label —
/// `8 px root padding` + `Эмулятор KR580VM80A` glyph width + `18 px
/// row spacing`. The exact pixel target is approximate (text metrics
/// vary with the OS font fallback), but the dropdown only needs to
/// land "near" the trigger, not dead-centre under it.
const MENU_DROPDOWN_LEFT: f32 = 175.0;

impl DesktopApp {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let main = row![self.schematic_panel(), self.side_panel()]
            .spacing(8)
            .height(Length::Fill);

        let content = column![self.menu_bar(), main]
            .padding(8)
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
            stack![app_root, menu_dropdown_overlay(dropdown)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            app_root
        };

        // One scrim covers both interactive overlays we have today
        // (opcode picker and menu dropdown). When either is open we
        // wrap the whole thing in a `mouse_area` whose press emits
        // the matching "close" message; clicks inside the dropdowns
        // themselves do not bubble up because the dropdowns sit
        // inside `opaque` wrappers that swallow pointer events.
        if self.opcode_dropdown_address.is_some() {
            mouse_area(app_with_menu)
                .on_press(Message::HideOpcodeDropdown)
                .into()
        } else if self.open_menu.is_some() {
            mouse_area(app_with_menu)
                .on_press(Message::MenuClosed)
                .into()
        } else {
            app_with_menu
        }
    }
}

/// Pads the dropdown into the corner under the "Файл" trigger using a
/// pair of `Space`s, then `opaque`-wraps it so the surrounding
/// scrim's `mouse_area` does not see clicks landing on the dropdown
/// itself.
fn menu_dropdown_overlay(dropdown: Element<'_, Message>) -> Element<'_, Message> {
    column![
        Space::new().height(Length::Fixed(MENU_DROPDOWN_TOP)),
        row![
            Space::new().width(Length::Fixed(MENU_DROPDOWN_LEFT)),
            opaque(dropdown),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
