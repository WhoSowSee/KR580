//! Application shell: the iced state container, message routing, theme
//! selection, and the keyboard subscription.
//!
//! The two heaviest sub-pieces live in dedicated modules:
//!
//! - `messages` owns the `Message` enum (it grows often and would crowd
//!   the state container otherwise).
//! - `constants` owns the widget identifiers, the register order, and a
//!   couple of register-name helpers. They are re-exported from this
//!   module so the rest of the crate can keep importing them as
//!   `crate::app::FOO`.

mod constants;
mod messages;
mod modal;
mod register_inline;
mod undo;

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID, parse_register_name,
    register_name,
};
pub(crate) use messages::{MenuId, Message, RegisterInlineTarget, SpeedTier};
pub(crate) use modal::DiscardModalButton;
pub(crate) use register_inline::RegisterMove;
pub(crate) use undo::{UndoEntry, UndoStack};

use iced::{Point, event, keyboard, mouse, time};
use iced::{Subscription, Task, Theme};
use k580_app::{AppSnapshot, EmulatorHandle, Snapshot580Flavour, initial_snapshot, spawn_emulator};
use k580_core::RegisterName;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::platform;

/// Default tier the speed switch boots into. `Medium` is the rest
/// position the user reaches for ~80% of the time — the program
/// visibly walks but doesn't crawl. The "Файл → Новый файл" gesture
/// re-arms the same tier (we never silently snap to a different one
/// behind the user's back).
pub(crate) const DEFAULT_SPEED_TIER: SpeedTier = SpeedTier::Medium;

/// Hz the "Slow" tier targets. 5 Hz = one instruction every 200 ms,
/// the режим обучения / отладки where the user reads each line out
/// loud before the highlight moves on.
pub(crate) const SLOW_TIER_HZ: u32 = 5;

/// Hz the "Medium" tier targets. 20 Hz reads as "the program is
/// running" while the eye still keeps up with each PC update — the
/// rest position of the switch.
pub(crate) const MEDIUM_TIER_HZ: u32 = 20;

/// Fallback Hz the "High" tier uses when we can't query the OS for
/// the primary monitor's refresh rate. 60 Hz is the floor every
/// modern desktop guarantees, so under-promising here is safer than
/// guessing higher and burning CPU on UI ticks the panel can't
/// physically display.
pub(crate) const HIGH_TIER_FALLBACK_HZ: u32 = 60;

/// Hard ceiling on the resolved "High" Hz. 240 Hz monitors exist;
/// 480 Hz panels are starting to ship. Above that we'd be paying
/// the cost of a UI tick per frame for changes the eye can't see,
/// and the hard floor on the worker (`MIN_STEP_INTERVAL = 1ms`)
/// would kick in anyway, so we cap at a sensible practical limit.
pub(crate) const HIGH_TIER_CEILING_HZ: u32 = 240;

/// Hz the "Max" tier targets. 1000 Hz is the practical ceiling of
/// the worker — `MIN_STEP_INTERVAL = 1 ms` floors the
/// `SetStepInterval` value, so anything higher would just be clamped
/// at the actor boundary anyway. Unlike `High`, `Max` is **not**
/// coupled to the monitor refresh rate: the UI subscription still
/// fires at ~60 Hz (the 16 ms floor in `subscription`), so a 1000 Hz
/// worker delivers ~16 instructions per Tick and the highlighted row
/// visibly *jumps* across memory instead of walking. That's the
/// explicit trade the label "Максимум" promises — "выполни как можно
/// быстрее, не показывай мне каждый шаг" — for users who just want
/// the program to finish (e.g. while iterating on a long routine).
pub(crate) const MAX_TIER_HZ: u32 = 1000;

