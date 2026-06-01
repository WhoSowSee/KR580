use super::register_inline::RegisterMove;
use crate::i18n::Lang;
use iced::Point;
use iced::keyboard;
use k580_core::RegisterName;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuId {
    File,
    Mp,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpeedTier {
    Slow,
    Medium,
    High,
    Max,
}

pub(crate) use super::settings_modal::SettingsCategory;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegisterInlineTarget {
    Schematic(RegisterName),
    Mux(RegisterName),
}

impl RegisterInlineTarget {
    pub(crate) fn register(self) -> RegisterName {
        match self {
            Self::Schematic(register) | Self::Mux(register) => register,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Tick,
    StepInstruction,
    RestartProgram,
    StepTact,
    ToggleRun,
    ResetCpu,
    ResetRam,
    OpenSnapshot,
    /// Used at startup when the OS hands us `argv[1]`.
    LoadSnapshotFromPath(PathBuf),
    SaveSnapshot,
    SaveSnapshotAs,
    SaveLegacySnapshot,
    OpenLegacySnapshot,
    NewFile,
    Export,
    Import,
    RegisterNameChanged(String),
    RegisterPrevious,
    RegisterNext,
    RegisterValueChanged(String),
    ApplyRegister,
    RegisterSelected(RegisterInlineTarget),
    RegisterEnter(RegisterInlineTarget),
    RefocusInlineRegister,
    InlineRegisterValueChanged(RegisterInlineTarget, String),
    ApplyInlineRegisterValue(RegisterInlineTarget),
    RegisterHoverStarted(RegisterInlineTarget),
    RegisterHoverEnded(RegisterInlineTarget),
    MemorySelected(u16),
    MemoryEnter(u16),
    /// Re-focus the inline value editor after the surrounding row was rebuilt.
    RefocusInline,
    MemoryAddressPrevious,
    MemoryAddressNext,
    MemoryAddressPageUp,
    MemoryAddressPageDown,
    /// `+1` for Up, `-1` for Down.
    ArrowKey(i32),
    HorizontalArrowKey(i32),
    RegisterCtrlArrowKey(RegisterMove),
    MemoryScrolled(f32, f32),
    JumpMemoryAddress,
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    InlineMemoryValueChanged(u16, String),
    ApplyInlineMemoryValue(u16),
    OpcodeDropdownToggled(u16),
    OpcodeSearchChanged(String),
    OpcodeSelected(u16, u8),
    OpcodeScrolled,
    HideOpcodeDropdown,
    DismissErrorNotice,
    DismissHaltNotice,
    DismissInfoNotice,
    ClearHalt,
    ToggleHalt,
    EscPressed,
    EnterPressed,
    OpenOpcodePicker,
    ApplyMemory,
    ModifiersChanged(keyboard::Modifiers),
    FocusCycle {
        backward: bool,
    },
    FocusResolved {
        focused: iced::widget::Id,
        backward: bool,
    },
    CursorMoved(Point),
    MousePressed,
    FocusReconciled(Option<iced::widget::Id>),
    ResolveFocusedTracker(Option<iced::widget::Id>),
    WindowOpened(iced::window::Id),
    WindowResized(f32),
    FrameRendered,
    MenuCategoriesToggled,
    MenuToggled(MenuId),
    MenuClosed,
    /// Used by menu items to close the dropdown before dispatching their action.
    MenuBatch(Vec<Message>),
    SpeedTierChanged(SpeedTier),
    WindowDragStart,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowMaximizedChanged(bool),
    Undo,
    Redo,
    ConfirmDiscard,
    CancelDiscard,
    /// OS-side close (× / Alt+F4); routed through the dirty gate.
    WindowCloseRequested,
    OpenSettings,
    CloseSettings,
    SaveSettings,
    OpenAbout,
    CloseAbout,
    ShowHelpComingSoon,
    OpenUrl(&'static str),
    OpenMonitor,
    CloseMonitor,
    ToggleMonitorSplit,
    ToggleMonitorHexPopup,
    CycleMonitorHexFilter,
    MonitorHexScrolled,
    ClearMonitorBuffer,
    SaveMonitorImage,
    SettingsCategorySelected(SettingsCategory),
    SettingsSearchChanged(String),
    SettingsDraftLanguageChanged(Lang),
    SettingsDraftSpeedChanged(SpeedTier),
    SettingsLanguageDropdownToggled,
    SettingsResetRequested,
    SettingsResetConfirmed,
    SettingsResetCancelled,
    PersistSettings,
    SettingsSectionCycle {
        backward: bool,
    },
}
