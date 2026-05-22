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

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID,
    parse_register_name, register_name,
};
pub(crate) use messages::{MenuId, Message};

use iced::{Point, event, keyboard, mouse, time};
use iced::{Subscription, Task, Theme};
use k580_app::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use k580_core::RegisterName;
use std::path::PathBuf;
use std::time::Duration;

use crate::platform;

/// Default speed of the paced `Run` loop, in instructions per second.
/// 10 Hz mirrors `k580_app::DEFAULT_STEP_INTERVAL` (100 ms between
/// instructions) — slow enough that a human eye can follow each PC
/// update, fast enough that a 50-instruction program still finishes
/// in roughly five seconds.
pub(crate) const DEFAULT_STEP_HZ: u32 = 10;

/// Lower bound exposed by the speed slider. One instruction per
/// second is the slowest "still feels alive" pace; anything below
/// that and the user starts wondering whether the emulator has
/// hung. The worker enforces a `1 ms` floor independently, so this
/// only constrains what the slider can produce.
pub(crate) const MIN_STEP_HZ: u32 = 1;

/// Upper bound exposed by the speed slider. 1000 Hz keeps a 50-step
/// program flickering past in well under a second; pushing it any
/// higher just floods the UI with redraws faster than the eye can
/// see anything change. Users who want raw throughput can still call
/// `RunForTStates` from the menu, which bypasses the pacing entirely.
pub(crate) const MAX_STEP_HZ: u32 = 1000;

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) status: String,
    pub(crate) selected_register: RegisterName,
    pub(crate) register_name_input: String,
    pub(crate) register_value_input: String,
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
    /// Speed of the paced `Run` loop in instructions per second.
    /// Bound to the slider on the left-hand schematic panel; the
    /// `SpeedChanged` handler turns it into a `Duration` and ships
    /// `AppCommand::SetStepInterval` to the worker. Default matches
    /// `k580_app::DEFAULT_STEP_INTERVAL`.
    pub(crate) step_hz: u32,
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
                status: "Ready".to_owned(),
                selected_register: RegisterName::A,
                register_name_input: "A".to_owned(),
                register_value_input: "00".to_owned(),
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
                // 10 instructions per second matches
                // `DEFAULT_STEP_INTERVAL = 100 ms` over in `k580_app`.
                // Slow enough that the eye can follow each PC update,
                // fast enough that a 50-instruction program still
                // finishes in five seconds.
                step_hz: DEFAULT_STEP_HZ,
                halt_notice: None,
            },
            startup_task,
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.pull_events();
                self.memory_scroll_visible_ticks =
                    self.memory_scroll_visible_ticks.saturating_sub(1);
                self.opcode_scroll_visible_ticks =
                    self.opcode_scroll_visible_ticks.saturating_sub(1);
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
                const TRACKED: [&str; 5] = [
                    MEMORY_ADDRESS_INPUT_ID,
                    MEMORY_VALUE_INPUT_ID,
                    REGISTER_NAME_INPUT_ID,
                    REGISTER_VALUE_INPUT_ID,
                    MEMORY_INLINE_INPUT_ID,
                ];

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
            Message::ResetCpu => self.dispatch(k580_app::AppCommand::ResetCpu),
            Message::ResetRam => self.dispatch(k580_app::AppCommand::ResetRam),
            Message::OpenSnapshot => self.open_snapshot(),
            Message::LoadSnapshotFromPath(path) => self.load_snapshot_from_path(path),
            Message::SaveSnapshot => self.save_snapshot(),
            Message::SaveSnapshotAs => self.save_snapshot_as(),
            Message::NewFile => {
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
                self.status = "Новый файл".to_owned();
            }
            Message::Export => self.export_file(),
            Message::Import => self.import_file(),
            Message::RegisterSelected(register) => self.select_register(register),
            Message::RegisterNameChanged(value) => {
                // Mirror focus into our cosmetic tracker so the shell
                // border updates the same frame the user starts typing.
                // `MousePressed` -> `reconcile_focus_at` already does
                // this on click; this write covers the case where the
                // user reaches the field via Tab and starts typing
                // before the next click event arrives.
                self.change_register_name(value);
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
                self.focused_input = Some(REGISTER_VALUE_INPUT_ID);
            }
            Message::ApplyRegister => {
                if self.keyboard_modifiers.command() {
                    return self
                        .find_next_memory_address_in_direction(self.keyboard_modifiers.shift());
                }
                return self.apply_register_and_step(self.keyboard_modifiers.shift());
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
            Message::EscPressed => {
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
                if self.focused_input == Some(MEMORY_INLINE_INPUT_ID) {
                    return self.cancel_inline_memory_edit().chain(resolve);
                }
                self.hide_opcode_dropdown();
                return resolve;
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
                return Task::batch([
                    iced::window::run(id, |window| platform::cloak_window(window, true)).discard(),
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
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
            Message::SpeedChanged(hz) => {
                // Clamp defensively even though the slider already
                // constrains the range — keeps the field honest for
                // future call sites (e.g. a hex-typed speed input).
                let hz = hz.clamp(MIN_STEP_HZ, MAX_STEP_HZ);
                self.step_hz = hz;
                // Convert "instructions per second" to "duration per
                // instruction". The worker floors at 1 ms, so even
                // the maximum slider value lands in legal territory.
                let interval = Duration::from_micros(1_000_000 / u64::from(hz));
                self.dispatch(k580_app::AppCommand::SetStepInterval(interval));
            }
        }
        Task::none()
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            time::every(Duration::from_millis(100)).map(|_| Message::Tick),
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
                    let latin = key.to_latin(physical_key)?;
                    match (latin, modifiers.shift()) {
                        ('n', false) => Some(Message::NewFile),
                        ('o', false) => Some(Message::OpenSnapshot),
                        ('s', false) => Some(Message::SaveSnapshot),
                        ('s', true) => Some(Message::SaveSnapshotAs),
                        ('i', false) => Some(Message::Import),
                        ('e', false) => Some(Message::Export),
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
                        ('r', false) => Some(Message::ToggleRun),
                        ('t', false) => Some(Message::StepInstruction),
                        ('y', false) => Some(Message::StepTact),
                        ('r', true) => Some(Message::ResetRam),
                        ('g', true) => Some(Message::ResetCpu),
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
                    keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        Some(Message::MemoryAddressPageUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        Some(Message::MemoryAddressPageDown)
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
            // Memory address field and "no focus" both fall through to
            // memory navigation: stepping the address there *is* what the
            // user wants, and the unfocused case keeps the legacy global
            // shortcut.
            _ => self.step_memory_address(-direction),
        }
    }
}
