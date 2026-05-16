//! Top-of-window menu strip.
//!
//! All emulator-control actions are exposed as a flat row of clickable
//! labels. Real cascading menus would be nicer, but iced 0.14 has no
//! first-class menu widget yet, so we lean on small `button` widgets that
//! emit `Message` values directly.

use iced::widget::{container, row};
use iced::{Element, Length, alignment};

use super::styles::{menu_bar_style, menu_button_style};
use super::theme::{TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::{DesktopApp, Message};

impl DesktopApp {
    pub(super) fn menu_bar(&self) -> Element<'_, Message> {
        container(
            row![
                ui_text("Emulator KR580VM80A", 14, TOKYO_MAGENTA),
                menu_label("File"),
                menu_label("MP-System"),
                menu_label("View"),
                menu_label("Settings"),
                menu_label("Help"),
                menu_action("Open", Message::OpenSnapshot),
                menu_action("Save", Message::SaveSnapshot),
                menu_action("TXT", Message::ExportTxt),
                menu_action("XLSX", Message::ExportXlsx),
                menu_action("DOCX", Message::ExportDocx),
                menu_action("Step", Message::StepInstruction),
                menu_action("Tact", Message::StepTact),
                menu_action("Run", Message::Run),
                menu_action("Stop", Message::Stop),
                menu_action("Reset", Message::ResetCpu),
                menu_action("RAM", Message::ResetRam),
            ]
            .spacing(18)
            .align_y(alignment::Vertical::Center),
        )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(menu_bar_style)
        .into()
    }
}

fn menu_label(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 13, TOKYO_TEXT).into()
}

fn menu_action(label: &'static str, message: Message) -> Element<'static, Message> {
    iced::widget::button(ui_text(label, 12, TOKYO_MUTED))
        .on_press(message)
        .padding(4)
        .style(move |_theme, status| menu_button_style(status))
        .into()
}