/// Resolves a `SpeedTier` to its target instructions/sec. `Slow`,
/// `Medium`, and `Max` are constants; `High` queries the platform
/// for the primary display's refresh rate and clamps it into a
/// usable range. Called from both the message handler (which ships
/// the value to the worker as a `Duration`) and the `subscription`
/// (which uses it to pace UI ticks against worker output).
pub(crate) fn tier_hz(tier: SpeedTier) -> u32 {
    match tier {
        SpeedTier::Slow => SLOW_TIER_HZ,
        SpeedTier::Medium => MEDIUM_TIER_HZ,
        SpeedTier::High => platform::primary_monitor_refresh_hz()
            .unwrap_or(HIGH_TIER_FALLBACK_HZ)
            .clamp(HIGH_TIER_FALLBACK_HZ, HIGH_TIER_CEILING_HZ),
        SpeedTier::Max => MAX_TIER_HZ,
    }
}

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) status: String,
    pub(crate) selected_register: RegisterName,
    pub(crate) register_name_input: String,
    pub(crate) register_value_input: String,
    pub(crate) active_register_target: Option<RegisterInlineTarget>,
    pub(crate) inline_register_target: Option<RegisterInlineTarget>,
    pub(crate) hovered_register_target: Option<RegisterInlineTarget>,
    pub(crate) memory_scroll_first_row: u16,
    pub(crate) memory_scroll_offset: f32,
    pub(crate) memory_viewport_height: f32,
    pub(crate) memory_scroll_visible_ticks: u8,
    pub(crate) opcode_scroll_visible_ticks: u8,
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
    pub(crate) memory_inline_value_input: String,
    pub(crate) opcode_dropdown_address: Option<u16>,
    pub(crate) opcode_search_input: String,
    /// Cached substring pattern for the address-search workflow. Stored
    /// separately from `memory_address_input` because every successful
    /// match overwrites the input with the matched 4-digit address; without
    /// this cache the second Ctrl+Enter would search for the matched
    /// address itself instead of the original pattern.
    pub(crate) memory_search_pattern: Option<String>,
    /// Latest known state of the keyboard modifiers. Used to disambiguate
    /// `Enter` (apply memory write) from `Ctrl+Enter` (find next match) which
    /// the text input cannot tell apart on its own.
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
    /// Identifier of the text input that the user has most recently
    /// interacted with, used purely to drive cosmetic focus styling on the
    /// spinner shells. Iced 0.14 has no `on_focus`/`on_blur` callbacks, so
    /// we sync this from any signal that implies focus (typing, Tab
    /// navigation, explicit focus tasks).
    pub(crate) focused_input: Option<&'static str>,
    /// Latest known cursor position, refreshed on every
    /// `mouse::Event::CursorMoved` from the global event listener. The
    /// `MousePressed` handler uses this to reconcile focus state against
    /// the click coordinates, because `mouse::Event::ButtonPressed` only
    /// carries the button identity. Defaults to the origin until iced
    /// reports the first cursor movement; in practice the user has to
    /// move the cursor before they can click anything, so the default
    /// is never observed.
    pub(crate) latest_cursor_position: Point,
    /// Visual "armed" state of the action panel's run/pause toggle.
    /// Decoupled from `AppCommand::Run` dispatch (see `Message::ToggleRun`)
    /// so empty pages never burn 100k T-states on a stray click.
    pub(crate) running: bool,
    /// One-shot signal that the next `Message::Tick` must run
    /// `follow_pc_during_run` even though `self.running` is already
    /// `false`. Set in `consume_event` for the auto-pause branches
    /// (`HaltStateChanged`, `ErrorRaised`, `Stopped`). At high speed —
    /// e.g. 1000 Hz — the worker can drain a long burst of
    /// `StateChanged` snapshots followed by a terminal `Stopped` /
    /// `HaltStateChanged` inside a single 100 ms tick. Without this
    /// flag, `consume_event` clears `self.running` *before* the Tick
    /// branch reads it, so the closing `follow_pc_during_run` never
    /// runs and the highlight is left on whichever row the previous
    /// tick reached. The flag is consumed (set back to `false`) the
    /// moment Tick processes it, so it never strands the highlight in
    /// follow-mode after the run truly stops.
    pub(crate) pending_follow_pc: bool,
    /// Set on `TactAdvanced { instruction_boundary: true }`; cleared by
    /// the step-tact handler. PC mutates on the first tact in core, so
    /// before/after comparison would teleport — the handler waits for
    /// this flag instead.
    pub(crate) last_tact_was_boundary: bool,
    /// Tracks how many frames iced has rendered since startup. We keep the
    /// window cloaked (DWM-hidden on Windows) until the second frame so the
    /// OS never gets a chance to flash its default white client area.
    pub(crate) startup_frames_seen: u8,
    /// Identifier of the top-level menu that is currently dropped down,
    /// or `None` if the menu bar is at rest. Set by `MenuToggled` and
    /// cleared by `MenuClosed`. The menu-bar view reads this to decide
    /// whether to render the floating dropdown panel, and the root
    /// `view` adds a transparent scrim that closes the menu on stray
    /// clicks while it is open.
    pub(crate) open_menu: Option<MenuId>,
    /// Filesystem path of the snapshot that the user is currently
    /// editing, set whenever `OpenSnapshot` succeeds and after every
    /// successful `SaveSnapshot` / `SaveSnapshotAs`. With this stored,
    /// "Сохранить" overwrites the file in place instead of asking the
    /// user where to put it again — that is the gesture every desktop
    /// app implements and the absence of it is exactly what the user
    /// reported as "когда я нажимаю Сохранить, мне снова предлагают
    /// сохранить, хотя я его уже открыл". `Сохранить как` ignores it
    /// (and replaces it on success).
    pub(crate) current_snapshot_path: Option<PathBuf>,
    /// Filesystem path of the **legacy** `.580` file the user is
    /// currently editing, set whenever `OpenLegacySnapshot` succeeds
    /// and after every successful `SaveLegacySnapshot`. Lives in a
    /// separate field from `current_snapshot_path` so that Ctrl+S
    /// (plain Save, K580 v1 TLV) and Ctrl+Alt+S (Save legacy) can
    /// each remember *their own* path without one silently
    /// overwriting the other in a format the reference emulator
    /// could not load back. Cleared whenever the document switches
    /// to a non-legacy origin (Open / New / Import / Save-as on the
    /// v1 path) — the next legacy-format save then prompts
    /// for a fresh location instead of reusing a stale path that no
    /// longer matches the document.
    pub(crate) current_legacy_snapshot_path: Option<PathBuf>,
    /// Scratch slot the runtime uses to communicate the result of an
    /// auto-detect `LoadAnySnapshot` dispatch back to its caller. The
    /// worker emits `AppEvent::SnapshotFlavourLoaded(flavour)` after a
    /// successful auto-detect load; `consume_event` parks the value
    /// here, and `load_snapshot_from_path` reads it on the way out to
    /// route the path into the matching "current path" field
    /// (`current_snapshot_path` for Modern, `current_legacy_snapshot_path`
    /// for Legacy). We keep it as a transient field rather than
    /// threading the flavour through `dispatch_sync`'s return value so
    /// the dispatch path stays oblivious to the flavour concept and
    /// only the open-file helper that asked for auto-detection has to
    /// reason about it. Always reset to `None` immediately after the
    /// helper consumes it.
    pub(crate) pending_snapshot_flavour: Option<Snapshot580Flavour>,
    /// Currently active tier on the speed switch. Three discrete
    /// positions (Slow / Medium / High) replace the older free-form
    /// slider so the user picks an intent ("walk every line" / "watch
    /// it run" / "as fast as the screen can paint") instead of
    /// chasing a Hz number whose effect plateaued above 60 anyway.
    /// Resolved to a concrete Hz on demand via `tier_hz()`.
    pub(crate) speed_tier: SpeedTier,
    /// Floating notification shown at the top centre of the window
    /// when a run/step gesture is refused because the CPU has halted
    /// (Variant A: halt-blocked controls — see `docs/ui_app.md`).
    /// Lives outside `self.status` because the status bar is the
    /// wrong place for the message: at 13 px on the dark board the
    /// multi-line Russian hint blended into the chrome, and the user
    /// asked for it to come back as a separate framed notice that
    /// sits above the schematic the same way the file-menu dropdown
    /// does. Cleared by `ResetCpu` (the only gesture that unblocks
    /// the run state) and by every successful step / run path so the
    /// message disappears the moment the user is no longer halt-blocked.
    pub(crate) halt_notice: Option<String>,
    /// Wall-clock deadline at which the floating halt notice should
    /// auto-dismiss itself. Mirrors `error_notice_dismiss_at`: the
    /// user explicitly asked for halt_notice to fade after 8 s the
    /// same way the file-error overlay does, so the two passive
    /// notices behave identically on every axis except phrasing.
    /// Polled inside `Message::Tick` next to the error-notice
    /// branch; kept in lockstep with `halt_notice` via
    /// `clear_halt_notice()` so a stale deadline can never fire on a
    /// fresh notice or pretend it dismissed something the user
    /// already cleared by hand.
    pub(crate) halt_notice_dismiss_at: Option<Instant>,
    /// Latch raised the moment a run / step / restart gesture is
    /// refused by the halt-block check (see `raise_halt_notice`).
    /// While `true`, every execution-side button on the action panel
    /// renders disabled (their `Message` is wrapped in `None` so iced
    /// drops the `on_press` callback) and the corresponding keyboard
    /// shortcuts (Ctrl+R, Ctrl+T, Ctrl+Y) early-return inside the
    /// runtime helpers. The block deliberately outlives the
    /// 8-second halt-notice fade — that is the user's whole point:
    /// "до тех пор пока не сброшу флаг или регистры", the overlay
    /// disappearing without unblocking would leave them with the
    /// same dead buttons but no on-screen explanation.
    ///
    /// Cleared by exactly the gestures that make the halt bit
    /// unblockable in the worker: `Message::ResetCpu` (full
    /// power-on reset), `Message::ClearHalt` (the strictly weaker
    /// "flip the halt bit only" twin), and the `apply_snapshot`
    /// branch that sees the new snapshot reports `halted = false`
    /// (snapshot loads, undo of a halting instruction, etc.). All
    /// three sites also clear `halt_notice` in lockstep, keeping
    /// the visual and logical halves of "user is unstuck now"
    /// aligned.
    ///
    /// Setting the latch lives inside `raise_halt_notice` rather
    /// than at every call site so a future halt-block path cannot
    /// forget to arm the block — there is exactly one chokepoint
    /// for "user just discovered the CPU is halted", and that's
    /// where both the overlay and the latch turn on together.
    pub(crate) run_blocked_after_halt: bool,
    /// Floating notification shown at the top centre of the window
    /// when a file-system operation fails (open / save / import /
    /// export couldn't read or write the file). Lives in its own
    /// field, separate from `halt_notice`, because the two have
    /// distinct lifecycles: `halt_notice` is gated on the CPU's
    /// halted bit and clears the moment the user resets registers,
    /// while `error_notice` is gated on the user's *next gesture* —
    /// any successful file operation, an Esc press, or a click on
    /// the notice itself dismisses it. Routed through this field
    /// rather than `self.status` because the status bar at 13 px on
    /// the dark schematic plate is too quiet a channel for "open
    /// failed" — the user reported missing the message entirely. A
    /// red-tinted framed overlay catches the eye the way a status
    /// line cannot.
    pub(crate) error_notice: Option<String>,
    /// Wall-clock deadline at which the floating error notice should
    /// auto-dismiss itself. Set to `Instant::now() + 8s` whenever
    /// `error_notice` is populated; cleared when the notice is
    /// cleared. Polled inside `Message::Tick` so the dismissal is
    /// driven by the same timer that paces the rest of the UI — at
    /// 16-100 ms tick intervals the actual dismiss happens within
    /// one frame of the deadline, which is plenty for a passive
    /// notification.
    ///
    /// Driven off `Instant` rather than a tick counter because the
    /// tick interval is variable (it tracks the CPU speed tier); a
    /// counter would need to be retuned every time the user changed
    /// speed. A wall-clock deadline is just always 8 seconds in the
    /// future regardless of how fast Tick is firing.
    pub(crate) error_notice_dismiss_at: Option<Instant>,
    /// Floating notification shown at the top centre of the window
    /// when the user opens a legacy-format `.580` file (whether via
    /// double-click / `argv[1]`, or "Файл → Открыть (старый
    /// формат)"). Same chrome as `error_notice` and `halt_notice`
    /// except the frame is `TOKYO_YELLOW` instead of `TOKYO_RED`:
    /// yellow signals "heads up, not an error" — the user does not
    /// need to act, just notice that the snapshot came in via the
    /// legacy decoder so a subsequent save routes through the right
    /// serializer. Lifecycle is independent of the other two
    /// notices: it auto-fades after 5 seconds (vs. 8 for the
    /// red-bordered ones — informational notices ride on a shorter
    /// timer per the user's explicit "5 секунд" ask), or earlier on
    /// click / Esc. Cleared at the start of every fresh file
    /// gesture so the user can tell *this* attempt apart from the
    /// last one.
    pub(crate) info_notice: Option<String>,
    /// Wall-clock deadline at which the floating info notice should
    /// auto-dismiss itself. Mirrors `error_notice_dismiss_at` and
    /// `halt_notice_dismiss_at`: armed in lockstep with
    /// `info_notice` via `raise_info_notice`, polled inside
    /// `Message::Tick`, and cleared in lockstep via
    /// `clear_info_notice`. The 5-second window is shorter than the
    /// 8 seconds the red overlays use because the info notice is
    /// strictly informational — there is nothing for the user to
    /// act on, so a longer fade would just block more of the
    /// schematic for no benefit.
    pub(crate) info_notice_dismiss_at: Option<Instant>,
    /// Cached identifier of the main window. Captured on the very
    /// first `WindowOpened` so the custom caption buttons (drag /
    /// minimise / toggle-maximise / close) can dispatch
    /// `iced::window::*` tasks without an extra `get_latest` round
    /// trip per click. `None` until the first frame; the buttons
    /// short-circuit to `Task::none()` while it is unset.
    pub(crate) window_id: Option<iced::window::Id>,
    /// Latest known maximised state of the main window. Driven by the
    /// `WindowMaximizedChanged` poll — see the matching message in
    /// `app::messages`. The maximise/restore caption button reads it
    /// to decide which of the two glyphs (`window-maximize` /
    /// `window-restore`) to render: without this flag the icon would
    /// stay frozen on "maximise" even after the window already fills
    /// the screen.
    pub(crate) window_maximized: bool,
    /// Whether the top-level menu category labels (Файл, МП-Система,
    /// Вид, Настройки, Справка) are currently visible in the menu bar.
    /// Toggled by clicking the cpu brand mark on the far left of the
    /// bar — same gesture native macOS / Windows apps assign to a
    /// hamburger or "show menu" affordance, and it lets the user
    /// reclaim the bar's vertical band as pure drag/title chrome
    /// when they don't need the dropdowns. Default `true` so a fresh
    /// session reads as the familiar menu bar; the cpu glyph itself
    /// stays visible regardless so the user always has something to
    /// click to bring the categories back.
    pub(crate) menu_categories_visible: bool,
    /// Single chronological timeline of edits the user can rewind via
    /// Ctrl+Z and replay via Ctrl+Shift+Z. Holds both text-input
    /// edits (coalesced per-field so consecutive keystrokes collapse
    /// into one entry) and full `Cpu8080State` snapshot pairs for
    /// every gesture that mutates the worker (`SetMemory`,
    /// `SetRegister`, `ResetCpu`, `ResetRam`, snapshot/import loads).
    /// See `app::undo` for the storage shape and coalescing rules.
    pub(crate) undo_stack: UndoStack,
    /// Set the moment the user makes a CPU-mutating gesture
    /// (`SetMemory`, `SetRegister`, `ResetCpu`, `ResetRam`, inline
    /// commit, opcode picker write) and cleared on every gesture that
    /// establishes a fresh \"saved\" baseline: `SaveSnapshot`,
    /// `SaveSnapshotAs`, `LoadSnapshot`, `Import`, `NewFile`. Read by
    /// the gestures that throw away the current document
    /// (`OpenSnapshot`, `NewFile`, `Import`, `WindowClose`) to decide
    /// whether to put up a confirmation modal first. Without this
    /// flag the only way to know if there is unsaved work is to
    /// compare the live snapshot against the on-disk file, which is
    /// both expensive and racy.
    pub(crate) dirty: bool,
    /// Keyboard focus inside the unsaved-changes confirmation modal.
    /// The modal is drawn by our own overlay rather than a native
    /// dialog, so Tab / Shift+Tab / Enter are routed through this
    /// explicit two-button focus ring while `pending_action` is set.
    pub(crate) discard_modal_focus: DiscardModalButton,
    /// Action that has been queued behind a confirmation modal. The
    /// gestures that may throw away unsaved work (open file / new
    /// file / import / close window) check `dirty` first; with the
    /// flag set they stash the action here and put up a modal that
    /// asks the user to confirm or cancel. `Message::ConfirmDiscard`
    /// then runs whatever was stashed; `Message::CancelDiscard`
    /// clears the field. While this is `Some`, `app::modal` captures
    /// every user event except the modal's own buttons, Enter,
    /// Tab/Shift+Tab, Esc, and passive system bookkeeping.
    pub(crate) pending_action: Option<PendingAction>,
}

