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

        legend_panel("Ячейка ОЗУ и ее значение", content, Length::Shrink)
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

        legend_panel("Регистр и его значение", content, Length::Shrink)
    }

    /// Composes the bottom toolbar: two side-by-side framed panels that
    /// mirror the toolbar of the reference KR-580 emulator. The left
    /// panel ("Выполнение") groups the execution-flow buttons (run /
    /// step instruction / step tact); the right panel ("Сброс") groups
    /// the destructive memory-state buttons (reset RAM / reset
    /// registers). Splitting them into separate frames replaces the old
    /// `vertical_divider` strip — the gap between the two `legend_panel`
    /// frames now does the same visual job, and each group gets its own
    /// title so the user can tell at a glance what the buttons do
    /// before reading the per-button tooltip. Each button's accent
    /// colour tints the SVG glyph at rest; the surrounding chrome stays
    /// neutral and only the surface tone shifts on hover/press.
    fn actions_panel(&self) -> Element<'_, Message> {
        // Horizontal gap between icon chips inside each action panel.
        // Kept identical for "Выполнение" and "Сброс" so the rhythm of
        // the two strips matches; `legend_panel`'s 10 px inner padding
        // plus the centring `container` then makes the gap from the
        // edge chip to its frame's border read as the same distance in
        // both panels.
        const CHIP_SPACING: f32 = 14.0;

        // The two leftmost buttons are tumblers driven by `self.running`.
        //
        // 1. The first button toggles between a green play glyph
        //    ("armed for run") and a red pause glyph ("running"). The
        //    flip and the actual `AppCommand::Run` dispatch are both
        //    gated on the byte at `cpu.pc` inside
        //    `DesktopApp::toggle_run`: an empty memory page is a
        //    no-op (status-bar hint, neither icon swap nor T-states),
        //    so `self.running` is only ever `true` while the worker
        //    is genuinely chasing a real instruction stream.
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

        // While `run_blocked_after_halt` is armed (the user just hit
        // HLT, the worker reported the error, the 8-second halt notice
        // is up *or has already faded*), every chip that would pump
        // the CPU forward is rendered without an `on_press` so iced
        // greys it out and ignores clicks. Only the two reset chips
        // and the dedicated `Сбросить флаг HLT` menu entry remain
        // live, mirroring the user's contract: "пока не сброшу флаг
        // или регистры — кнопки запуска блокируются". `Пауза` while
        // running is technically not a "запуск", but `ToggleRun`
        // *cannot* fire when running is true *and* the latch is true:
        // the latch is cleared the moment `apply_snapshot` sees a
        // non-halted state, and the run loop never starts in the
        // first place if `cpu.halted` is true (the worker would have
        // bounced it). Wrapping `ToggleRun` in `None` therefore
        // costs nothing — the only path it would have served is
        // already unreachable.
        let blocked = self.run_blocked_after_halt;
        let gate = |msg: Message| if blocked { None } else { Some(msg) };

        let execution_strip = row![
            icon_action_button(run_icon, gate(Message::ToggleRun), run_accent, run_tooltip),
            icon_action_button(step_icon, gate(step_message), TOKYO_BLUE, step_tooltip),
            icon_action_button(
                icons::redo_dot(),
                gate(Message::StepTact),
                TOKYO_YELLOW,
                "Выполнить такт",
            ),
        ]
        .spacing(CHIP_SPACING)
        .align_y(alignment::Vertical::Center);

        let reset_strip = row![
            icon_action_button(
                icons::reset_ram(),
                Some(Message::ResetRam),
                TOKYO_RED,
                "Сброс ОЗУ",
            ),
            icon_action_button(
                icons::reset_registers(),
                Some(Message::ResetCpu),
                TOKYO_MAGENTA,
                "Сброс регистров",
            ),
        ]
        .spacing(CHIP_SPACING)
        .align_y(alignment::Vertical::Center);

        let execution_panel = legend_panel(
            "Выполнение",
            container(execution_strip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );
        let reset_panel = legend_panel(
            "Сброс",
            container(reset_strip)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
            Length::Shrink,
        );

        // Pin each frame's width so the empty space *inside* the
        // border around the chip strip is identical in both panels.
        // `FillPortion(3) / FillPortion(2)` would split the column
        // proportionally to the chip count — but that gives the
        // smaller "Сброс" group too much breathing room while
        // squeezing "Выполнение" almost flush against its border.
        //
        // The two strips have a fixed difference of 52 px in content
        // width (one extra chip + one extra `CHIP_SPACING` gap), so
        // hard-coding widths that differ by exactly 52 px guarantees
        // each centred strip leaves the same amount of slack on each
        // side, which reads as "the edge chip sits the same distance
        // from the frame edge in both panels".
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
