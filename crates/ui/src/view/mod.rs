//! View layer for the desktop UI.

mod chips;
mod current_command;
mod cycles;
mod editors;
mod icons;
mod lamps;
mod memory_list;
mod menu;
mod menu_dropdowns;
mod menu_labels;
mod modal;
mod mux;
mod notices;
mod opcode_dropdown;
mod schematic;
mod speed;
mod status_register;
mod styles;
mod theme;
mod utils;
mod widgets;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use modal::discard_modal_overlay;
use notices::{error_notice_overlay, halt_notice_overlay, info_notice_overlay};
use styles::app_style;

use crate::app::{DesktopApp, MenuId, Message};

/// Vertical offset of the dropdown so its top border sits on the
/// menu bar's bottom hairline.
const MENU_DROPDOWN_TOP: f32 = 34.0;

/// Per-trigger horizontal offset. Tied to `.left(11)` padding in
/// `menu/menu_bar()`. Exposed so the bar's hairline can punch a hole
/// under the open dropdown.
pub(super) const FILE_MENU_DROPDOWN_LEFT: f32 = 39.0;
pub(super) const MP_MENU_DROPDOWN_LEFT: f32 = 93.0;

impl DesktopApp {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let main = row![self.schematic_panel(), self.side_panel()]
            .spacing(8)
            .height(Length::Fill);

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

        let app_with_menu: Element<'_, Message> = if let Some(dropdown) = self.menu_dropdown() {
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

        // Notice stacking order, bottom to top: halt → error → info.
        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.halt_notice.as_deref() {
                stack![app_with_menu, halt_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_menu
            };

        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.error_notice.as_deref() {
                stack![app_with_overlays, error_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_overlays
            };

        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.info_notice.as_deref() {
                stack![app_with_overlays, info_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_overlays
            };

        // One scrim covers both interactive overlays (opcode picker
        // and menu dropdown). Their dropdowns sit inside `opaque` so
        // inner clicks don't bubble up.
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

        if let Some(action) = self.pending_action.as_ref() {
            stack![
                scrimmed,
                discard_modal_overlay(action, self.discard_modal_focus)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            scrimmed
        }
    }
}

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
