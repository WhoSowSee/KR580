use super::register_inline::RegisterMove;
use crate::i18n::Lang;
use crate::persistence::{ColorScheme, ShortcutAction, ShortcutBinding};
use iced::Point;
use iced::keyboard;
use iced::widget::text_editor;
use k580_core::RegisterName;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuId {
    File,
    Mp,
    View,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolWindowKind {
    Monitor,
    Floppy,
    Hdd,
    Network,
    Printer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpeedTier {
    Slow,
    Medium,
    High,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportTab {
    Xlsx,
    Text,
}

impl ExportTab {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Xlsx => "xlsx",
            Self::Text => "txt",
        }
    }

    pub(crate) fn default_file_name(self) -> &'static str {
        match self {
            Self::Xlsx => "kr580_export.xlsx",
            Self::Text => "kr580_export.txt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportMemoryColumn {
    Address,
    Value,
    Command,
    Comment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportRegister {
    Accumulator,
    W,
    Z,
    B,
    C,
    D,
    E,
    H,
    L,
    StackPointer,
    ProgramCounter,
    Cycles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportFlag {
    Sign,
    Zero,
    AuxiliaryCarry,
    Parity,
    Carry,
}

pub(crate) use super::help::{HelpNode, HelpSearchResponse};
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

    pub(crate) fn for_register(register: RegisterName) -> Self {
        match register {
            RegisterName::A => Self::Schematic(register),
            _ => Self::Mux(register),
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
    LoadSnapshotFromPath(PathBuf),
    SaveSnapshot,
    SaveSnapshotAs,
    NewFile,
    Export,
    ExportTabSelected(ExportTab),
    ToggleExportMemoryColumn(ExportMemoryColumn),
    ToggleExportRegister(ExportRegister),
    ToggleExportFlag(ExportFlag),
    ExportMemoryStartChanged(String),
    ExportMemoryEndChanged(String),
    ExportTargetChanged(String),
    ExportTargetDropdownToggled,
    ExportTargetSelected(String),
    ExportTargetAdd,
    ExportTargetDelete,
    ConfirmExport,
    CancelExport,
    Import,
    ImportFileBrowse,
    ImportTargetDropdownToggled,
    ImportTargetSelected(String),
    ImportTargetScrolled,
    ConfirmImport,
    CancelImport,
    RegisterNameChanged(String),
    RegisterPrevious,
    RegisterNext,
    RegisterValueChanged(String),
    ApplyRegister,
    RegisterSelected(RegisterInlineTarget),
    RegisterEnter(RegisterInlineTarget),
    RegisterReplace(RegisterInlineTarget),
    InlineRegisterValueChanged(RegisterInlineTarget, String),
    ApplyInlineRegisterValue(RegisterInlineTarget),
    RegisterHoverStarted(RegisterInlineTarget),
    RegisterHoverEnded(RegisterInlineTarget),
    MemorySelected(u16),
    MemoryEnter(u16),
    MemoryReplace(u16),
    /// Re-focus the inline value editor after the surrounding row was rebuilt.
    RefocusInline,
    MemoryAddressPrevious,
    MemoryAddressNext,
    MemoryAddressPageUp,
    MemoryAddressPageDown,
    /// `+1` for Up, `-1` for Down.
    ArrowKey(i32),
    HorizontalArrowKey(i32),
    RegisterArrowKey(RegisterMove),
    MemoryScrolled(f32, f32),
    JumpMemoryAddress,
    JumpMemoryTo(u16),
    MemoryAddressChanged(String),
    MemoryValueChanged(String),
    InlineMemoryValueChanged(u16, String),
    ApplyInlineMemoryValue(u16),
    PasteMemoryBytesRequested,
    MemoryBytesPasted(Option<String>),
    OpcodeDropdownToggled(u16),
    OpcodeSearchChanged(String),
    OpcodeSelected(u16, u8),
    OpcodeScrolled,
    HideOpcodeDropdown,
    DismissErrorNotice,
    DismissHaltNotice,
    ClearHalt,
    ToggleHalt,
    EscPressed,
    EnterPressed,
    MemoryCellAction,
    MemoryCellReturn,
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
    RuntimeEvent {
        event: iced::Event,
        status: iced::event::Status,
        window: iced::window::Id,
    },
    MousePressed,
    MousePressedIgnored,
    FocusReconciled {
        generation: u64,
        hit: Option<iced::widget::Id>,
    },
    ResolveFocusedTracker(Option<iced::widget::Id>),
    WindowOpened(iced::window::Id),
    WindowClosed(iced::window::Id),
    WindowResized {
        id: iced::window::Id,
        size: iced::Size,
    },
    FrameRendered,
    MenuCategoriesToggled,
    MenuToggled(MenuId),
    MenuClosed,
    /// Used by menu items to close the dropdown before dispatching their action.
    MenuBatch(Vec<Message>),
    SpeedTierChanged(SpeedTier),
    WindowDragStart,
    ToolWindowDragStart(ToolWindowKind),
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowMaximizedChanged(bool),
    Undo,
    Redo,
    ConfirmDiscard,
    CancelDiscard,
    /// OS-side close (× / Alt+F4); routed through the dirty gate.
    WindowCloseRequested(iced::window::Id),
    OpenSettings,
    CloseSettings,
    SaveSettings,
    OpenAbout,
    CloseAbout,
    OpenHelp,
    CloseHelp,
    HelpNodeSelected(HelpNode),
    HelpNodeToggled(HelpNode),
    HelpSearchChanged(String),
    HelpSearchFinished(HelpSearchResponse),
    HelpTextAction(text_editor::Action),
    HelpToggleExpandAll,
    OpenUrl(&'static str),
    OpenMonitor,
    CloseMonitor,
    DetachToolWindow(ToolWindowKind),
    AttachToolWindow(ToolWindowKind),
    ToggleToolWindowAlwaysOnTop(ToolWindowKind),
    ToggleMonitorSplit,
    ToggleMonitorHexPopup,
    CycleMonitorHexFilter,
    MonitorHexScrolled,
    ClearMonitorBuffer,
    SaveMonitorImage,
    OpenFloppy,
    CloseFloppy,
    ToggleFloppyImageContents,
    OpenFloppyImage,
    DetachFloppyImage,
    SaveFloppyBuffer,
    ToggleFloppyDebugBuffer,
    ClearFloppyBuffer,
    OpenHdd,
    CloseHdd,
    ClearHddBuffer,
    ChooseHddDirectory,
    ToggleHddDebugBuffer,
    DeleteHddFile,
    CreateHddFile,
    ToggleHddImageContents,
    OpenNetwork,
    CloseNetwork,
    OpenNetworkSettings,
    CloseNetworkSettings,
    NetworkModeChanged(crate::backend::NetworkMode),
    NetworkHostChanged(String),
    NetworkPortChanged(String),
    ApplyNetworkSettings,
    ClearNetworkBuffers,
    ToggleNetworkBufferView,
    OpenPrinter,
    ClosePrinter,
    TogglePrinterBufferView,
    ClearPrinterBuffer,
    PrintPrinterPdf,
    ToggleStackView,
    SettingsCategorySelected(SettingsCategory),
    SettingsSearchChanged(String),
    SettingsDraftLanguageChanged(Lang),
    SettingsDraftSpeedChanged(SpeedTier),
    SettingsDraftFollowPcSet(bool),
    SettingsDraftMemoryOperandHighlightingSet(bool),
    SettingsDraftColorSchemeChanged(ColorScheme),
    SettingsFloppyImageBrowse,
    SettingsDraftFloppyImageSet(PathBuf),
    SettingsFloppyImageClear,
    SettingsHddDirectoryBrowse,
    SettingsDraftHddDirectorySet(PathBuf),
    SettingsNetworkClientHostChanged(String),
    SettingsNetworkClientPortChanged(String),
    SettingsNetworkServerHostChanged(String),
    SettingsNetworkServerPortChanged(String),
    SettingsLanguageDropdownToggled,
    SettingsShortcutCaptureStarted(ShortcutAction),
    SettingsShortcutCaptured(ShortcutBinding),
    SettingsShortcutCaptureCancelled,
    SettingsShortcutsReset,
    SettingsResetRequested,
    SettingsResetConfirmed,
    SettingsResetCancelled,
    PersistSettings,
    SettingsSectionCycle {
        backward: bool,
    },
    SettingsFileAssociationRegister,
    SettingsFileAssociationUnregister,
}
