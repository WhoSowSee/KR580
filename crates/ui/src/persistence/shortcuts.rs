use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShortcutAction {
    NewFile,
    OpenSnapshot,
    SaveSnapshot,
    SaveSnapshotAs,
    Import,
    Export,
    ToggleRun,
    StepInstruction,
    StepTact,
    ResetRam,
    ResetCpu,
    ClearHalt,
    OpenHelp,
    OpenSettings,
    OpenMonitor,
    OpenFloppy,
    OpenHdd,
    OpenNetwork,
    OpenPrinter,
    ToggleStackView,
    Undo,
    Redo,
    OpenOpcodePicker,
    #[serde(alias = "memoryOperandAction")]
    MemoryCellAction,
    MemoryCellReturn,
    JumpMemoryStart,
    JumpMemoryEnd,
}

impl ShortcutAction {
    pub const ALL: [Self; 27] = [
        Self::NewFile,
        Self::OpenSnapshot,
        Self::SaveSnapshot,
        Self::SaveSnapshotAs,
        Self::Import,
        Self::Export,
        Self::ToggleRun,
        Self::StepInstruction,
        Self::StepTact,
        Self::ResetRam,
        Self::ResetCpu,
        Self::ClearHalt,
        Self::OpenHelp,
        Self::OpenSettings,
        Self::OpenMonitor,
        Self::OpenFloppy,
        Self::OpenHdd,
        Self::OpenNetwork,
        Self::OpenPrinter,
        Self::ToggleStackView,
        Self::Undo,
        Self::Redo,
        Self::OpenOpcodePicker,
        Self::MemoryCellAction,
        Self::MemoryCellReturn,
        Self::JumpMemoryStart,
        Self::JumpMemoryEnd,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShortcutKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Comma,
    Period,
    Slash,
    Semicolon,
    Quote,
    BracketLeft,
    BracketRight,
    Backslash,
    Minus,
    Equal,
    Backquote,
    Enter,
}

impl ShortcutKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::BracketLeft => "[",
            Self::BracketRight => "]",
            Self::Backslash => "\\",
            Self::Minus => "-",
            Self::Equal => "=",
            Self::Backquote => "`",
            Self::Enter => "Enter",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
    pub modifiers: ShortcutModifiers,
    pub key: ShortcutKey,
}

impl ShortcutBinding {
    pub const fn new(ctrl: bool, shift: bool, alt: bool, key: ShortcutKey) -> Self {
        Self {
            modifiers: ShortcutModifiers { ctrl, shift, alt },
            key,
        }
    }

    pub fn label(self) -> String {
        let mut parts = Vec::with_capacity(4);
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        parts.push(self.key.label());
        parts.join("+")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutOverride {
    pub action: ShortcutAction,
    pub binding: Option<ShortcutBinding>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettings {
    #[serde(default)]
    pub bindings: Vec<ShortcutOverride>,
}

impl ShortcutSettings {
    pub fn binding(&self, action: ShortcutAction) -> Option<ShortcutBinding> {
        self.bindings
            .iter()
            .rev()
            .find(|entry| entry.action == action)
            .map(|entry| entry.binding)
            .unwrap_or_else(|| default_binding(action))
    }

    pub fn assign(&mut self, action: ShortcutAction, binding: ShortcutBinding) {
        for other in ShortcutAction::ALL {
            if other != action && self.binding(other) == Some(binding) {
                self.set_raw(other, None);
            }
        }
        self.set_raw(action, Some(binding));
        self.normalize();
    }

    pub fn normalize(&mut self) {
        let mut normalized = Vec::new();
        for action in ShortcutAction::ALL {
            if let Some(binding) = self
                .bindings
                .iter()
                .rev()
                .find(|entry| entry.action == action)
                .and_then(|entry| entry.binding)
            {
                if Some(binding) != default_binding(action) {
                    normalized.push(ShortcutOverride {
                        action,
                        binding: Some(binding),
                    });
                }
            } else if self
                .bindings
                .iter()
                .rev()
                .any(|entry| entry.action == action && entry.binding.is_none())
            {
                normalized.push(ShortcutOverride {
                    action,
                    binding: None,
                });
            }
        }
        self.bindings = normalized;
    }

    fn set_raw(&mut self, action: ShortcutAction, binding: Option<ShortcutBinding>) {
        self.bindings.retain(|entry| entry.action != action);
        self.bindings.push(ShortcutOverride { action, binding });
    }
}

pub fn default_binding(action: ShortcutAction) -> Option<ShortcutBinding> {
    use ShortcutAction as Action;
    use ShortcutKey as Key;
    match action {
        Action::NewFile => Some(ShortcutBinding::new(true, false, false, Key::N)),
        Action::OpenSnapshot => Some(ShortcutBinding::new(true, false, false, Key::O)),
        Action::SaveSnapshot => Some(ShortcutBinding::new(true, false, false, Key::S)),
        Action::SaveSnapshotAs => Some(ShortcutBinding::new(true, true, false, Key::S)),
        Action::Import => Some(ShortcutBinding::new(true, false, false, Key::I)),
        Action::Export => Some(ShortcutBinding::new(true, false, false, Key::E)),
        Action::ToggleRun => Some(ShortcutBinding::new(true, false, false, Key::R)),
        Action::StepInstruction => Some(ShortcutBinding::new(true, false, false, Key::T)),
        Action::StepTact => Some(ShortcutBinding::new(true, false, false, Key::Y)),
        Action::ResetRam => Some(ShortcutBinding::new(true, true, false, Key::R)),
        Action::ResetCpu => Some(ShortcutBinding::new(true, true, false, Key::G)),
        Action::ClearHalt => Some(ShortcutBinding::new(true, true, false, Key::H)),
        Action::OpenHelp => Some(ShortcutBinding::new(true, false, false, Key::H)),
        Action::OpenSettings => Some(ShortcutBinding::new(true, false, false, Key::Comma)),
        Action::OpenMonitor => Some(ShortcutBinding::new(true, false, false, Key::M)),
        Action::OpenFloppy => Some(ShortcutBinding::new(true, false, false, Key::F)),
        Action::OpenHdd => Some(ShortcutBinding::new(true, false, false, Key::D)),
        Action::OpenNetwork => Some(ShortcutBinding::new(true, false, false, Key::A)),
        Action::OpenPrinter => Some(ShortcutBinding::new(true, false, false, Key::P)),
        Action::ToggleStackView => Some(ShortcutBinding::new(true, true, false, Key::C)),
        Action::Undo => Some(ShortcutBinding::new(true, false, false, Key::Z)),
        Action::Redo => Some(ShortcutBinding::new(true, true, false, Key::Z)),
        Action::OpenOpcodePicker => Some(ShortcutBinding::new(false, false, false, Key::E)),
        Action::JumpMemoryStart => Some(ShortcutBinding::new(false, false, true, Key::Q)),
        Action::MemoryCellAction => Some(ShortcutBinding::new(false, false, true, Key::Enter)),
        Action::MemoryCellReturn => Some(ShortcutBinding::new(false, true, true, Key::Enter)),
        Action::JumpMemoryEnd => Some(ShortcutBinding::new(false, false, true, Key::E)),
    }
}