/// Action queued behind the \"unsaved changes\" confirmation modal.
/// Each variant corresponds to one of the gestures that throw away
/// the current document. Carries enough state to replay the action
/// verbatim once the user confirms — `OpenSnapshotAt` carries the
/// path the file dialog already returned, so confirming does not
/// reopen the picker.
#[derive(Clone, Debug)]
pub(crate) enum PendingAction {
    /// User picked \"Файл → Открыть\" / Ctrl+O. The file dialog has
    /// not run yet; confirmation re-enters `open_snapshot` so the
    /// dialog opens *after* the user decides to discard.
    OpenSnapshot,
    /// User picked \"Файл → Новый файл\" / Ctrl+N. Confirmation runs
    /// the same wipe-RAM-and-CPU sequence the dirty-free path does.
    NewFile,
    /// User picked \"Файл → Импорт\" / Ctrl+I. Same shape as
    /// `OpenSnapshot`: confirmation opens the picker.
    Import,
    /// User picked the legacy-format open row. Same shape as
    /// `OpenSnapshot`: the file dialog has not run yet; confirmation
    /// re-enters `open_legacy_snapshot` so the dialog opens after
    /// the user accepts the discard. Saving as legacy does not need
    /// a `PendingAction` variant — it is non-destructive (writes a
    /// new file, never touches live state).
    OpenLegacySnapshot,
    /// User clicked the × caption button (or hit Alt+F4). The
    /// `WindowClose` handler intercepted the request before the OS
    /// could close the window; confirmation dispatches the actual
    /// `iced::window::close`.
    CloseWindow,
}

impl DesktopApp {
    /// Constructs the app and, when an initial snapshot path is given,
    /// queues a `LoadSnapshotFromPath` task so the file is opened as
    /// soon as the iced runtime starts pumping messages. This is the
    /// entry point used by `main` when the OS hands us a `.580` file
    /// via `argv[1]` — the user double-clicks the file in Explorer
    /// and expects the emulator to come up already pointed at it.
    /// Pass `None` for the normal "blank slate" launch.
    pub(crate) fn with_initial_path(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let handle = spawn_emulator();
        let startup_task = match initial {
            Some(path) => Task::done(Message::LoadSnapshotFromPath(path)),
            None => Task::none(),
        };
        (
            Self {
                handle,
                snapshot: initial_snapshot(),
                status: "Готов".to_owned(),
                selected_register: RegisterName::A,
                register_name_input: "A".to_owned(),
                register_value_input: "00".to_owned(),
                active_register_target: None,
                inline_register_target: None,
                hovered_register_target: None,
                memory_scroll_first_row: 0,
                memory_scroll_offset: 0.0,
                memory_viewport_height: 0.0,
                memory_scroll_visible_ticks: 0,
                opcode_scroll_visible_ticks: 0,
                memory_address_input: "0000".to_owned(),
                memory_value_input: "00".to_owned(),
                memory_inline_value_input: "00".to_owned(),
                opcode_dropdown_address: None,
                opcode_search_input: String::new(),
                memory_search_pattern: None,
                keyboard_modifiers: keyboard::Modifiers::default(),
                focused_input: None,
                latest_cursor_position: Point::ORIGIN,
                running: false,
                pending_follow_pc: false,
                last_tact_was_boundary: false,
                startup_frames_seen: 0,
                open_menu: None,
                // The path is set by `load_snapshot_from_path` on the
                // first tick — pre-seeding here would just duplicate
                // that write and add no observable behaviour, since
                // the user cannot interact with the app before the
                // startup task drains.
                current_snapshot_path: None,
                // Same rationale as `current_snapshot_path`: nothing
                // is loaded yet, the legacy-format save row
                // gesture must therefore prompt for a path on first
                // use. The startup task may flip this if the OS hands
                // us a `.580` via argv[1] *and* the loader picks the
                // legacy branch — but that decision lives in
                // `load_snapshot_from_path`, not here.
                current_legacy_snapshot_path: None,
                pending_snapshot_flavour: None,
                // The speed switch boots into Medium — that's the
                // rest position the user reaches for ~80% of the time
                // (program visibly walks, eye keeps up with each PC
                // update). Slow / High are deliberate gestures, not
                // defaults.
                speed_tier: DEFAULT_SPEED_TIER,
                halt_notice: None,
                halt_notice_dismiss_at: None,
                // Fresh launch is by definition not halted (the
                // initial snapshot's `halted = false`), so the
                // latch starts disarmed. The first run/step that
                // hits HLT will arm it through `raise_halt_notice`,
                // and ResetCpu / ClearHalt / a non-halted snapshot
                // load will disarm it back.
                run_blocked_after_halt: false,
                // Fresh launch has no errors to report yet — the
                // overlay only materialises when a file operation
                // explicitly fails (`AppEvent::ErrorRaised`) or the
                // dispatch channel itself errors out.
                error_notice: None,
                error_notice_dismiss_at: None,
                // Same as the error notice: nothing to inform the user
                // about on a fresh launch. The first legacy-format
                // open (whether via argv[1] or the menu) will arm
                // both fields through `raise_info_notice`.
                info_notice: None,
                info_notice_dismiss_at: None,
                window_id: None,
                window_maximized: false,
                menu_categories_visible: true,
                undo_stack: UndoStack::default(),
                // Fresh launch is by definition a clean baseline: no
                // edits have happened, the document on disk (if any
                // — the OS may have handed us a `.580` via argv[1]
                // and the startup task will load it on the first
                // tick) matches what the user sees. Both fields stay
                // `false`/`None` until a gesture flips them.
                dirty: false,
                discard_modal_focus: DiscardModalButton::Cancel,
                pending_action: None,
            },
            startup_task,
        )
    }

    /// Clears both halves of the error-notice state in lockstep. The
    /// floating overlay needs `error_notice` to disappear from the
    /// view, but the auto-dismiss deadline must go with it — a stale
    /// `Some(deadline)` left behind would silently fire on the next
    /// Tick and pretend it dismissed something the user had already
    /// cleared. Used everywhere the notice is reset: explicit
    /// dismissal (click / Esc / `DismissErrorNotice`), each fresh
    /// file gesture (open / save / save-as / save-legacy /
    /// open-legacy / import / export), and the auto-dismiss branch
    /// in `Message::Tick` itself.
    pub(crate) fn clear_error_notice(&mut self) {
        self.error_notice = None;
        self.error_notice_dismiss_at = None;
    }

    /// Clears both halves of the halt-notice state in lockstep, the
    /// same pattern `clear_error_notice` follows. The auto-dismiss
    /// deadline must travel with the visible field; without it a
    /// stale `Some(deadline)` would silently fire on the next Tick
    /// and look like it dismissed a fresh notice the user had only
    /// just seen. Used by every site that nulls `halt_notice`:
    /// explicit dismissal (click / Esc / `DismissHaltNotice`), the
    /// `apply_snapshot` branch that detects a non-halted CPU, and
    /// the auto-dismiss arm in `Message::Tick`.
    pub(crate) fn clear_halt_notice(&mut self) {
        self.halt_notice = None;
        self.halt_notice_dismiss_at = None;
    }

    /// Clears both halves of the info-notice state in lockstep, the
    /// same pattern `clear_error_notice` and `clear_halt_notice`
    /// follow. Used by every site that nulls `info_notice`:
    /// explicit dismissal (click / Esc / `DismissInfoNotice`), the
    /// auto-dismiss arm in `Message::Tick`, and the start of every
    /// fresh file gesture so the previous notice does not linger
    /// across an unrelated attempt.
    pub(crate) fn clear_info_notice(&mut self) {
        self.info_notice = None;
        self.info_notice_dismiss_at = None;
    }

