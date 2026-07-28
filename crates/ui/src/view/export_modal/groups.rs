use iced::widget::{Space, column, container, mouse_area, row, stack};
use iced::{Element, Length, alignment};

use super::controls::{checkbox_row, flag_checkbox, group_box, input_shell, label, suffix};
use super::target::{target_dropdown_overlay, target_row_height, target_selector};
use crate::app::{
    ExportFlag, ExportFlagSelection, ExportMemoryColumn, ExportMemoryColumns, ExportModalFocus,
    ExportRegister, ExportRegisterSelection, ExportTab, Message,
};
use crate::i18n::{Key, Lang};

const HEX_INPUT_WIDTH: f32 = 78.0;
const REGISTER_GROUP_WIDTH: f32 = 220.0;

pub(super) struct MemoryGroupState<'a> {
    pub(super) tab: ExportTab,
    pub(super) focus: ExportModalFocus,
    pub(super) keyboard_focus_visible: bool,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) target_dropdown_open: bool,
    pub(super) target_highlight: Option<usize>,
    pub(super) memory_start: &'a str,
    pub(super) memory_end: &'a str,
    pub(super) columns: ExportMemoryColumns,
    pub(super) lang: Lang,
}

pub(super) fn memory_group<'a>(state: MemoryGroupState<'a>) -> Element<'a, Message> {
    let MemoryGroupState {
        tab,
        focus,
        keyboard_focus_visible,
        target_input,
        target_options,
        target_dropdown_open,
        target_highlight,
        memory_start,
        memory_end,
        columns,
        lang,
    } = state;

    let mut content = column![
        target_selector(
            tab,
            target_input,
            target_dropdown_open,
            focus,
            keyboard_focus_visible,
            lang,
        ),
        row![
            label(lang.t(Key::ExportRangeFrom)),
            input_shell(
                memory_start,
                HEX_INPUT_WIDTH,
                Message::ExportMemoryStartChanged,
                true,
                keyboard_focus_visible && focus == ExportModalFocus::MemoryStart,
            ),
            suffix("h"),
            label(lang.t(Key::ExportRangeTo)),
            input_shell(
                memory_end,
                HEX_INPUT_WIDTH,
                Message::ExportMemoryEndChanged,
                true,
                keyboard_focus_visible && focus == ExportModalFocus::MemoryEnd,
            ),
            suffix("h"),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center),
        Space::new().height(Length::Fixed(2.0)),
        checkbox_row(
            lang.t(Key::ExportColumnAddress),
            columns.address,
            Message::ToggleExportMemoryColumn(ExportMemoryColumn::Address),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_column(ExportMemoryColumn::Address),
        ),
        checkbox_row(
            lang.t(Key::ExportColumnValue),
            columns.value,
            Message::ToggleExportMemoryColumn(ExportMemoryColumn::Value),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_column(ExportMemoryColumn::Value),
        ),
        checkbox_row(
            lang.t(Key::ExportColumnCommand),
            columns.command,
            Message::ToggleExportMemoryColumn(ExportMemoryColumn::Command),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_column(ExportMemoryColumn::Command),
        ),
    ]
    .spacing(8)
    .width(Length::Fill);

    if tab == ExportTab::Xlsx {
        content = content.push(checkbox_row(
            lang.t(Key::ExportColumnComment),
            columns.comment,
            Message::ToggleExportMemoryColumn(ExportMemoryColumn::Comment),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_column(ExportMemoryColumn::Comment),
        ));
    }

    let content: Element<'a, Message> = if target_dropdown_open {
        let close_layer = column![
            Space::new().height(Length::Fixed(target_row_height())),
            mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message::ExportTargetDropdownToggled),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        stack![
            content,
            close_layer,
            target_dropdown_overlay(tab, target_options, target_highlight),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        content.into()
    };

    group_box(
        lang.t(Key::ExportMemoryGroup),
        content,
        Length::FillPortion(3),
    )
}

pub(super) fn register_group(
    registers: ExportRegisterSelection,
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let scratch = row![
        column![
            register_check(
                "W",
                registers.w,
                ExportRegister::W,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "B",
                registers.b,
                ExportRegister::B,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "D",
                registers.d,
                ExportRegister::D,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "H",
                registers.h,
                ExportRegister::H,
                focus,
                keyboard_focus_visible
            ),
        ]
        .spacing(5)
        .width(Length::Fill),
        column![
            register_check(
                "Z",
                registers.z,
                ExportRegister::Z,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "C",
                registers.c,
                ExportRegister::C,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "E",
                registers.e,
                ExportRegister::E,
                focus,
                keyboard_focus_visible
            ),
            register_check(
                "L",
                registers.l,
                ExportRegister::L,
                focus,
                keyboard_focus_visible
            ),
        ]
        .spacing(5)
        .width(Length::Fill),
    ]
    .spacing(12);

    let content = column![
        checkbox_row(
            lang.t(Key::ExportRegisterAccumulator),
            registers.accumulator,
            Message::ToggleExportRegister(ExportRegister::Accumulator),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_register(ExportRegister::Accumulator),
        ),
        scratch,
        Space::new().height(Length::Fixed(2.0)),
        checkbox_row(
            lang.t(Key::ExportRegisterStackPointer),
            registers.stack_pointer,
            Message::ToggleExportRegister(ExportRegister::StackPointer),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_register(ExportRegister::StackPointer),
        ),
        checkbox_row(
            lang.t(Key::ExportRegisterProgramCounter),
            registers.program_counter,
            Message::ToggleExportRegister(ExportRegister::ProgramCounter),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_register(ExportRegister::ProgramCounter),
        ),
        checkbox_row(
            lang.t(Key::ExportRegisterCycles),
            registers.cycles,
            Message::ToggleExportRegister(ExportRegister::Cycles),
            keyboard_focus_visible
                && focus == ExportModalFocus::for_register(ExportRegister::Cycles),
        ),
    ]
    .spacing(6)
    .width(Length::Fill);

    group_box(
        lang.t(Key::ExportRegistersGroup),
        content,
        Length::Fixed(REGISTER_GROUP_WIDTH),
    )
}

pub(super) fn flags_group(
    flags: ExportFlagSelection,
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let content = row![
        flag_check(
            "Z",
            flags.zero,
            ExportFlag::Zero,
            focus,
            keyboard_focus_visible,
        ),
        flag_check(
            "S",
            flags.sign,
            ExportFlag::Sign,
            focus,
            keyboard_focus_visible,
        ),
        flag_check(
            "P",
            flags.parity,
            ExportFlag::Parity,
            focus,
            keyboard_focus_visible,
        ),
        flag_check(
            "C",
            flags.carry,
            ExportFlag::Carry,
            focus,
            keyboard_focus_visible,
        ),
        flag_check(
            "AC",
            flags.auxiliary_carry,
            ExportFlag::AuxiliaryCarry,
            focus,
            keyboard_focus_visible,
        ),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill);

    group_box(lang.t(Key::ExportFlagsGroup), content, Length::Fill)
}

fn register_check(
    label_text: &'static str,
    checked: bool,
    register: ExportRegister,
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
) -> Element<'static, Message> {
    checkbox_row(
        label_text,
        checked,
        Message::ToggleExportRegister(register),
        keyboard_focus_visible && focus == ExportModalFocus::for_register(register),
    )
}

fn flag_check(
    label_text: &'static str,
    checked: bool,
    flag: ExportFlag,
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
) -> Element<'static, Message> {
    flag_checkbox(
        label_text,
        checked,
        Message::ToggleExportFlag(flag),
        keyboard_focus_visible && focus == ExportModalFocus::for_flag(flag),
    )
}
