//! Right-hand side panel: memory list, memory cell editor, register editor.
//!
//! Each editor is a small framed panel with a spinner-driven left input,
//! a plain right input, and an enter button. They share the spinner shell
//! built in `widgets::spinner_text_input`.

use iced::widget::{column, container, row, text_input};
use iced::{Element, Length, alignment};

use super::icons;
use super::styles::input_style;
use super::theme::{MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_RED, TOKYO_YELLOW};
use super::widgets::{
    enter_button, icon_action_button, legend_panel, spinner_text_input, vertical_divider,
};
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
            self.actions_panel(),
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

    /// Composes the "Управление" panel: a single horizontal strip of
    /// square SVG icon buttons that mirrors the toolbar of the reference
    /// KR-580 emulator. The execution group (run / step instruction /
    /// step tact) sits on the left, a thin vertical divider separates
    /// it from the memory-state group on the right (reset RAM / reset
    /// registers). Each button's accent colour tints the SVG glyph at
    /// rest and lights up the border on hover/press; the tooltip body
    /// explains what the glyph does so the icon-only layout stays
    /// discoverable.
    fn actions_panel(&self) -> Element<'_, Message> {
        // The two leftmost buttons are tumblers driven by `self.running`.
        //
        // 1. The first button toggles between a green play glyph
        //    ("armed for run") and a red pause glyph ("running"). The
        //    actual `AppCommand::Run` dispatch is gated on the byte at
        //    `cpu.pc` inside `DesktopApp::toggle_run`, so an empty memory
        //    page only swaps the icon without consuming any T-states.
        // 2. The second button is `step-forward` at rest and
        //    `refresh-ccw` while running. In the running state it sends
        //    `Message::RestartProgram`, which resets the CPU and re-runs
        //    from `0x0000` (memory is preserved). At rest it stays bound
        //    to `Message::StepInstruction` and the memory list follows
        //    the new PC after the step.
        let (run_icon, run_accent, run_tooltip) = if self.running {
            (icons::pause(), TOKYO_RED, "Пауза")
        } else {
            (icons::play(), TOKYO_GREEN, "Выполнить программу")
        };
        let (step_icon, step_message, step_tooltip) = if self.running {
            (
                icons::refresh_ccw(),
                Message::RestartProgram,
                "Перезапустить программу",
            )
        } else {
            (
                icons::step_forward(),
                Message::StepInstruction,
                "Выполнить команду",
            )
        };
        let strip = row![
            icon_action_button(run_icon, Message::ToggleRun, run_accent, run_tooltip),
            icon_action_button(step_icon, step_message, TOKYO_BLUE, step_tooltip),
            icon_action_button(
                icons::redo_dot(),
                Message::StepTact,
                TOKYO_YELLOW,
                "Выполнить такт",
            ),
            vertical_divider(),
            icon_action_button(
                icons::reset_ram(),
                Message::ResetRam,
                TOKYO_RED,
                "Сброс ОЗУ",
            ),
            icon_action_button(
                icons::reset_registers(),
                Message::ResetCpu,
                TOKYO_MAGENTA,
                "Сброс регистров",
            ),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

        let content = container(strip)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel("Управление", content, Length::Shrink)
    }
}
