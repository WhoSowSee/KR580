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

/// Four-position speed switch in the schematic panel. Replaces the
/// freeform slider after the user observed that "above ~60 Hz nothing
/// visually changes anyway" — named tiers communicate intent honestly
/// instead of inviting the user to chase a sweet spot that doesn't
/// exist.
///
/// - `Slow` — 5 Hz, one instruction every 200 ms. Matches the pace of
///   "step through and read every line", the режим обучения.
/// - `Medium` — 20 Hz, the default. Visibly "the program is running"
///   while the eye still keeps up with each PC update.
/// - `High` — locked to the primary monitor's refresh rate (with a
///   60 Hz fallback when the OS query fails). Each frame becomes one
///   instruction; the program finishes as fast as the screen can
///   paint without skipping rows.
/// - `Max` — uncoupled from the monitor: ships `MIN_STEP_INTERVAL`
///   (1 ms) to the worker so it churns at ~1000 instructions/sec.
///   The UI subscription still ticks at ~60 Hz, so the highlighted
///   row visibly *jumps* across memory rather than walking — the
///   trade-off is explicit in the label "Максимум": "выполни как
///   можно быстрее, не показывай мне каждый шаг".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpeedTier {
    Slow,
    Medium,
    High,
    Max,
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
    /// clicking the pause glyph disarms it. Pause is unconditional —
    /// the handler dispatches `AppCommand::Stop` regardless of where
    /// PC has walked to. Run-arming is gated on the byte at `cpu.pc`
    /// being non-zero (and the CPU not halted): on an empty page the
    /// press is a status-bar no-op (`No program at <PC>`), so the
    /// visual flag never goes out of sync with the worker and a
    /// subsequent import or snapshot load cannot inherit a stale
    /// "armed" state.
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
    /// Raw Esc keypress from the global keyboard subscription. The
    /// listener cannot read app state (it's a `Fn` closure), so the
    /// router lives in `update`: with the inline memory editor
    /// focused, Esc reverts the pending edit; otherwise it closes
    /// the opcode dropdown — the previous Esc binding. Adding this
    /// thin layer keeps the Esc handler honest about which gesture
    /// it represents in any given moment.
    EscPressed,
    /// Raw Enter keypress from the global keyboard subscription,
    /// fired only when the press was reported with
    /// `Status::Ignored` — i.e. no focusable consumed it. The
    /// handler reads the currently selected memory address from
    /// `memory_address_input` and dispatches `MemoryEnter` so the
    /// inline value editor receives focus on that row. This is the
    /// keyboard-only counterpart to double-clicking the row, and
    /// the recovery path after Esc / a dead-space click cleared
    /// focus — without it the user has to reach for the mouse to
    /// resume editing. Inside any text input the press never lands
    /// here because `text_input::on_submit` captures it first.
    EnterPressed,
    /// Raw E keypress (no modifiers) from the global keyboard
    /// subscription, fired only when the press was reported with
    /// `Status::Ignored` — i.e. no text input owned the caret. The
    /// handler opens the floating opcode picker for the currently
    /// selected memory row and chains a focus task onto its search
    /// field, so the user can immediately start typing a hex byte
    /// or a mnemonic. This is the keyboard-only counterpart to
    /// clicking the command column: it recovers the picker from a
    /// no-focus state without forcing the user back to the mouse.
    /// Inside any text input the press is captured by the input
    /// itself and never reaches this branch.
    OpenOpcodePicker,
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
    /// Result of an `iced::find_focused()` poll fired after a gesture
    /// that may have left iced and the cosmetic `focused_input`
    /// tracker in disagreement: Esc (which iced consumes by clearing
    /// `state.is_focused` on the active text_input) and dead-space
    /// clicks (where iced clears the previous focus too, but
    /// `FocusReconciled(None)` deliberately leaves our tracker alone
    /// to absorb layout-race false negatives). When the poll returns
    /// `None` we know iced really has no focused widget, so the
    /// cosmetic shell border on the prior input is stale and gets
    /// cleared. A `Some` reply means a focusable still owns the
    /// caret — usually because of the same layout-race scenario the
    /// click reconciler exists to absorb — and we leave the tracker
    /// alone.
    ResolveFocusedTracker(Option<iced::widget::Id>),
    /// Iced reports that a window has been opened. We respond by cloaking it
    /// via DWM on Windows so the launch flash never reaches the screen.
    WindowOpened(iced::window::Id),
    /// Iced has rendered a frame. After the second frame we know the wgpu
    /// surface is presenting our content, so we can safely uncloak.
    FrameRendered,
    /// User clicked the cpu brand mark on the far left of the menu
    /// bar. Toggles `DesktopApp::menu_categories_visible`: visible
    /// labels (Файл / МП-Система / View / Settings / Help) collapse,
    /// hidden labels reappear. Also clears `open_menu` on the
    /// "hide" half of the toggle so a dropdown can't outlive its
    /// trigger label and end up floating over an empty bar with no
    /// visible anchor.
    MenuCategoriesToggled,
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
    /// Speed switch on the left-hand schematic panel: the user picked
    /// a new tier (Slow / Medium / High). The handler resolves it to
    /// a concrete Hz via `tier_hz()`, stashes both the tier and the
    /// resolved Hz on `DesktopApp`, and ships
    /// `AppCommand::SetStepInterval(1s/hz)` to the worker.
    SpeedTierChanged(SpeedTier),
    /// User pressed the left mouse button on the empty area of the
    /// custom title bar. The handler dispatches `iced::window::drag`
    /// for the cached `window_id`, which hands the press over to the
    /// OS so the user can drag the borderless window the same way
    /// they would the native caption. Emitted by the `mouse_area`
    /// wrapping the drag region in `view::titlebar`.
    WindowDragStart,
    /// User clicked the "—" caption button. Dispatches
    /// `iced::window::minimize(true)` for the cached window id.
    WindowMinimize,
    /// User clicked the "□"/"❐" caption button. Dispatches
    /// `iced::window::toggle_maximize` and chains an `is_maximized`
    /// poll so `DesktopApp::window_maximized` (and therefore the
    /// glyph on the button) catches up with the new OS-side state.
    WindowToggleMaximize,
    /// User clicked the "×" caption button. Dispatches
    /// `iced::window::close` for the cached window id, ending the
    /// app the same way clicking the native close button would.
    WindowClose,
    /// Result of an `is_maximized` poll fired after
    /// `WindowToggleMaximize` and after every `WindowOpened`. Cached
    /// on `DesktopApp::window_maximized` so the maximise/restore
    /// glyph matches the actual OS state — without the poll the
    /// button would always show the "maximise" square even after the
    /// window was already filling the screen.
    WindowMaximizedChanged(bool),
    /// Ctrl+Z — undo the most recent edit. Pops the top entry from
    /// the shared undo stack: a `Text` entry restores the matching
    /// input field's previous string; a `Cpu` entry replays the
    /// pre-mutation `Cpu8080State` through `AppCommand::ApplyCpuState`,
    /// which the worker treats as a `Stopped` + state-replace pair so
    /// the run/halt indicators come back into agreement with the
    /// rewound state. The popped entry is moved onto the redo stack
    /// so Ctrl+Shift+Z can replay it.
    Undo,
    /// Ctrl+Shift+Z — redo the most recently undone edit. Mirror of
    /// `Undo`: pops the top entry from the redo stack, applies its
    /// `after` half (text or CPU), and pushes the entry back onto
    /// the undo stack.
    Redo,
    /// User clicked "Закрыть" in the unsaved-changes confirmation
    /// modal. The handler reads `pending_action`, clears it, and
    /// runs the queued action (open file / new file / import /
    /// close window) without re-checking `dirty` — the user has
    /// just told us they accept losing the in-flight edits.
    ConfirmDiscard,
    /// User clicked "Отменить" in the unsaved-changes confirmation
    /// modal, or pressed Esc while it was open. The handler clears
    /// `pending_action` and leaves the document untouched; the
    /// modal disappears on the next frame.
    CancelDiscard,
    /// Iced reports that the user has asked the OS to close the
    /// window (× caption button, Alt+F4, taskbar close item). With
    /// `exit_on_close_request(false)` set on the application, iced
    /// no longer auto-closes — it forwards the request as a
    /// `window::Event::CloseRequested` which the keyboard/mouse
    /// listener turns into this message. The handler routes it
    /// through the dirty gate: with no unsaved edits we dispatch
    /// `iced::window::close` immediately, otherwise we stash
    /// `PendingAction::CloseWindow` and let the modal take over.
    WindowCloseRequested,
}