    /// Arms the info-notice overlay with `text` and a 5-second
    /// auto-dismiss deadline. Centralised here so every caller gets
    /// the same timer constant (vs. the 8 seconds the red overlays
    /// use) without duplicating the `Instant::now() + Duration`
    /// dance at every site. Currently the only caller is
    /// `load_snapshot_from_path` on the legacy branch, but the
    /// helper is public to the crate so any future "heads up"
    /// notice can route through it instead of growing yet another
    /// pair of fields.
    pub(crate) fn raise_info_notice(&mut self, text: String) {
        self.info_notice = Some(text);
        self.info_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(5));
    }

    /// Arms the halt-notice overlay with the canonical two-line body
    /// — first line is the diagnosis ("процессор остановлен командой
    /// HLT"), second line is the recommendation ("Сбросьте регистры
    /// или флаг HLT"). Centralised here so every halt-block site
    /// (run-arming, step-instruction, step-tact) gets the same text
    /// and the same 8-second auto-dismiss deadline without each
    /// site having to remember to bump both halves.
    ///
    /// The 8 s window matches `error_notice` exactly. The user
    /// explicitly asked for the two passive overlays to behave
    /// identically — same chrome, same fade — so the timer constant
    /// has to track the one in `consume_event::ErrorRaised`.
    pub(crate) fn raise_halt_notice(&mut self) {
        self.halt_notice =
            Some("Процессор остановлен командой HLT\nСбросьте регистры или флаг HLT".to_owned());
        self.halt_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
        // Single chokepoint for "user just discovered the CPU is
        // halted": every halt-block path (run-arming, step-instruction,
        // step-tact, restart-program) routes through this helper, so
        // arming the latch here means no individual call site can
        // forget to do it. The latch outlives the 8-second overlay
        // fade — disabling the buttons is the user-visible contract
        // ("до тех пор пока не сброшу флаг или регистры"), and the
        // overlay is just the explanation for why they went dead.
        self.run_blocked_after_halt = true;
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        if let Some(task) = self.route_discard_modal_message(&message) {
            return task;
        }

        match message {
            Message::Tick => {
                self.pull_events();
                self.memory_scroll_visible_ticks =
                    self.memory_scroll_visible_ticks.saturating_sub(1);
                self.opcode_scroll_visible_ticks =
                    self.opcode_scroll_visible_ticks.saturating_sub(1);
                // Auto-dismiss the floating error notice once the
                // 8-second window armed by `consume_event` is up.
                // Polled here rather than via a dedicated timer
                // subscription because Tick already fires often
                // enough (16-100 ms) that the visible delay between
                // deadline and dismissal is at most one frame —
                // imperceptible for a passive notification — and
                // adding a second `time::every` for a feature this
                // small would only complicate the subscription list.
                if let Some(deadline) = self.error_notice_dismiss_at
                    && Instant::now() >= deadline
                {
                    self.clear_error_notice();
                }
                // Same auto-dismiss treatment for the halt notice —
                // the user asked for the two passive overlays to
                // fade identically, and the deadline is armed by
                // `raise_halt_notice` in lockstep with the visible
                // field, so this branch is structurally identical
                // to the error-notice one above.
                if let Some(deadline) = self.halt_notice_dismiss_at
                    && Instant::now() >= deadline
                {
                    self.clear_halt_notice();
                }
                // Same auto-dismiss treatment for the info notice —
                // the user asked the legacy-format heads-up to
                // disappear after 5 seconds. The deadline is armed
                // by `raise_info_notice` in lockstep with the
                // visible field, so this branch is structurally
                // identical to the error/halt ones above; only the
                // timer constant is shorter (5 s vs. 8 s).
                if let Some(deadline) = self.info_notice_dismiss_at
                    && Instant::now() >= deadline
                {
                    self.clear_info_notice();
                }
                // Drag the memory highlight along with PC while the
                // paced Run loop is firing. `pull_events` has just
                // folded the latest snapshot in, so `cpu.pc` already
                // reflects the most recent worker tick; if the user's
                // visible spinner address has fallen behind, snap the
                // selection forward and re-anchor the viewport. Done
                // here (not inside `consume_event`) because issuing the
                // scroll Task from this branch keeps it on the same
                // frame as the snapshot apply, and because Tick is the
                // single place where we already centralise per-frame
                // bookkeeping for the memory list.
                //
                // `pending_follow_pc` covers the "fast run that already
                // halted inside this tick" case: at e.g. 1000 Hz the
                // worker can publish a long burst of `StateChanged`
                // followed by a terminal `Stopped` / `HaltStateChanged`
                // before we ever return from `pull_events`, so by the
                // time we read `self.running` it is already `false` and
                // the highlight would be stranded mid-program. The flag
                // is set on those auto-pause branches and consumed here
                // so the closing tick still chases PC to its final
                // resting place (HLT for the halt path).
                if self.running || self.pending_follow_pc {
                    self.pending_follow_pc = false;
                    return self.follow_pc_during_run();
                }
            }
            Message::CursorMoved(point) => {
                // Cache the latest cursor position so the next
                // `MousePressed` knows where the click landed. The
                // mouse::Event::ButtonPressed variant carries only
                // the button identity, not the coordinates, so we
                // have to track them ourselves.
                self.latest_cursor_position = point;
            }
            Message::MousePressed => {
                // Authoritative focus reconciliation, in two passes.
                //
                // Pass 1 (`find_focusable_at`) is read-only: it walks
                // the widget tree and returns the id of the focusable
                // whose bounds contain the click point, or `None` if
                // the click missed every focusable.
                //
                // Pass 2 (`unfocus_except`) is the mutation: given a
                // confirmed hit id, it walks the tree again and
                // clears `state.is_focused` on every focusable that
                // is *not* the hit. This is what fixes the
                // column→stack capture race described in
                // `runtime::focus_ops` — text_inputs in sibling
                // panels never see the click, so without this pass
                // they would keep stale `Some(_)` flags from earlier
                // typing.
                //
                // Pass 2 only runs when pass 1 found a hit. A `None`
                // result is treated as "leave focus alone" instead
                // of "clear everything" because of a layout race:
                // iced processes the click in the freshly-clicked
                // input's `update` *before* draining the operation
                // queue, and the layout may shift by a pixel or two
                // in between, making the input's reported bounds
                // miss the click point. A single-pass operation
                // would then unfocus the input that just processed
                // the click, dropping the caret mid-edit. Splitting
                // the work and bailing out on `None` keeps repeat
                // clicks inside an already-focused input safe.
                return iced::advanced::widget::operate(crate::runtime::find_focusable_at(
                    self.latest_cursor_position,
                ))
                .map(Message::FocusReconciled);
            }
            Message::FocusReconciled(hit) => {
                const TRACKED: [&str; 6] = [
                    MEMORY_ADDRESS_INPUT_ID,
                    MEMORY_VALUE_INPUT_ID,
                    REGISTER_NAME_INPUT_ID,
                    REGISTER_VALUE_INPUT_ID,
                    REGISTER_INLINE_INPUT_ID,
                    MEMORY_INLINE_INPUT_ID,
                ];

                // A click anywhere — focusable or dead space — ends
                // whatever in-flight text-edit was being coalesced
                // onto the top undo entry. The new gesture is by
                // definition a logically separate edit (the user
                // moved their attention), so the next `push_text`
                // must start a fresh entry instead of glueing onto
                // the previous one.
                self.undo_stack.break_coalescing();

                // Map the bare `Id` back to one of our static string
                // identifiers so the cosmetic shell border can index
                // into its own table. Untracked focusables (the
                // opcode-search input, for example, which is unkeyed)
                // resolve to `None` and clear the indicator entirely
                // — the user clicked into a region we don't decorate
                // with a focus ring.
                let resolved = hit.as_ref().and_then(|id| {
                    TRACKED
                        .into_iter()
                        .find(|known| *id == iced::widget::Id::new(known))
                });

                // Update the cosmetic tracker first so the focus ring
                // matches the new state on the same frame. Two cases
                // here:
                //
                // * `hit = Some(id)` — pass 1 found a focusable
                //   under the click. Update the ring and chain pass
                //   2 (`unfocus_except`) to clear stale focus on
                //   every *other* focusable. We deliberately do not
                //   touch the hit widget's state: iced's
                //   `text_input::update` has already set
                //   `is_focused = Some(_)` for it, and calling
                //   `state.focus()` ourselves would snap the caret
                //   to the end via `move_cursor_to_end`.
                //
                // * `hit = None` — pass 1 found nothing. Either the
                //   click landed in dead space (panel border, label,
                //   gap between widgets) or a layout race left the
                //   focused input's bounds momentarily not matching
                //   the click point. In neither case is wiping all
                //   focus the right move: dead-space clicks should
                //   leave focus alone (otherwise the user can never
                //   keep typing after clicking the surrounding
                //   chrome), and races are exactly the scenario the
                //   split is designed to absorb. So we simply do not
                //   issue a pass 2 here, leaving every focusable's
                //   state untouched.
                if let Some(id) = hit {
                    self.focused_input = resolved;
                    return iced::advanced::widget::operate(crate::runtime::unfocus_except(id))
                        .discard();
                }
                // Pass-1 missed every focusable. Two scenarios fold
                // into this branch and we cannot tell them apart from
                // coordinates alone:
                //
                // 1. Dead-space click — the user clicked a panel
                //    border, label, or gap. Iced's text_input::update
                //    has already cleared `state.is_focused` on the
                //    previously focused input (every input that does
                //    not contain the click runs that clearing branch),
                //    so the caret is gone but our cosmetic tracker
                //    still points at the now-stale widget.
                //
                // 2. Layout-race false negative — the click landed on
                //    a focusable but a sub-pixel layout shift between
                //    the click event and our reconcile pass made the
                //    bounds miss. In this case iced's per-widget code
                //    *did* see the click and `state.is_focused` is
                //    still set on whatever input owns the caret.
                //
                // Polling `find_focused_optional()` lets iced be the
                // authoritative oracle: a `None` reply means scenario
                // 1 (clear the cosmetic tracker), a `Some` means
                // scenario 2 (leave it alone). The `_optional` variant
                // wraps the answer in `Option<Id>` and always reports
                // back via `Outcome::Some(option)` — the built-in
                // `find_focused` returns `Outcome::None` when nothing
                // is focused, which would silently drop the message
                // exactly when we need it the most.
                return iced::advanced::widget::operate(crate::runtime::find_focused_optional())
                    .map(Message::ResolveFocusedTracker);
            }
            Message::ResolveFocusedTracker(focused) => {
                // Iced says no focusable owns the caret right now —
                // the previous owner (Esc consumed it, or a
                // dead-space click cleared it) is gone. Drop the
                // cosmetic tracker so the shell border on the prior
                // input fades the same frame.
                //
                // A `Some(_)` reply means a focusable still has the
                // caret. We deliberately do nothing in that case:
                // the `*Changed`, `MemoryEnter`, and click-reconcile
                // paths are responsible for keeping the tracker in
                // sync on focus *acquisition*, and overwriting it
                // here would race with those.
                if focused.is_none() {
                    self.focused_input = None;
                }
            }
            Message::StepInstruction => return self.step_instruction_and_advance(),
            Message::RestartProgram => self.restart_program(),
            Message::StepTact => return self.step_tact_and_maybe_advance(),
            Message::ToggleRun => self.toggle_run(),
            Message::ResetCpu => {
                // ResetCpu is the canonical "clean power-on": it
                // wipes registers, PC, SP, interrupt state, halt,
                // and cycle_count in one shot, so the halt-block
                // latch must come down with it. The `apply_snapshot`
                // branch will also see `halted = false` on the
                // returning snapshot and clear `halt_notice` there,
                // but the latch governs UI controls and lives only
                // on the UI side, so we drop it here.
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ResetCpu);
            }
            Message::ResetRam => {
                // `ResetRam` wipes the program out of memory and, on
                // the worker side, also lifts the halt flip-flop —
                // the program that put the CPU into HLT is gone, so
                // staying halted-on-nothing is a pure UI artifact.
                // Lift the run-block latch here for the same reason
                // we do it for `ResetCpu` / `ClearHalt`: the latch is
                // UI-only state, the snapshot returning from the
                // worker will then confirm the un-halt through
                // `apply_snapshot`, and the user gets the execution
                // chips back the moment they hit «Сброс ОЗУ». Without
                // this branch the chips would stay greyed even after
                // a successful RAM wipe, which is exactly the bug the
                // user filed.
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ResetRam);
            }
            Message::ClearHalt => {
                // Strictly weaker reset than `ResetCpu`: only the
                // halt flip-flop flips. Routed through the undo
                // stack the same way every other CPU mutation is,
                // so Ctrl+Z brings the halt bit back if the user
                // changes their mind. Lift the UI latch in lockstep
                // with the worker dispatch — the snapshot returning
                // from the worker will also clear the notice via
                // `apply_snapshot`, but the latch is UI-only state
                // and we control it here.
                //
                // Гейт `!cpu.halted → no-op`: визуально пункт меню
                // «Сбросить флаг HLT» уже серый когда флаг выключен
                // (см. `menu_item` в `view/menu.rs` с `enabled=false`),
                // но шорткат `Ctrl+Shift+H` приходит сюда напрямую
                // через keyboard subscription, минуя UI-гейт. Без
                // этого ранне возврата нажатие шортката с уже
                // погашенным флагом пушило бы в undo-стек пустую
                // операцию (snapshot до == snapshot после) и без
                // нужды дёргало воркер. Здесь — единая точка, где
                // оба пути (клик и шорткат) сходятся, поэтому одного
                // чека достаточно.
                if !self.snapshot.cpu.halted {
                    return Task::none();
                }
                self.run_blocked_after_halt = false;
                self.dispatch_with_undo(k580_app::AppCommand::ClearHalt);
            }
            Message::ToggleHalt => {
                // Click on the "HLT ВКЛ" / "HLT ВЫКЛ" indicator at
                // the top of the schematic plate. The press flips
                // the halt flip-flop in whichever direction the
                // current state needs: halted -> running re-uses
                // the `ClearHalt` lifecycle (latch comes down,
                // worker emits `Stopped` + `HaltStateChanged(false)`),
                // running -> halted ships the new
                // `SetHalted(true)` verb that arms the bit
                // explicitly. Both legs route through
                // `dispatch_with_undo` so the press is reversible
                // — Ctrl+Z brings whichever state the user just
                // left back. We resolve the toggle direction here
                // (UI side) instead of inside the worker so the
                // snapshot we capture for the undo stack matches
                // what the user sees on screen at press time; the
                // worker's `SetHalted` then treats the value as
                // authoritative and only emits `HaltStateChanged`
                // when the bit actually flipped, keeping repeated
                // toggles cheap.
                if self.snapshot.cpu.halted {
                    self.run_blocked_after_halt = false;
                    self.dispatch_with_undo(k580_app::AppCommand::ClearHalt);
                } else {
                    self.dispatch_with_undo(k580_app::AppCommand::SetHalted(true));
                }
            }
            Message::OpenSnapshot => {
                // Dirty-gate: with unsaved edits, queue the action and
                // raise the modal instead of opening the picker. The
                // user gets a chance to cancel; if they confirm the
                // discard, `ConfirmDiscard` re-emits `OpenSnapshot`
                // through `Task::done`, but by then `dirty` has been
                // cleared so this branch falls through to the picker.
                if self.dirty {
                    self.open_discard_modal(PendingAction::OpenSnapshot);
                } else {
                    self.open_snapshot();
                }
            }
            Message::LoadSnapshotFromPath(path) => self.load_snapshot_from_path(path),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::SaveSnapshotAs => self.save_snapshot_as(),
            Message::SaveLegacySnapshot => self.save_legacy_snapshot(),
            Message::OpenLegacySnapshot => {
                // Same dirty-gate as `OpenSnapshot`: with unsaved edits,
                // queue the action and raise the modal. The legacy
                // open path replaces RAM + PC under the user's feet,
                // so the same "are you sure you want to lose this?"
                // protection applies. `ConfirmDiscard` re-emits this
                // message after wiping `dirty`, and the second pass
                // falls through to the picker.
                if self.dirty {
                    self.open_discard_modal(PendingAction::OpenLegacySnapshot);
                } else {
                    self.open_legacy_snapshot();
                }
            }
            Message::NewFile => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::NewFile);
                } else {
                    self.run_new_file();
                }
            }
            Message::Export => self.export_file(),
            Message::Import => {
                if self.dirty {
                    self.open_discard_modal(PendingAction::Import);
                } else {
                    self.import_file();
                }
            }
            Message::RegisterNameChanged(value) => {
                // Mirror focus into our cosmetic tracker so the shell
                // border updates the same frame the user starts typing.
                // `MousePressed` -> `reconcile_focus_at` already does
                // this on click; this write covers the case where the
                // user reaches the field via Tab and starts typing
                // before the next click event arrives.
                self.change_register_name(value);
                self.active_register_target = None;
                self.inline_register_target = None;
                self.focused_input = Some(REGISTER_NAME_INPUT_ID);
            }
            Message::RegisterPrevious => self.step_register(-1),
            Message::RegisterNext => self.step_register(1),
            Message::RegisterValueChanged(value) => {
                // See RegisterNameChanged — same rationale. We
                // deliberately do NOT return any focus operation here:
                // operations from `*Changed` handlers are queued and
                // can drain after the user has clicked into a different
                // panel, which would steal focus from the freshly
                // clicked input. The authoritative focus mutation
                // happens in `MousePressed` -> `reconcile_focus_at`.
                self.change_register_value(value);
                self.active_register_target = None;
                self.inline_register_target = None;
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
            }
            Message::RegisterSelected(target) => self.select_register_target(target),
            Message::RegisterEnter(target) => {
                self.enter_inline_register(target);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInlineRegister);
            }
            Message::RefocusInlineRegister => {
                return iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID);
            }
            Message::InlineRegisterValueChanged(target, value) => {
                self.change_inline_register_value(target, value);
                self.focused_input = Some(REGISTER_INLINE_INPUT_ID);
            }
            Message::ApplyInlineRegisterValue(target) => {
                return self.apply_inline_register_value(target, self.keyboard_modifiers.shift());
            }
            Message::RegisterHoverStarted(target) => {
                self.hovered_register_target = Some(target);
            }
            Message::RegisterHoverEnded(target) => {
                if self.hovered_register_target == Some(target) {
                    self.hovered_register_target = None;
                }
            }
            Message::MemorySelected(address) => {
                // Single-click on the row: only move the highlight.
                // Focus stays where it was, so the user does not get
                // dropped into editing mode by an accidental click on
                // the address or command columns. To start editing,
                // they have to click the value column directly or
                // double-click the row.
                self.select_memory(address);
            }
            Message::MemoryEnter(address) => {
                // Either a double-click on the row or a single-click on
                // the value cell — both gestures unambiguously mean
                // "I want to type a new byte here".
                //
                // We can't focus the inline editor synchronously: the
                // very same `ButtonPressed` that triggered this message
                // also fires `Message::MousePressed` from the global
                // `event::listen_with` subscription, which dispatches
                // `reconcile_focus_at(cursor)` and clears focus from
                // every focusable whose bounds don't contain the click
                // point. For double-clicks on the address or command
                // columns the click point is *outside* the inline
                // editor's bounds, so a synchronously-issued
                // `operation::focus` would be promptly undone by the
                // reconcile pass.
                //
                // Bouncing through `RefocusInline` defers the focus to
                // the next update tick, well after the reconcile has
                // run. The cosmetic tracker is set immediately so the
                // shell border updates the same frame.
                self.select_memory(address);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
                return Task::done(Message::RefocusInline);
            }
            Message::RefocusInline => {
                // Deferred follow-up to ArrowUp/ArrowDown inside the
                // inline editor: by the time this message lands the
                // row at the new address has been laid out, so the
                // freshly-spawned `text_input` is in the tree and the
                // focus operation can target it. The cosmetic tracker
                // is already correct since we never changed it during
                // the step.
                return iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID);
            }
            Message::MemoryAddressPrevious => return self.step_memory_address(-1),
            Message::MemoryAddressNext => return self.step_memory_address(1),
            Message::MemoryAddressPageUp => return self.step_memory_address(-16),
            Message::MemoryAddressPageDown => return self.step_memory_address(16),
            Message::ArrowKey(direction) => return self.handle_arrow_key(direction),
            Message::HorizontalArrowKey(direction) => {
                return self.handle_horizontal_arrow_key(direction);
            }
            Message::RegisterCtrlArrowKey(direction) => {
                return self.navigate_inline_register_target(direction);
            }
            Message::MemoryScrolled(offset, viewport_height) => {
                self.memory_viewport_height = viewport_height;
                self.scroll_memory(offset);
                self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::JumpMemoryAddress => {
                if self.keyboard_modifiers.command() {
                    // Ctrl+Enter forward search, Ctrl+Shift+Enter backward.
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    // Alt+Enter from the address field commits the typed
                    // address and jumps the memory list to it (the visible
                    // scroll target).
                    return self.jump_memory_address();
                }
                // Plain Enter / Shift+Enter: stay in the editor, advance or
                // step back the address in the input itself, without
                // scrolling the memory list.
                return self.advance_memory_address(self.keyboard_modifiers.shift());
            }
            Message::MemoryAddressChanged(value) => {
                // See RegisterNameChanged — same rationale. Mirror
                // focus for cosmetic styling, but do not return any
                // focus operation: queued ops would race with later
                // clicks and steal focus from the freshly clicked
                // input.
                self.change_memory_address(value);
                self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
            }
            Message::MemoryValueChanged(value) => {
                self.change_memory_value(value);
                self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
            }
            Message::InlineMemoryValueChanged(address, value) => {
                self.change_inline_memory_value(address, value);
                self.focused_input = Some(MEMORY_INLINE_INPUT_ID);
            }
            Message::ApplyInlineMemoryValue(address) => {
                let backward = self.keyboard_modifiers.shift();
                self.apply_inline_memory_value(address);
                let step = self.step_memory_address(if backward { -1 } else { 1 });
                // The inline editor widget is rebuilt against the new
                // address, which would normally drop focus. Re-focus it
                // here so the user can keep typing the next byte without
                // reaching for the mouse.
                return step.chain(iced::widget::operation::focus(MEMORY_INLINE_INPUT_ID));
            }
            Message::OpcodeDropdownToggled(address) => self.toggle_opcode_dropdown(address),
            Message::OpcodeSearchChanged(value) => self.opcode_search_input = value,
            Message::OpcodeSelected(address, value) => self.select_opcode(address, value),
            Message::OpcodeScrolled => {
                self.opcode_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
            }
            Message::HideOpcodeDropdown => self.hide_opcode_dropdown(),
            Message::DismissErrorNotice => {
                // Drop the floating error overlay. The document state
                // was already preserved by whichever fail-safe early
                // return surfaced the notice, so there is nothing
                // else to undo here — just clearing the field is
                // enough for the next render to hide the frame.
                self.clear_error_notice();
            }
            Message::DismissHaltNotice => {
                // Drop the floating halt-block overlay. The actual
                // halted bit on the CPU is unchanged — the user still
                // has to reset registers before a run/step gesture
                // does anything — but the notice itself is just a UI
                // hint and clearing it on click matches the behaviour
                // of the file-error overlay it now visually mirrors.
                self.clear_halt_notice();
            }
            Message::DismissInfoNotice => {
                // Drop the floating info overlay (currently raised
                // only on legacy-format opens). Purely cosmetic —
                // the document is already loaded and the
                // `current_legacy_snapshot_path` slot already
                // remembers the format choice for the next save —
                // so clearing the field is enough for the next
                // render to hide the frame.
                self.clear_info_notice();
            }
            Message::EscPressed => {
                // Esc closes the current logical edit: any in-flight
                // text coalescing must end here so the next keystroke
                // starts a fresh undo entry rather than continuing
                // whatever the user was just typing into.
                self.undo_stack.break_coalescing();
                // Error notice takes the very first slot in the Esc
                // priority order: a failed open/save is the most
                // recent thing the user is reacting to, and a stray
                // Esc to clear it should not also collapse a modal
                // they're about to confirm or revert an inline edit
                // they're mid-keying. Cheap and additive — clears
                // the field and falls through to the rest of the
                // chain (modal / inline edit / opcode dropdown), so
                // a single Esc keeps doing what it always did when
                // the overlay isn't up.
                if self.error_notice.is_some() {
                    self.clear_error_notice();
                    return Task::none();
                }
                // Halt notice takes the next slot: a fresh halt-block
                // hint is the second-most-recent thing the user is
                // reacting to, and it now wears the same red-bordered
                // frame as the file-error overlay. Esc dismisses it
                // for the same reason — passive notices should clear
                // on the universal "back out" key. Falls through to
                // the modal / inline-edit / dropdown chain so a
                // single Esc keeps doing what it always did when no
                // overlay is up.
                if self.halt_notice.is_some() {
                    self.clear_halt_notice();
                    return Task::none();
                }
                // Info notice takes the next slot, below halt and
                // error: it is purely informational so it should be
                // the easiest of the three overlays to dismiss with
                // Esc, but it must still rank above the modal /
                // inline-edit / dropdown chain the way the other
                // two notices do. Falls through to the rest of the
                // chain when no notice is up.
                if self.info_notice.is_some() {
                    self.clear_info_notice();
                    return Task::none();
                }
                // Top-level menus are transient popup chrome, so Esc
                // should collapse them before it falls through to
                // editor-specific recovery like inline-memory revert
                // or opcode picker dismissal.
                if self.open_menu.is_some() {
                    self.open_menu = None;
                    return Task::none();
                }
                // Pick the gesture by current focus: with the inline
                // memory editor active, Esc reverts the pending edit;
                // any other context falls back to closing the opcode
                // dropdown — the legacy Esc binding. Keeping the
                // routing in `update` (where we can read `self`)
                // avoids leaking state into the `Fn` event listener.
                //
                // Either way iced has consumed the Esc by clearing
                // `state.is_focused` on whatever text_input was
                // focused, so the cosmetic tracker is stale. Chain a
                // `find_focused_optional()` poll onto whatever task
                // the gesture produces; the resolver clears the
                // tracker when iced confirms no focusable owns the
                // caret. The `_optional` variant is what makes the
                // message arrive even when nothing is focused — the
                // built-in `find_focused` returns `Outcome::None` in
                // that case and the message is silently dropped.
                let resolve =
                    iced::advanced::widget::operate(crate::runtime::find_focused_optional())
                        .map(Message::ResolveFocusedTracker);
                if self.focused_input == Some(REGISTER_INLINE_INPUT_ID) {
                    return self.cancel_inline_register_edit().chain(resolve);
                }
                if self.focused_input == Some(MEMORY_INLINE_INPUT_ID) {
                    return self.cancel_inline_memory_edit().chain(resolve);
                }
                self.hide_opcode_dropdown();
                return resolve;
            }
            Message::EnterPressed => {
                // The keyboard subscription only emits this message
                // when the press was `Status::Ignored` — i.e. no
                // focusable consumed it. So there is no need to gate
                // on `self.focused_input` here: any text_input that
                // owned focus would have captured the Enter and
                // routed it through its own `on_submit`
                // (`ApplyMemory`, `ApplyRegister`,
                // `ApplyInlineMemoryValue`, …) before this branch
                // ever ran.
                //
                if let Some(target) = self.active_register_target {
                    return Task::done(Message::RegisterEnter(target));
                }

                // Re-enter inline editing for whichever row the
                // memory address spinner is currently pointing at.
                // The spinner mirrors the highlight, so this is the
                // row the user has been navigating with arrows or
                // PageUp/PageDown. A non-hex address shouldn't
                // happen in normal use (the spinner only renders
                // four hex digits), but we still bail silently
                // rather than panic — the press becomes a no-op the
                // same way a pre-edit click on dead space would.
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                return Task::done(Message::MemoryEnter(address));
            }
            Message::OpenOpcodePicker => {
                // Same gating story as `EnterPressed`: the listener
                // only forwards E when iced reports `Status::Ignored`,
                // so any text input that owned focus has already
                // consumed the keypress (the user was typing the
                // letter, not invoking a shortcut).
                //
                // Toggle the floating opcode picker on the currently
                // selected row and chain a focus task onto its search
                // field. `toggle_opcode_dropdown` is the same path the
                // click on the command column takes, so the visual
                // state is identical — the only added work is the
                // focus task, which is harmless if the dropdown was
                // already open (re-focusing the same widget is a
                // no-op).
                let Some(address) = self.selected_memory_address() else {
                    return Task::none();
                };
                self.toggle_opcode_dropdown(address);
                // If the toggle just *closed* the dropdown (the user
                // pressed E twice), don't bother focusing — the search
                // field is no longer in the tree. Open-state is the
                // post-toggle invariant we need to check.
                if self.opcode_dropdown_address.is_none() {
                    return Task::none();
                }
                self.focused_input = Some(OPCODE_SEARCH_INPUT_ID);
                return iced::widget::operation::focus(OPCODE_SEARCH_INPUT_ID);
            }
            Message::ApplyMemory => {
                if self.keyboard_modifiers.command() {
                    // Ctrl+Enter forward search, Ctrl+Shift+Enter backward.
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                if self.keyboard_modifiers.alt() {
                    // Alt+Enter from the value field writes the byte and
                    // jumps the memory list to the same address.
                    return self.apply_memory_and_jump();
                }
                // Plain Enter / Shift+Enter: behaviour depends on which
                // memory-editor field the user is working in. From the
                // address field we just step the address; from the value
                // field we also commit the byte. Either way focus stays
                // where it was.
                let from_address = self.focused_input == Some(MEMORY_ADDRESS_INPUT_ID);
                let backward = self.keyboard_modifiers.shift();
                if from_address {
                    return self.advance_memory_address(backward);
                }
                return self.apply_memory_and_step(backward);
            }
            Message::ModifiersChanged(modifiers) => {
                self.keyboard_modifiers = modifiers;
            }
            Message::FocusCycle { backward } => {
                // Ask iced for the id of the currently focused widget. If
                // nothing is focused, this resolves to no value and the
                // continuation never fires—exactly what we want, because
                // focusing "the next widget" is meaningless without a
                // starting point.
                use iced::advanced::widget::operation::focusable::find_focused;
                return iced::advanced::widget::operate(find_focused())
                    .map(move |focused| Message::FocusResolved { focused, backward });
            }
            Message::FocusResolved { focused, backward } => {
                return self.cycle_focus(focused, backward);
            }
            Message::WindowOpened(id) => {
                // Cloak immediately, then unhide the window. Because the
                // window is cloaked, DWM never composites the white client
                // area; the user only sees the window once we uncloak it
                // after iced has presented its first real frame.
                //
                // Cache the id so the custom caption buttons (drag /
                // minimise / toggle-maximise / close) can dispatch
                // `iced::window::*` tasks without a `get_latest` round
                // trip per click, and seed the maximised flag so the
                // maximise/restore glyph matches the OS-side state from
                // the very first frame instead of frozen on "maximise".
                //
                // `set_rounded_corners` opts the borderless window into
                // Windows 11's DWM rounded-corner treatment so the user
                // does not see a sharp 90° client area; on other
                // platforms (and on Windows 10) the call is a no-op.
                self.window_id = Some(id);
                return Task::batch([
                    iced::window::run(id, |window| platform::cloak_window(window, true)).discard(),
                    iced::window::run(id, |window| platform::set_rounded_corners(window)).discard(),
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::FrameRendered => {
                if self.startup_frames_seen < u8::MAX {
                    self.startup_frames_seen = self.startup_frames_seen.saturating_add(1);
                }
                // Wait for the second frame so we are certain the wgpu
                // swapchain has produced and presented our content before
                // exposing the window.
                if self.startup_frames_seen == 2 {
                    return iced::window::latest()
                        .and_then(|id| {
                            iced::window::run(id, |window| platform::cloak_window(window, false))
                        })
                        .discard();
                }
            }
            Message::MenuToggled(menu) => {
                // Toggle: clicking the same label twice closes the
                // dropdown, clicking a different label switches to
                // it. Either way the export submenu collapses,
                // because its visibility belongs to whatever
                // top-level menu was open before — once we navigate
                // away, leaving it expanded would resurrect stale
                // state on the next "Файл" click.
                self.open_menu = if self.open_menu == Some(menu) {
                    None
                } else {
                    Some(menu)
                };
            }
            Message::MenuClosed => {
                self.open_menu = None;
            }
            Message::MenuCategoriesToggled => {
                // Flip the bar's "category strip" visibility. When
                // hiding, also collapse any open dropdown so the
                // floating panel can't outlive its trigger label —
                // without this the panel would keep painting over the
                // schematic with nothing visible to dismiss it except
                // the global scrim, which is a worse affordance than
                // a missing trigger.
                self.menu_categories_visible = !self.menu_categories_visible;
                if !self.menu_categories_visible {
                    self.open_menu = None;
                }
            }
            Message::MenuBatch(messages) => {
                // Fan a list of messages out into a `Task::batch` of
                // `Task::done` calls. Iced runs the batched tasks in
                // submission order, which is what lets a menu item
                // close the dropdown first and *then* dispatch its
                // real action — the user never sees the menu linger
                // behind a file dialog or an emulator command.
                let tasks = messages.into_iter().map(Task::done).collect::<Vec<_>>();
                return Task::batch(tasks);
            }
            Message::SpeedTierChanged(tier) => {
                // Stash the tier so the schematic switch reflects the
                // new active segment, then ship two commands to the
                // worker: the inter-instruction delay and the run
                // mode. `tier_hz` resolves Slow / Medium / High to a
                // concrete Hz (the High tier asks the platform for
                // the monitor refresh rate and clamps it); Max
                // bypasses the Hz path entirely and switches the
                // worker to burst mode, so it stops paying the
                // per-instruction timer + crossbeam + redraw cost
                // that made Max indistinguishable from High before.
                self.speed_tier = tier;
                let hz = tier_hz(tier);
                // Convert "instructions per second" to "duration per
                // instruction". The worker floors at 1 ms anyway, so
                // even the highest tier on a 480 Hz panel lands in
                // legal territory.
                let interval = Duration::from_micros(1_000_000 / u64::from(hz.max(1)));
                self.dispatch(k580_app::AppCommand::SetStepInterval(interval));
                // The run mode is what actually unlocks "Максимум":
                // burst mode tells the worker to run instructions in
                // a tight inner loop bounded by `slice` wall-time
                // (16 ms ≈ one display frame, which keeps Stop
                // responsive within the same window iced uses for
                // its own redraw). Paced mode for the other tiers
                // keeps the original behaviour — one snapshot per
                // step, the highlighted memory row walks one cell at
                // a time.
                let mode = match tier {
                    SpeedTier::Max => k580_app::RunMode::Burst {
                        slice: Duration::from_millis(16),
                    },
                    _ => k580_app::RunMode::Paced,
                };
                self.dispatch(k580_app::AppCommand::SetRunMode(mode));
            }
            Message::WindowDragStart => {
                if self.close_titlebar_popup_before_drag() {
                    return Task::none();
                }
                // Hand the press over to the OS so it can run its
                // native drag loop on the borderless window. iced
                // proxies the call straight to winit's
                // `drag_window`, which only succeeds while a left
                // button is currently pressed — perfect fit for the
                // `mouse_area::on_press` we wire this to.
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::drag(id);
            }
            Message::WindowMinimize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::minimize(id, true);
            }
            Message::WindowToggleMaximize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                // Optimistic flip first so the caption glyph swaps
                // immediately on click; the trailing `is_maximized`
                // poll reconciles the flag against the OS-side
                // result in case the toggle was refused (e.g. the
                // window manager blocked maximisation).
                self.window_maximized = !self.window_maximized;
                return Task::batch([
                    iced::window::toggle_maximize(id),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowClose => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                return iced::window::close(id);
            }
            Message::WindowMaximizedChanged(maximized) => {
                self.window_maximized = maximized;
            }
            Message::Undo => return self.apply_undo(),
            Message::Redo => return self.apply_redo(),
            Message::ConfirmDiscard => {
                // User accepted the loss of in-flight edits. Read the
                // queued action, clear it, and re-emit the original
                // gesture as a fresh message. Each gated entry point
                // re-checks `dirty`; we wipe the flag *before*
                // dispatching so the second pass falls through to the
                // real handler instead of bouncing back into the
                // modal. `CloseWindow` is not gated, so we dispatch
                // `WindowClose` directly.
                return self.confirm_discard();
            }
            Message::CancelDiscard => {
                // User backed out of the destructive gesture. Drop
                // the queued action and leave the document untouched
                // — the modal disappears on the next frame because
                // `view` only paints it when `pending_action.is_some()`.
                self.cancel_discard();
            }
            Message::WindowCloseRequested => {
                // The OS has asked the window to close (× caption
                // button, Alt+F4, taskbar). With
                // `exit_on_close_request(false)` set on the
                // application iced no longer auto-closes; we route
                // the request through the dirty gate exactly like
                // the other discard-paths. The clean path falls
                // through to `WindowClose`, which dispatches
                // `iced::window::close` for the cached window id.
                if self.dirty {
                    self.open_discard_modal(PendingAction::CloseWindow);
                } else {
                    return Task::done(Message::WindowClose);
                }
            }
        }
        Task::none()
    }

    /// Inline logic of "Файл → Новый файл". Lifted out of the
    /// `Message::NewFile` handler so the dirty-gate path and the
    /// `ConfirmDiscard` re-entry can share the implementation —
    /// confirming the modal re-emits `Message::NewFile` through
    /// `Task::done`, which then runs the same wipe sequence as a
    /// direct gesture from a clean document.
    fn run_new_file(&mut self) {
        // Wipe RAM and registers in one shot. Order matters
        // less than it looks because both reset commands fan
        // out to the worker thread serially, but we send RAM
        // first so the snapshot the user sees on the next
        // tick is consistent with "blank slate, PC at 0".
        self.dispatch(k580_app::AppCommand::ResetRam);
        self.dispatch(k580_app::AppCommand::ResetCpu);
        self.running = false;
        // Drop the remembered snapshot path: a "new file" has
        // no associated path on disk, so the next "Сохранить"
        // must prompt for one — same behaviour as every text
        // editor.
        self.current_snapshot_path = None;
        // The legacy path is part of the previous document's
        // identity too — wiping it here keeps "Новый файл" a true
        // blank slate: the next Ctrl+Alt+S must prompt for a fresh
        // location rather than overwrite a legacy file from before
        // the new-file gesture.
        self.current_legacy_snapshot_path = None;
        // The user explicitly asked for a blank slate; the
        // pre-NewFile timeline has nothing to anchor onto in
        // the new document, so the cleanest mental model is
        // "history starts here". Wiping both stacks also
        // prevents Ctrl+Z from rewinding past the new-file
        // boundary into RAM/registers that no longer exist
        // on screen.
        self.undo_stack.clear();
        // Blank slate is also a "clean baseline" for the
        // dirty flag: the user explicitly asked for a fresh
        // document, so no further close/open gesture should
        // prompt until they make their first edit.
        self.dirty = false;
        self.status = "Новый файл".to_owned();
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        // The Tick subscription is what drives `pull_events` — every
        // fire pulls *all* worker events queued since the last fire and
        // folds them into the snapshot in one go. That has a subtle
        // consequence for the paced run loop: at 100 ms-per-tick a
        // 50 Hz run produces five `StateChanged` per Tick, and only
        // the last one survives in the highlighted row because the
        // earlier four get overwritten before iced has a chance to
        // redraw. The user reads this as "PC skips lines instead of
        // walking them."
        //
        // Couple the Tick cadence to the active speed tier while
        // running so each worker step gets its own redraw. The
        // `tier_hz` resolver already returns a "displayable" Hz —
        // Slow / Medium are constants below 60, High is the monitor
        // refresh rate clamped to a usable ceiling — so we just turn
        // the tier's Hz into a millisecond period, with a 16 ms floor
        // (≈60 Hz) so a future bump to e.g. a 240 Hz panel still
        // lands in territory iced can actually redraw without losing
        // a frame to event-drain overhead.
        //
        // While paused we go back to 100 ms so the UI stays
        // responsive to manual gestures (step, edits, etc.) without
        // waking the runtime at refresh rate for nothing.
        let tick_interval = if self.running {
            let hz = u64::from(tier_hz(self.speed_tier).max(1));
            let raw_ms = (1000_u64 / hz).max(16);
            Duration::from_millis(raw_ms.min(100))
        } else {
            Duration::from_millis(100)
        };
        let mut subscriptions = vec![
            time::every(tick_interval).map(|_| Message::Tick),
            iced::window::open_events().map(Message::WindowOpened),
            event::listen_with(|event, status, _window| match (event, status) {
                (iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)), _) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Escape),
                        ..
                    }),
                    _,
                ) => Some(Message::EscPressed),
                // File-menu shortcuts. Match Ctrl-modified character keys
                // *before* the Tab/arrow handlers and *unconditionally*
                // (no `Status::Ignored` filter): we want Ctrl+S to save
                // even when a `text_input` has focus, otherwise the user
                // has to click out of every input first. We translate via
                // `to_latin(physical_key)` so a Russian keyboard layout
                // — where `н` sits on the physical N key — still resolves
                // to `Some('n')` and fires the same shortcut.
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        physical_key,
                        modifiers,
                        ..
                    }),
                    _,
                ) if modifiers.command() => {
                    if let Some(direction) = register_inline::ctrl_arrow_move(&key, modifiers) {
                        return Some(Message::RegisterCtrlArrowKey(direction));
                    }

                    let latin = key.to_latin(physical_key)?;
                    // The match arms read `(letter, shift, alt)`. Alt is
                    // tracked explicitly so the legacy save/open
                    // shortcuts (Ctrl+Alt+S / Ctrl+Alt+O) can sit
                    // alongside their plain Ctrl twins without
                    // colliding: pressing Ctrl+S with Alt held must
                    // NOT also fire `SaveSnapshot`, otherwise the user
                    // would silently get *both* a v1 save and the
                    // legacy save dialog on the same keystroke.
                    let alt = modifiers.alt();
                    match (latin, modifiers.shift(), alt) {
                        ('n', false, false) => Some(Message::NewFile),
                        ('o', false, false) => Some(Message::OpenSnapshot),
                        ('s', false, false) => Some(Message::SaveSnapshot),
                        ('s', true, false) => Some(Message::SaveSnapshotAs),
                        ('i', false, false) => Some(Message::Import),
                        ('e', false, false) => Some(Message::Export),
                        // Legacy `.580` shortcuts. Ctrl+Alt picks the
                        // legacy twins of S/O — same key the user
                        // already associates with save/open, the Alt
                        // bit signals "the niche format" the way Shift
                        // signals "save as" elsewhere. No legacy
                        // counterparts for N/I/E because legacy is a
                        // file format, not a workflow.
                        ('s', false, true) => Some(Message::SaveLegacySnapshot),
                        ('o', false, true) => Some(Message::OpenLegacySnapshot),
                        // МП-Система. Ctrl+letter for the three execution
                        // gestures (R = Run, T = sTep instruction — "S"
                        // is taken by Save, T is the natural next pick;
                        // Y sits next to T on both QWERTY and ЙЦУКЕН so
                        // "step instruction → step tact" reads as a
                        // finer-grained variant of the same gesture).
                        // Ctrl+Shift+letter for the destructive resets:
                        // capitalised intuition + a guaranteed-not-while-
                        // typing modifier on RAM/registers wipes. R doubles
                        // as "Run" and "Reset RAM" without colliding because
                        // the Shift bit picks the destructive twin, the
                        // same way Save / Save As share the S key.
                        // Ctrl+Shift+G mirrors the action panel's "Сброс
                        // регистров" button — both dispatch `ResetCpu`,
                        // which per `prompt/09_quality_gates.md` is the
                        // single "clean power-on" gesture: registers,
                        // PC, SP, interrupt state, halt, **and**
                        // cycle_count. There is no separate "registers
                        // only" semantic in the spec.
                        ('r', false, false) => Some(Message::ToggleRun),
                        ('t', false, false) => Some(Message::StepInstruction),
                        ('y', false, false) => Some(Message::StepTact),
                        ('r', true, false) => Some(Message::ResetRam),
                        ('g', true, false) => Some(Message::ResetCpu),
                        // Ctrl+Shift+H — clear the halt flip-flop
                        // without touching anything else (the
                        // weakest reset gesture in the menu, and
                        // the only thing that lifts the post-HLT
                        // run-block latch on its own without
                        // wiping registers/PC/RAM). Same Shift-bit
                        // story as ResetRam / ResetCpu: capitalised
                        // intuition for "this changes execution
                        // state", and a guaranteed-not-while-typing
                        // modifier so a stray Ctrl+H in a text
                        // input cannot silently flip the halt bit
                        // out from under the user.
                        ('h', true, false) => Some(Message::ClearHalt),
                        // Undo / redo. Bound unconditionally (no
                        // `Status::Ignored` filter) for the same reason
                        // every editor binds them this way: iced's
                        // `text_input` does not implement undo, so a
                        // user typing into the value column expects
                        // Ctrl+Z to roll the buffer back even though
                        // the input "owns" the keystroke. We swallow
                        // it here before iced has a chance to ignore
                        // it. The shared stack means the same shortcut
                        // also rewinds CPU mutations (ResetCpu /
                        // ResetRam / SetMemory / snapshot loads) when
                        // no input is focused — one timeline, one
                        // mental model.
                        ('z', false, false) => Some(Message::Undo),
                        ('z', true, false) => Some(Message::Redo),
                        _ => None,
                    }
                }
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Tab),
                        modifiers,
                        ..
                    }),
                    iced::event::Status::Ignored,
                ) => Some(Message::FocusCycle {
                    backward: modifiers.shift(),
                }),
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }),
                    iced::event::Status::Ignored,
                ) => match key {
                    // ArrowUp/ArrowDown are routed by the message handler:
                    // the destination depends on which input owns focus and
                    // we don't want to read app state from inside the
                    // (Fn, not FnMut) listener closure.
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        Some(Message::ArrowKey(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        Some(Message::ArrowKey(-1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        Some(Message::HorizontalArrowKey(-1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        Some(Message::HorizontalArrowKey(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        Some(Message::MemoryAddressPageUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        Some(Message::MemoryAddressPageDown)
                    }
                    // Enter outside any text input: the iced runtime
                    // reports the press as `Status::Ignored` because no
                    // focusable claimed it. Route it through a dedicated
                    // message so the update handler can re-enter inline
                    // editing on the currently selected memory row —
                    // recovers from Esc / dead-space click without
                    // forcing the user back to the mouse.
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Some(Message::EnterPressed)
                    }
                    // Bare E (no modifiers) outside any text input:
                    // open the floating opcode picker on the current
                    // row and focus its search field. Same gating
                    // story as Enter — `Status::Ignored` means no
                    // input owned the keystroke, so the user is not
                    // trying to type the letter into an editor. The
                    // Ctrl+E shortcut for "Экспорт" is handled by the
                    // earlier `modifiers.command()` arm and does not
                    // reach here. We deliberately do not check
                    // `modifiers` again: the previous branch already
                    // matched anything with Ctrl/Cmd, so by the time
                    // we land in `Status::Ignored` it is plain E,
                    // Shift+E, AltGr+E, etc. — all of which the user
                    // would expect to drop them into the picker, and
                    // none of which produces a printable character
                    // that needs preserving (no input is focused).
                    keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("e") => {
                        Some(Message::OpenOpcodePicker)
                    }
                    _ => None,
                },
                // Track the cursor on every move regardless of whether
                // a widget captured the event — we need the latest
                // position cached so the next `ButtonPressed` knows
                // where the click landed. CursorMoved events fire
                // continuously during dragging, but the message
                // handler is a single field write so the cost is
                // negligible.
                (iced::Event::Mouse(mouse::Event::CursorMoved { position }), _) => {
                    Some(Message::CursorMoved(position))
                }
                // Fire reconciliation on every left mouse press,
                // regardless of capture status. Listening to captured
                // presses is the whole point: when text_input::update
                // captures a press inside panel A's input, the column
                // still propagates to panel B's stack, but B's stack
                // bails out and B's text_inputs never see the click.
                // The reconcile pass walks the tree from the outside
                // and clears every focusable not under the cursor,
                // fixing whatever stale state the broken propagation
                // left behind.
                (iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), _) => {
                    Some(Message::MousePressed)
                }
                // OS-side close request (× caption, Alt+F4, taskbar
                // close). With `exit_on_close_request(false)` set on
                // the application, iced forwards the request as a
                // `window::Event::CloseRequested` instead of closing
                // the window itself. We turn it into a message the
                // update handler can route through the dirty gate.
                (iced::Event::Window(iced::window::Event::CloseRequested), _) => {
                    Some(Message::WindowCloseRequested)
                }
                _ => None,
            }),
        ];

        // Only listen to frame events while we are still cloaked. Once the
        // window is uncloaked there is nothing more to do, and iced docs warn
        // that the rate of `frames()` matches the display refresh rate.
        if self.startup_frames_seen < 2 {
            subscriptions.push(iced::window::frames().map(|_| Message::FrameRendered));
        }

        Subscription::batch(subscriptions)
    }

    /// Routes ArrowUp/ArrowDown to whichever editor currently owns focus.
    /// `direction` is `+1` for ArrowUp and `-1` for ArrowDown, matching
    /// the convention "up increments, down decrements" used by numeric
    /// byte fields. With nothing tracked focused we fall back to memory
    /// list navigation, which is the legacy app-wide shortcut.
    fn handle_arrow_key(&mut self, direction: i32) -> Task<Message> {
        match self.focused_input {
            Some(REGISTER_NAME_INPUT_ID) => {
                // ArrowUp moves to the register listed *above* the current
                // one in `REGISTER_ORDER`, which means stepping by `-1`.
                self.step_register(-direction);
                Task::none()
            }
            Some(REGISTER_VALUE_INPUT_ID) => {
                self.step_register_value_input(direction);
                Task::none()
            }
            Some(REGISTER_INLINE_INPUT_ID) => {
                self.step_register_value_input(direction);
                iced::widget::operation::focus(REGISTER_INLINE_INPUT_ID)
            }
            Some(MEMORY_VALUE_INPUT_ID) => {
                self.step_memory_value_input(direction);
                Task::none()
            }
            Some(MEMORY_INLINE_INPUT_ID) => {
                // Stepping the memory address moves the highlight onto a
                // different row, which means iced drops the inline
                // `text_input` from the row that was selected and
                // spawns a fresh one with the same id under the new
                // row. Chaining `operation::focus` directly here would
                // run before the rebuild, so the focus would land on
                // the widget that is about to disappear and the caret
                // would vanish.
                //
                // Bouncing through a `RefocusInline` message defers the
                // focus operation to the next update tick: by then the
                // new row is laid out, the new `text_input` is in the
                // tree, and `operation::focus(MEMORY_INLINE_INPUT_ID)`
                // hits it. The cosmetic `focused_input` tracker is
                // already pointing at this id, so we leave it alone.
                //
                // We also call `step_memory_address_browse` instead of
                // `step_memory_address`: the latter goes through
                // `select_memory -> sync_pc_to_cursor -> dispatch_sync`,
                // which blocks on a worker round-trip. The blocking
                // path was eating focus on the inline editor (the
                // `StateChanged` event came back synchronously in the
                // middle of the handler and the resulting view rebuild
                // landed before our `Task::done(RefocusInline)` made it
                // out the door). The browse-mode step keeps PC
                // untouched and updates only the spinner / inline
                // value so the row swap is purely cosmetic.
                let scroll = self.step_memory_address_browse(-direction);
                scroll.chain(Task::done(Message::RefocusInline))
            }
            None if self.active_register_target.is_some() => {
                let movement = if direction > 0 {
                    RegisterMove::Up
                } else {
                    RegisterMove::Down
                };
                self.navigate_active_register_target(movement);
                Task::none()
            }
            // Memory address field and "no focus" both fall through to
            // memory navigation: stepping the address there *is* what the
            // user wants, and the unfocused case keeps the legacy global
            // shortcut.
            _ => self.step_memory_address(-direction),
        }
    }

    fn handle_horizontal_arrow_key(&mut self, direction: i32) -> Task<Message> {
        if self.focused_input.is_none() && self.active_register_target.is_some() {
            let movement = if direction < 0 {
                RegisterMove::Left
            } else {
                RegisterMove::Right
            };
            self.navigate_active_register_target(movement);
        }
        Task::none()
    }
}
