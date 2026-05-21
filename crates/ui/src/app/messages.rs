//! Message taxonomy for the iced application.
//!
//! Lives in its own module so the (large, frequently extended) enum does
//! not crowd the state container and update routing in `app/mod.rs`. New
//! messages should be added here together with a one-paragraph doc that
//! explains what user gesture maps to them.

use iced::Point;
use iced::keyboard;
use k580_core::RegisterName;
use std::path::PathBuf;

/// Identifies a top-level dropdown in the menu bar. Only one menu may
/// be open at a time, and the bar's `view` decides whether to render
/// the floating panel by comparing this against `DesktopApp::open_menu`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuId {
    /// "Файл" — new / open / save / save-as / export-submenu.
    File,
    /// "МП-Система" — выполнить программу / команду / такт, очистить
    /// ОЗУ / регистры. Mirrors the run/step/reset gestures already
    /// available on the action panel so the user does not have to
    /// hunt for the icon glyphs to find them.
    Mp,
}

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
    /// Load a snapshot from a known path without opening a picker
    /// dialog. Emitted at startup when the user double-clicks a `.580`
    /// file in Explorer (the OS hands the path to us as `argv[1]`),
    /// and could be reused in the future for any "open this specific
    /// file" flow that bypasses the picker. The handler shares the
    /// post-load reconciliation with `OpenSnapshot` so the spinner
    /// and inline editor pick up the loaded PC.
    LoadSnapshotFromPath(PathBuf),
    SaveSnapshot,
    /// "Сохранить как" entry from the File dropdown. Currently behaves
    /// the same as `SaveSnapshot` because `rfd::FileDialog::save_file`
    /// already opens a "save as" picker every time — we don't yet
    /// remember a previously-used snapshot path. Wired as its own
    /// message so the menu maps cleanly to user-visible labels and
    /// future work can split the two without churning the menu code.
    SaveSnapshotAs,
    /// Wipe RAM and registers in one shot. Bound to "Новый файл" in
    /// the File dropdown — the gesture mirrors "discard current work
    /// and start with a blank slate", so we send both `ResetRam` and
    /// `ResetCpu` to the worker.
    NewFile,
    /// Open a single save dialog that accepts both TXT and XLSX
    /// (the two formats `Exporters` produce). The handler routes
    /// the chosen path to the matching `AppCommand::ExportTxt` /
    /// `AppCommand::ExportXlsx` based on the file extension picked
    /// in the OS file dialog. Wired to "Экспорт" in the File menu —
    /// the menu has no submenu for exports so the user picks the
    /// format once, in the place where the OS already lets them
    /// pick it (the file picker), instead of twice.
    Export,
    /// Open a single file picker that accepts both TXT and XLSX
    /// (the two formats `Exporters` produce). The handler routes the
    /// chosen path to the matching `AppCommand::ImportTxt` /
    /// `AppCommand::ImportXlsx` based on the file extension. Wired
    /// to "Импорт" in the File menu — the menu has no submenu for
    /// imports so the user does not pick the format twice
    /// (extension + dialog filter).
    Import,
    RegisterSelected(RegisterName),
    RegisterNameChanged(String),
    RegisterPrevious,
    RegisterNext,
    RegisterValueChanged(String),
    ApplyRegister,
    /// Single-click on a memory row: select the address (move the
    /// highlight onto it) but do **not** focus the inline value editor.
    /// The user has to either click the value cell directly or
    /// double-click the row to enter editing mode.
    MemorySelected(u16),
    /// Enter inline editing for `address`: select the row and put
    /// keyboard focus onto the inline value `text_input`. Emitted by
    /// double-click on the row, single-click on the value cell, and the
    /// programmatic "step memory address" path that wants the caret to
    /// follow the highlight (e.g. ArrowUp/ArrowDown while the inline
    /// editor was already focused).
    MemoryEnter(u16),
    /// Re-focus the inline value editor after the surrounding row has
    /// been re-built. Emitted by `handle_arrow_key` after stepping the
    /// memory address: stepping deselects the old row, which causes
    /// iced to drop the inline `text_input` from the tree, so chaining
    /// `operation::focus` to the same task hits a widget that no longer
    /// exists. Bouncing through a separate message defers the focus
    /// operation to the next frame, after iced has rebuilt the row at
    /// the new address with a fresh `text_input` carrying the same id.
    RefocusInline,
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
    /// Latest known cursor position, broadcast by iced on every
    /// `mouse::Event::CursorMoved`. Cached so the `MousePressed` handler
    /// can look up the click coordinates without having to dig them out
    /// of any individual widget — `mouse::Event::ButtonPressed` carries
    /// only the button identity, not the position.
    CursorMoved(Point),
    /// The user pressed the left mouse button somewhere in the window.
    /// The handler runs `find_focusable_at(self.latest_cursor_position)`
    /// to identify which focusable claims the click, and — only when
    /// it finds one — chains `unfocus_except(id)` to clear stale
    /// `is_focused` flags on every other focusable. This bypasses
    /// iced's per-widget propagation, which the column→stack capture
    /// race in `runtime/focus_ops.rs` makes unreliable across sibling
    /// panels. The two-pass split also keeps repeat clicks inside an
    /// already-focused input safe: when a layout race makes the click
    /// momentarily fall outside the input's reported bounds, no hit
    /// is found and no unfocus pass runs, so focus is preserved
    /// instead of being wiped mid-edit.
    MousePressed,
    /// Result of the `find_focusable_at` operation: the id of the
    /// focusable that the most recent click landed on, or `None` if
    /// the click missed every focusable (or hit an unkeyed one).
    /// Used to drive the cosmetic `focused_input` indicator
    /// authoritatively from cursor coordinates rather than relying on
    /// iced's natural focus-tracking, which the column→stack race
    /// breaks. The handler also chains the follow-up `unfocus_except`
    /// pass when a hit is found.
    FocusReconciled(Option<iced::widget::Id>),
    /// Iced reports that a window has been opened. We respond by cloaking it
    /// via DWM on Windows so the launch flash never reaches the screen.
    WindowOpened(iced::window::Id),
    /// Iced has rendered a frame. After the second frame we know the wgpu
    /// surface is presenting our content, so we can safely uncloak.
    FrameRendered,
    /// User clicked a top-level menu label in the menu bar. Toggles
    /// the corresponding dropdown: opens it if no menu was open, closes
    /// it if the same menu was already open, switches to the new menu
    /// otherwise.
    MenuToggled(MenuId),
    /// Close any currently-open menu. Emitted by the scrim-`mouse_area`
    /// that wraps the app while a dropdown is open (catches clicks in
    /// dead space) and by every actionable menu item just before it
    /// dispatches its real message, so the dropdown disappears before
    /// whatever the item triggers (a file dialog, an emulator command,
    /// …) takes over.
    MenuClosed,
    /// Run a list of messages in order on the next update tick. Used
    /// by menu items so a single press can close the dropdown *and*
    /// dispatch the action it represents — without this batch wrapper
    /// the dropdown would still be visible for a frame or two while
    /// the action's file dialog opened on top of it. The handler
    /// drains the vector via `Task::batch(Task::done(...))`, which
    /// preserves the original order.
    MenuBatch(Vec<Message>),
}
