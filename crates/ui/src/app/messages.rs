//! Message taxonomy for the iced application.
//!
//! Lives in its own module so the (large, frequently extended) enum does
//! not crowd the state container and update routing in `app/mod.rs`. New
//! messages should be added here together with a one-paragraph doc that
//! explains what user gesture maps to them.

use iced::keyboard;
use k580_core::RegisterName;

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Tick,
    StepInstruction,
    /// Reset the CPU registers/flags and re-run the program from
    /// `0x0000`. Bound to the second action-panel button while the run
    /// state is armed: the icon swaps from `step-forward` to
    /// `refresh-ccw`, mirroring the reference KR-580 emulator's "restart
    /// from the beginning" gesture. With the run state idle this message
    /// is never emitted; the same button reverts to `StepInstruction`.
    RestartProgram,
    StepTact,
    Run,
    Stop,
    /// Toggle the visual run/pause state of the action panel's leftmost
    /// button. Mirrors the reference KR-580 emulator: clicking the play
    /// glyph arms the "running" state (icon swaps to a red pause), and
    /// clicking again disarms it. The handler dispatches a real
    /// `AppCommand::Run` only when the byte at `cpu.pc` is non-zero,
    /// i.e. there is actually a program loaded at the current address —
    /// otherwise the press is purely cosmetic and no T-states are
    /// consumed.
    ToggleRun,
    ResetCpu,
    ResetRam,
    OpenSnapshot,
    SaveSnapshot,
    ExportTxt,
    ExportXlsx,
    ExportDocx,
    RegisterSelected(RegisterName),
    RegisterNameChanged(String),
    RegisterPrevious,
    RegisterNext,
    RegisterValueChanged(String),
    ApplyRegister,
    MemorySelected(u16),
    MemoryAddressPrevious,
    MemoryAddressNext,
    MemoryAddressPageUp,
    MemoryAddressPageDown,
    /// Raw ArrowUp/ArrowDown press from the global keyboard subscription.
    /// `direction` is `+1` for ArrowUp and `-1` for ArrowDown, matching the
    /// "up increments, down decrements" convention used by numeric byte
    /// fields. The handler dispatches to whichever editor currently owns
    /// focus, so the very same key changes a register byte in one place
    /// and scrolls the memory list in another, depending on context.
    ArrowKey(i32),
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
    ApplyMemory,
    /// Latest keyboard modifier state, broadcast by iced whenever any of the
    /// modifier keys change. Cached so message handlers can disambiguate
    /// modified shortcuts (Ctrl+Enter, Alt+Enter) before the text input's
    /// own `on_submit` fires.
    ModifiersChanged(keyboard::Modifiers),
    /// Move keyboard focus inside the focus group of the currently focused
    /// input. `backward` swaps direction (Shift+Tab). Groups are isolated:
    /// the memory address/value pair, the register name/value pair, and the
    /// inline memory list cycle independently.
    FocusCycle {
        backward: bool,
    },
    /// Internal continuation of `FocusCycle`: carries the id of the widget
    /// that owned focus when Tab was pressed. We compute the destination in
    /// the `update` handler because only there can we tweak app state (e.g.
    /// shift the inline-edited address) before issuing the actual focus
    /// task.
    FocusResolved {
        focused: iced::widget::Id,
        backward: bool,
    },
    /// Result of the periodic `find_focused` poll. Carries the ids of any
    /// focused widgets iced reports — typically zero or one — so the UI can
    /// keep `DesktopApp::focused_input` in sync regardless of how the user
    /// reached the input (typing, Tab, mouse click).
    FocusPolled(Vec<iced::widget::Id>),
    /// Iced reports that a window has been opened. We respond by cloaking it
    /// via DWM on Windows so the launch flash never reaches the screen.
    WindowOpened(iced::window::Id),
    /// Iced has rendered a frame. After the second frame we know the wgpu
    /// surface is presenting our content, so we can safely uncloak.
    FrameRendered,
}
