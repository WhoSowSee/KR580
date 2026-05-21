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
//! - [`menu`]: top menu strip.
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

use iced::widget::{column, container, mouse_area, row};
use iced::{Element, Length};

use styles::app_style;

use crate::app::{DesktopApp, Message};

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

        if self.opcode_dropdown_address.is_some() {
            mouse_area(app_root)
                .on_press(Message::HideOpcodeDropdown)
                .into()
        } else {
            app_root
        }
    }
}
