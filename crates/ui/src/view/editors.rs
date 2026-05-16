//! Right-hand side panel: memory list, memory cell editor, register editor.
//!
//! Each editor is a small framed panel with a spinner-driven left input,
//! a plain right input, and an enter button. They share the spinner shell
//! built in `widgets::spinner_text_input`.

use iced::widget::{column, container, row, text_input};
use iced::{Element, Length, alignment};

use super::styles::input_style;
use super::theme::MONO_FONT;
use super::widgets::{enter_button, legend_panel, spinner_text_input};
use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message, REGISTER_NAME_INPUT_ID,
    REGISTER_VALUE_INPUT_ID,
};

impl DesktopApp {
    pub(super) fn side_panel(&self) -> Element<'_, Message> {
        column![
            self.memory_panel(),
            self.memory_editor_panel(),
            self.register_editor_panel(),
        ]
        .spacing(8)
        .width(Length::Fixed(330.0))
        .height(Length::Fill)
        .into()
    }

    fn memory_editor_panel(&self) -> Element<'_, Message> {
        let controls = row![
            spinner_text_input(
                "0000",
                &self.memory_address_input,
                Message::MemoryAddressChanged,
                Message::MemoryAddressNext,
                Message::MemoryAddressPrevious,
                Length::Fixed(96.0),
                Message::JumpMemoryAddress,
                MEMORY_ADDRESS_INPUT_ID,
                self.focused_input == Some(MEMORY_ADDRESS_INPUT_ID),
            ),
            text_input("00", &self.memory_value_input)
                .id(MEMORY_VALUE_INPUT_ID)
                .on_input(Message::MemoryValueChanged)
                .on_submit(Message::ApplyMemory)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(58.0))
                .style(input_style),
            enter_button(Message::ApplyMemory),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(controls)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel("Ячейка ОЗУ и ее значение", content, Length::Shrink)
    }

    fn register_editor_panel(&self) -> Element<'_, Message> {
        let editor = row![
            spinner_text_input(
                "A",
                &self.register_name_input,
                Message::RegisterNameChanged,
                Message::RegisterNext,
                Message::RegisterPrevious,
                Length::Fixed(62.0),
                Message::ApplyRegister,
                REGISTER_NAME_INPUT_ID,
                self.focused_input == Some(REGISTER_NAME_INPUT_ID),
            ),
            text_input("00", &self.register_value_input)
                .id(REGISTER_VALUE_INPUT_ID)
                .on_input(Message::RegisterValueChanged)
                .on_submit(Message::ApplyRegister)
                .font(MONO_FONT)
                .size(16)
                .padding(6)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fixed(58.0))
                .style(input_style),
            enter_button(Message::ApplyRegister),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(editor)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel("Регистр и его значение", content, Length::Shrink)
    }
}
