//! Right-hand side panel: memory list, memory cell editor, register editor.
//!
//! Each editor is a small framed panel with a spinner-driven left input,
//! a plain right input, and an enter button. They share the spinner shell
//! built in `widgets::spinner_text_input`.

use iced::widget::{column, container, row, text_input};
use iced::{Element, Length, Padding, alignment};

use super::icons;
use super::styles::{input_borderless_style, input_shell_style};
use super::theme::{MONO_FONT, TOKYO_BLUE, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_RED, TOKYO_YELLOW};
use super::widgets::{enter_button, icon_action_button, legend_panel, spinner_text_input};
use crate::app::{
    DesktopApp, MEMORY_ADDRESS_INPUT_ID, MEMORY_VALUE_INPUT_ID, Message, REGISTER_NAME_INPUT_ID,
    REGISTER_VALUE_INPUT_ID,
};
use crate::i18n::Key;

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
        let value_focused = self.focused_input == Some(MEMORY_VALUE_INPUT_ID);
        let value_input: Element<'_, Message> = container(
            text_input("00", &self.memory_value_input)
                .id(MEMORY_VALUE_INPUT_ID)
                .on_input(Message::MemoryValueChanged)
                .on_submit(Message::ApplyMemory)
                .font(MONO_FONT)
                .size(16)
                .padding(Padding {
                    top: 6.0,
                    right: 0.0,
                    bottom: 6.0,
                    left: 0.0,
                })
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(input_borderless_style),
        )
        .width(Length::Fixed(58.0))
        .style(move |theme| input_shell_style(theme, value_focused))
        .into();

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
            value_input,
            enter_button(Message::ApplyMemory),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(controls)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel(self.lang.t(Key::MemoryEditorTitle), content, Length::Shrink)
    }

    fn register_editor_panel(&self) -> Element<'_, Message> {
        let value_focused = self.focused_input == Some(REGISTER_VALUE_INPUT_ID);
        let value_input: Element<'_, Message> = container(
            text_input("00", &self.register_value_input)
                .id(REGISTER_VALUE_INPUT_ID)
                .on_input(Message::RegisterValueChanged)
                .on_submit(Message::ApplyRegister)
                .font(MONO_FONT)
                .size(16)
                .padding(Padding {
                    top: 6.0,
                    right: 0.0,
                    bottom: 6.0,
                    left: 0.0,
                })
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .style(input_borderless_style),
        )
        .width(Length::Fixed(58.0))
        .style(move |theme| input_shell_style(theme, value_focused))
        .into();

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
            value_input,
            enter_button(Message::ApplyRegister),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

        let content = container(editor)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        legend_panel(
            self.lang.t(Key::RegisterEditorTitle),
            content,
            Length::Shrink,
        )
    }

    /// Bottom toolbar: two side-by-side framed panels mirroring the
    /// reference KR-580 emulator. The left panel groups execution
    /// controls (run / step instruction / step tact); the right
    /// groups destructive resets (RAM / registers). Each button's
    /// accent colour tints its glyph at rest; chrome stays neutral.
    fn actions_panel(&self) -> Element<'_, Message> {
        const CHIP_SPACING: f32 = 14.0;

        // Two leftmost buttons are tumblers driven by `self.running`:
        // run/pause is gated on `cpu.pc` inside `toggle_run`;
        // step/restart swaps `StepInstruction` ↔ `RestartProgram`
        // (ResetCpu + Run, RAM preserved).
        let (run_icon, run_accent, run_tooltip) = if self.running {
            (icons::pause(), TOKYO_RED, self.lang.t(Key::ActionPause))
        } else {
            (
                icons::play(),
                TOKYO_GREEN,
                self.lang.t(Key::ActionRunProgram),
            )
        };
        let (step_icon, step_message, step_tooltip) = if self.running {
            (
                icons::refresh_ccw(),
                Message::RestartProgram,
                self.lang.t(Key::ActionRestartProgram),
            )
        } else {
            (
                icons::step_forward(),
                Message::StepInstruction,
                self.lang.t(Key::ActionStepInstruction),
            )
        };

        // Post-HLT latch greys out every execution chip until reset.
        // `apply_snapshot` clears it on the first non-halted snapshot.
        let blocked = self.run_blocked_after_halt;
        let gate = |msg: Message| if blocked { None } else { Some(msg) };

        let execution_strip = row![
            icon_action_button(run_icon, gate(Message::ToggleRun), run_accent, run_tooltip),
            icon_action_button(step_icon, gate(step_message), TOKYO_BLUE, step_tooltip),
            icon_action_button(
                icons::redo_dot(),
                gate(Message::StepTact),
                TOKYO_YELLOW,
                self.lang.t(Key::ActionStepTact),
            ),
        ]
        .spacing(CHIP_SPACING)
        .align_y(alignment::Vertical::Center);

        let reset_strip = row![
            icon_action_button(
                icons::reset_ram(),
                Some(Message::ResetRam),
                TOKYO_RED,
                self.lang.t(Key::ActionResetRam),
            ),
            icon_action_button(
                icons::reset_registers(),
                Some(Message::ResetCpu),
                TOKYO_MAGENTA,
                self.lang.t(Key::ActionResetCpu),
            ),
        ]
        .spacing(CHIP_SPACING)
        .align_y(alignment::Vertical::Center);

        let execution_panel = legend_panel(
            self.lang.t(Key::ExecutionPanel),
            container(execution_strip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );
        let reset_panel = legend_panel(
            self.lang.t(Key::ResetPanel),
            container(reset_strip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );

        // Widths differ by exactly 52 px (one chip + one CHIP_SPACING
        // gap) so the centred strips leave equal slack on either side.
        const EXECUTION_PANEL_WIDTH: f32 = 186.0;
        const RESET_PANEL_WIDTH: f32 = 134.0;

        row![
            container(execution_panel).width(Length::Fixed(EXECUTION_PANEL_WIDTH)),
            container(reset_panel).width(Length::Fixed(RESET_PANEL_WIDTH)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center)
        .into()
    }
}
