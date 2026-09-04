use iced::keyboard;
use iced::keyboard::key::{Code, Physical};

use super::constants::{MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_VALUE_INPUT_ID};
use super::messages::Message;
use super::state::DesktopApp;
use crate::i18n::{Key, Lang};
use crate::persistence::{
    ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutModifiers, ShortcutSettings,
};

pub(crate) fn binding_from_event(
    physical_key: Physical,
    modifiers: keyboard::Modifiers,
) -> Option<ShortcutBinding> {
    Some(ShortcutBinding {
        modifiers: ShortcutModifiers {
            ctrl: modifiers.command(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
        },
        key: shortcut_key_from_physical(physical_key)?,
    })
}

#[cfg(test)]
pub(crate) fn shortcut_message(
    settings: &ShortcutSettings,
    physical_key: Physical,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    shortcut_message_for_context(settings, physical_key, modifiers, ShortcutContext::General)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShortcutContext {
    General,
    MemoryEditor,
    MemoryCell,
}

pub(crate) fn shortcut_context(app: &DesktopApp) -> ShortcutContext {
    match app.focused_input {
        Some(MEMORY_ADDRESS_INPUT_ID | MEMORY_VALUE_INPUT_ID) => ShortcutContext::MemoryEditor,
        Some(MEMORY_INLINE_INPUT_ID) => ShortcutContext::MemoryCell,
        None if app.selected_memory_action_address().is_some() => ShortcutContext::MemoryCell,
        _ => ShortcutContext::General,
    }
}

pub(crate) fn shortcut_message_for_context(
    settings: &ShortcutSettings,
    physical_key: Physical,
    modifiers: keyboard::Modifiers,
    context: ShortcutContext,
) -> Option<Message> {
    let binding = binding_from_event(physical_key, modifiers)?;
    ShortcutAction::ALL
        .into_iter()
        .filter(|action| context.allows(*action))
        .find(|action| settings.matches(*action, binding))
        .map(message_for_action)
}

impl ShortcutContext {
    pub(crate) fn scoped_action(self) -> Option<ShortcutAction> {
        match self {
            Self::MemoryEditor => Some(ShortcutAction::MemoryPatternSearch),
            Self::MemoryCell => Some(ShortcutAction::MemoryCellReplace),
            Self::General => None,
        }
    }

    pub(crate) fn allows(self, action: ShortcutAction) -> bool {
        match action {
            ShortcutAction::MemoryPatternSearch | ShortcutAction::MemoryCellReplace => {
                self.scoped_action() == Some(action)
            }
            _ => true,
        }
    }
}

pub(crate) fn shortcut_label(settings: &ShortcutSettings, message: &Message) -> Option<String> {
    let action = action_for_message(message)?;
    settings.binding(action).map(ShortcutBinding::label)
}

pub(crate) fn shortcut_action_label(action: ShortcutAction, lang: Lang) -> &'static str {
    match action {
        ShortcutAction::NewFile => lang.t(Key::FileNew),
        ShortcutAction::OpenSnapshot => lang.t(Key::FileOpen),
        ShortcutAction::SaveSnapshot => lang.t(Key::FileSave),
        ShortcutAction::SaveSnapshotAs => lang.t(Key::FileSaveAs),
        ShortcutAction::Import => lang.t(Key::FileImport),
        ShortcutAction::Export => lang.t(Key::FileExport),
        ShortcutAction::ToggleRun => lang.t(Key::MpRunProgram),
        ShortcutAction::StepInstruction => lang.t(Key::MpRunInstruction),
        ShortcutAction::StepTact => lang.t(Key::MpRunTact),
        ShortcutAction::ResetRam => lang.t(Key::MpResetRam),
        ShortcutAction::ResetCpu => lang.t(Key::MpResetCpu),
        ShortcutAction::ClearHalt => lang.t(Key::MpClearHalt),
        ShortcutAction::OpenHelp => lang.t(Key::HelpShowDocs),
        ShortcutAction::OpenSettings => lang.t(Key::SettingsTitle),
        ShortcutAction::OpenMonitor => lang.t(Key::DeviceMonitor),
        ShortcutAction::OpenFloppy => lang.t(Key::DeviceFloppy),
        ShortcutAction::OpenHdd => lang.t(Key::DeviceHdd),
        ShortcutAction::OpenNetwork => lang.t(Key::DeviceNetwork),
        ShortcutAction::OpenPrinter => lang.t(Key::DevicePrinter),
        ShortcutAction::ToggleStackView => lang.t(Key::ViewStackArea),
        ShortcutAction::Undo => match lang {
            Lang::Ru => "Отменить",
            Lang::En => "Undo",
        },
        ShortcutAction::Redo => match lang {
            Lang::Ru => "Вернуть",
            Lang::En => "Redo",
        },
        ShortcutAction::OpenOpcodePicker => match lang {
            Lang::Ru => "Выбор команды",
            Lang::En => "Opcode picker",
        },
        ShortcutAction::MemoryPatternSearch => match lang {
            Lang::Ru => "Переход по шаблону адреса ОЗУ",
            Lang::En => "Find memory address pattern",
        },
        ShortcutAction::MemoryCellReplace => match lang {
            Lang::Ru => "Замена значения ячейки ОЗУ",
            Lang::En => "Replace memory cell value",
        },
        ShortcutAction::MemoryCellAction => match lang {
            Lang::Ru => "Действие с ячейкой ОЗУ",
            Lang::En => "Memory cell action",
        },
        ShortcutAction::MemoryCellReturn => match lang {
            Lang::Ru => "Вернуться к ячейке операнда ОЗУ",
            Lang::En => "Return to memory operand cell",
        },
        ShortcutAction::JumpMemoryStart => match lang {
            Lang::Ru => "Перейти к 0000",
            Lang::En => "Jump to 0000",
        },
        ShortcutAction::JumpMemoryEnd => match lang {
            Lang::Ru => "Перейти к FFFF",
            Lang::En => "Jump to FFFF",
        },
    }
}

pub(crate) fn shortcut_search_matches(lang: Lang, lower_query: &str) -> bool {
    ShortcutAction::ALL.into_iter().any(|action| {
        shortcut_action_label(action, lang)
            .to_lowercase()
            .contains(lower_query)
    })
}

pub(crate) fn message_for_action(action: ShortcutAction) -> Message {
    match action {
        ShortcutAction::NewFile => Message::NewFile,
        ShortcutAction::OpenSnapshot => Message::OpenSnapshot,
        ShortcutAction::SaveSnapshot => Message::SaveSnapshot,
        ShortcutAction::SaveSnapshotAs => Message::SaveSnapshotAs,
        ShortcutAction::Import => Message::Import,
        ShortcutAction::Export => Message::Export,
        ShortcutAction::ToggleRun => Message::ToggleRun,
        ShortcutAction::StepInstruction => Message::StepInstruction,
        ShortcutAction::StepTact => Message::StepTact,
        ShortcutAction::ResetRam => Message::ResetRam,
        ShortcutAction::ResetCpu => Message::ResetCpu,
        ShortcutAction::ClearHalt => Message::ClearHalt,
        ShortcutAction::OpenHelp => Message::OpenHelp,
        ShortcutAction::OpenSettings => Message::OpenSettings,
        ShortcutAction::OpenMonitor => Message::OpenMonitor,
        ShortcutAction::OpenFloppy => Message::OpenFloppy,
        ShortcutAction::OpenHdd => Message::OpenHdd,
        ShortcutAction::OpenNetwork => Message::OpenNetwork,
        ShortcutAction::OpenPrinter => Message::OpenPrinter,
        ShortcutAction::ToggleStackView => Message::ToggleStackView,
        ShortcutAction::Undo => Message::Undo,
        ShortcutAction::Redo => Message::Redo,
        ShortcutAction::OpenOpcodePicker => Message::OpenOpcodePicker,
        ShortcutAction::MemoryPatternSearch => Message::MemoryPatternSearch,
        ShortcutAction::MemoryCellReplace => Message::MemoryCellReplace,
        ShortcutAction::MemoryCellAction => Message::MemoryCellAction,
        ShortcutAction::MemoryCellReturn => Message::MemoryCellReturn,
        ShortcutAction::JumpMemoryStart => Message::JumpMemoryTo(0x0000),
        ShortcutAction::JumpMemoryEnd => Message::JumpMemoryTo(0xFFFF),
    }
}

fn action_for_message(message: &Message) -> Option<ShortcutAction> {
    match message {
        Message::NewFile => Some(ShortcutAction::NewFile),
        Message::OpenSnapshot => Some(ShortcutAction::OpenSnapshot),
        Message::SaveSnapshot => Some(ShortcutAction::SaveSnapshot),
        Message::SaveSnapshotAs => Some(ShortcutAction::SaveSnapshotAs),
        Message::Import => Some(ShortcutAction::Import),
        Message::Export => Some(ShortcutAction::Export),
        Message::ToggleRun => Some(ShortcutAction::ToggleRun),
        Message::StepInstruction => Some(ShortcutAction::StepInstruction),
        Message::StepTact => Some(ShortcutAction::StepTact),
        Message::ResetRam => Some(ShortcutAction::ResetRam),
        Message::ResetCpu => Some(ShortcutAction::ResetCpu),
        Message::ClearHalt => Some(ShortcutAction::ClearHalt),
        Message::OpenHelp => Some(ShortcutAction::OpenHelp),
        Message::OpenSettings => Some(ShortcutAction::OpenSettings),
        Message::OpenMonitor => Some(ShortcutAction::OpenMonitor),
        Message::OpenFloppy => Some(ShortcutAction::OpenFloppy),
        Message::OpenHdd => Some(ShortcutAction::OpenHdd),
        Message::OpenNetwork => Some(ShortcutAction::OpenNetwork),
        Message::OpenPrinter => Some(ShortcutAction::OpenPrinter),
        Message::ToggleStackView => Some(ShortcutAction::ToggleStackView),
        Message::Undo => Some(ShortcutAction::Undo),
        Message::Redo => Some(ShortcutAction::Redo),
        Message::OpenOpcodePicker => Some(ShortcutAction::OpenOpcodePicker),
        Message::MemoryPatternSearch => Some(ShortcutAction::MemoryPatternSearch),
        Message::MemoryCellReplace => Some(ShortcutAction::MemoryCellReplace),
        Message::MemoryCellAction => Some(ShortcutAction::MemoryCellAction),
        Message::MemoryCellReturn => Some(ShortcutAction::MemoryCellReturn),
        Message::JumpMemoryTo(0x0000) => Some(ShortcutAction::JumpMemoryStart),
        Message::JumpMemoryTo(0xFFFF) => Some(ShortcutAction::JumpMemoryEnd),
        _ => None,
    }
}

fn shortcut_key_from_physical(physical_key: Physical) -> Option<ShortcutKey> {
    let Physical::Code(code) = physical_key else {
        return None;
    };
    Some(match code {
        Code::KeyA => ShortcutKey::A,
        Code::KeyB => ShortcutKey::B,
        Code::KeyC => ShortcutKey::C,
        Code::KeyD => ShortcutKey::D,
        Code::KeyE => ShortcutKey::E,
        Code::KeyF => ShortcutKey::F,
        Code::KeyG => ShortcutKey::G,
        Code::KeyH => ShortcutKey::H,
        Code::KeyI => ShortcutKey::I,
        Code::KeyJ => ShortcutKey::J,
        Code::KeyK => ShortcutKey::K,
        Code::KeyL => ShortcutKey::L,
        Code::KeyM => ShortcutKey::M,
        Code::KeyN => ShortcutKey::N,
        Code::KeyO => ShortcutKey::O,
        Code::KeyP => ShortcutKey::P,
        Code::KeyQ => ShortcutKey::Q,
        Code::KeyR => ShortcutKey::R,
        Code::KeyS => ShortcutKey::S,
        Code::KeyT => ShortcutKey::T,
        Code::KeyU => ShortcutKey::U,
        Code::KeyV => ShortcutKey::V,
        Code::KeyW => ShortcutKey::W,
        Code::KeyX => ShortcutKey::X,
        Code::KeyY => ShortcutKey::Y,
        Code::KeyZ => ShortcutKey::Z,
        Code::Digit0 => ShortcutKey::Digit0,
        Code::Digit1 => ShortcutKey::Digit1,
        Code::Digit2 => ShortcutKey::Digit2,
        Code::Digit3 => ShortcutKey::Digit3,
        Code::Digit4 => ShortcutKey::Digit4,
        Code::Digit5 => ShortcutKey::Digit5,
        Code::Digit6 => ShortcutKey::Digit6,
        Code::Digit7 => ShortcutKey::Digit7,
        Code::Digit8 => ShortcutKey::Digit8,
        Code::Digit9 => ShortcutKey::Digit9,
        Code::Comma => ShortcutKey::Comma,
        Code::Period => ShortcutKey::Period,
        Code::Slash => ShortcutKey::Slash,
        Code::Semicolon => ShortcutKey::Semicolon,
        Code::Quote => ShortcutKey::Quote,
        Code::BracketLeft => ShortcutKey::BracketLeft,
        Code::BracketRight => ShortcutKey::BracketRight,
        Code::Backslash => ShortcutKey::Backslash,
        Code::Minus => ShortcutKey::Minus,
        Code::Equal => ShortcutKey::Equal,
        Code::Backquote => ShortcutKey::Backquote,
        Code::Enter | Code::NumpadEnter => ShortcutKey::Enter,
        _ => return None,
    })
}

#[cfg(test)]
#[path = "shortcuts/tests.rs"]
mod tests;
