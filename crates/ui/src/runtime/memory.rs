//! Memory list, address spinner, inline editor, and the address-pattern
//! search — every method here lives on `DesktopApp` and is grouped by the
//! "memory editing" responsibility.

use crate::app::{
    DesktopApp, MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_ROW_HEIGHT,
    MEMORY_SCROLL_VISIBLE_TICKS, MEMORY_VALUE_INPUT_ID, Message,
};
use iced::Task;
use iced::widget::operation;
use k580_app::AppCommand;

use super::parse::{
    bounded_hex_input, parse_hex_u8, parse_hex_u16, saturating_step_u8, scroll_memory_to,
};

impl DesktopApp {
    pub(crate) fn select_memory(&mut self, address: u16) {
        self.active_register_target = None;
        self.inline_register_target = None;
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.set_memory_address(address);
    }

    /// Reads the currently selected memory address from the spinner
    /// input. The spinner mirrors the highlight, so the returned
    /// value is what every "act on the selected row" gesture should
    /// target — `EnterPressed` recovering inline editing after Esc,
    /// for example. Returns `None` when the field somehow holds a
    /// non-hex value; callers fall back to a no-op rather than
    /// guessing an address.
    pub(crate) fn selected_memory_address(&self) -> Option<u16> {
        parse_hex_u16(&self.memory_address_input).ok()
    }

    pub(crate) fn step_memory_address(&mut self, delta: i32) -> Task<Message> {
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = if delta.is_negative() {
            address.saturating_sub((-delta) as u16)
        } else {
            address.saturating_add(delta as u16)
        };
        self.select_memory(next);

        if self.memory_viewport_height <= 0.0 {
            // Viewport size unknown yet (no MemoryScrolled has fired). Skip
            // scrolling and leave the highlight where it is; iced will
            // report the viewport on the first scroll event.
            return Task::none();
        }

        let Some(target_offset) = self.scroll_offset_to_reveal(next) else {
            return Task::none();
        };

        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    /// Same as [`step_memory_address`] but skips the `SetPc` round-trip
    /// to the worker thread. Used by ArrowUp/ArrowDown inside the
    /// inline byte editor, where the user's mental model is "I'm
    /// browsing memory, the program counter has nothing to do with
    /// it" — and where the synchronous `dispatch_sync(SetPc)` inside
    /// `select_memory` was making the inline `text_input` lose focus
    /// on every keystroke. The address spinner remains the source of
    /// truth for the highlighted row, so callers that *do* want PC to
    /// follow the cursor (a single click, an Enter press) keep
    /// reaching for `select_memory`.
    pub(crate) fn step_memory_address_browse(&mut self, delta: i32) -> Task<Message> {
        let address = parse_hex_u16(&self.memory_address_input).unwrap_or(0);
        let next = if delta.is_negative() {
            address.saturating_sub((-delta) as u16)
        } else {
            address.saturating_add(delta as u16)
        };

        // Mirror the cosmetic side of `select_memory` without the
        // worker round-trip: clear the opcode dropdown context, write
        // the new address into the spinner, and refresh the inline
        // editor's value so the freshly-rendered row shows the byte
        // that lives there.
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        self.memory_search_pattern = None;

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }

        let Some(target_offset) = self.scroll_offset_to_reveal(next) else {
            return Task::none();
        };

        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    pub(crate) fn scroll_memory(&mut self, offset: f32) {
        self.memory_scroll_offset = offset.max(0.0);
        self.memory_scroll_first_row = (self.memory_scroll_offset / MEMORY_ROW_HEIGHT)
            .floor()
            .clamp(0.0, u16::MAX as f32) as u16;
    }

    pub(crate) fn change_memory_address(&mut self, value: String) {
        let Some(value) = bounded_hex_input(&value, 4) else {
            return;
        };

        // The user is editing the address inline; the previous Ctrl+Enter
        // search context is no longer relevant.
        self.memory_search_pattern = None;
        // Snapshot the buffer *before* the keystroke lands so Ctrl+Z
        // can rewind the field one logical edit at a time. The stack
        // coalesces consecutive same-field pushes, so a four-keystroke
        // address still collapses to a single entry.
        let before = self.memory_address_input.clone();
        self.memory_address_input = value;
        self.undo_stack.push_text(
            MEMORY_ADDRESS_INPUT_ID,
            before,
            self.memory_address_input.clone(),
        );
        if let Ok(address) = parse_hex_u16(&self.memory_address_input) {
            self.refresh_memory_value(address);
            self.sync_pc_to_cursor(address);
        }
    }

    pub(crate) fn change_memory_value(&mut self, value: String) {
        if let Some(value) = bounded_hex_input(&value, 2) {
            let before = self.memory_value_input.clone();
            self.memory_value_input = value;
            self.undo_stack.push_text(
                MEMORY_VALUE_INPUT_ID,
                before,
                self.memory_value_input.clone(),
            );
        }
    }

    /// Bumps the byte buffered in the memory cell editor by `delta`,
    /// saturating at `0x00`/`0xFF`. Same contract as
    /// `step_register_value_input`: nothing is written to memory until
    /// the user explicitly presses Enter.
    pub(crate) fn step_memory_value_input(&mut self, delta: i32) {
        let current = parse_hex_u8(&self.memory_value_input).unwrap_or(0);
        let next = saturating_step_u8(current, delta);
        self.memory_value_input = format!("{next:02X}");
    }

    pub(crate) fn change_inline_memory_value(&mut self, address: u16, value: String) {
        let Some(value) = bounded_hex_input(&value, 2) else {
            return;
        };

        self.memory_address_input = format!("{address:04X}");
        self.memory_inline_value_input = value;
        // No text-undo entry here on purpose. The inline buffer is a
        // *floating* field: it follows the highlighted address, and
        // `apply_snapshot` / `select_memory` overwrite it whenever
        // the cursor moves. A text entry tied to this id would have
        // its `before`/`after` interpreted against whichever address
        // the buffer happened to be pointing at when Ctrl+Z fires —
        // i.e. typing "12" into address 0x0010, walking to 0x0020,
        // and pressing Ctrl+Z would either be a visual no-op (if the
        // buffer already shows 0x0020's stored byte) or worse, write
        // "12" into the new address on Ctrl+Shift+Z. The actual byte
        // mutation reaches the undo stack as a `Cpu` pair when the
        // user presses Enter (`apply_inline_memory_value`) or when
        // save/export auto-commits the pending byte
        // (`commit_pending_inline_edit`), so nothing is lost — the
        // undo timeline only ever sees the committed state, which is
        // the only state Ctrl+Z can meaningfully target here.
    }

    pub(crate) fn apply_inline_memory_value(&mut self, address: u16) {
        match parse_hex_u8(&self.memory_inline_value_input) {
            Ok(value) => {
                self.memory_address_input = format!("{address:04X}");
                self.memory_value_input = format!("{value:02X}");
                self.memory_inline_value_input = self.memory_value_input.clone();
                // Pressing Enter is the gesture that commits the
                // pending byte to RAM — that's the moment Ctrl+Z
                // should snap the worker back, so capture before/after
                // around the dispatch. The text-side coalescing is
                // also broken here: the next keystroke into an inline
                // editor starts a fresh entry rather than glueing onto
                // the keystrokes that produced this byte.
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
            }
            Err(error) => self.status = error,
        }
    }

    /// Esc handler for the inline memory value editor. Restores the
    /// buffer to the byte that actually lives at the currently
    /// highlighted address — discarding whatever the user has typed
    /// but not yet committed with Enter. Mirrors the standard
    /// "Esc reverts an unsaved edit" affordance every desktop text
    /// field implements.
    ///
    /// Three values move in lockstep:
    ///
    /// * `memory_inline_value_input` — what the inline `text_input`
    ///   on the highlighted row renders; this is the visible buffer
    ///   the user has been typing into.
    /// * `memory_value_input` — the spinner-side mirror, kept in sync
    ///   by `change_inline_memory_value` so the side-panel cell editor
    ///   shows the same byte. Reverting only the inline buffer would
    ///   leave the side panel out of sync until the next selection
    ///   change.
    /// * `memory_address_input` — already correct (Esc does not move
    ///   the cursor), but we re-derive the address from it because the
    ///   inline editor only fires `ApplyInlineMemoryValue(address)` /
    ///   `InlineMemoryValueChanged(address, …)` with a known address
    ///   and `cancel_inline_memory_edit` lacks that parameter. With an
    ///   unparseable address (cannot happen in normal use) we bail
    ///   and refocus without touching the buffers.
    ///
    /// We refocus the inline editor explicitly: although the
    /// `text_input` is rebuilt around the same id every frame, an
    /// extra focus task here defends against the case where Esc was
    /// fired from the global keyboard subscription before iced has
    /// resolved the inline editor as the focus owner — we want the
    /// caret to stay where the user expects it.
    pub(crate) fn cancel_inline_memory_edit(&mut self) -> Task<Message> {
        let Ok(address) = parse_hex_u16(&self.memory_address_input) else {
            return operation::focus(crate::app::MEMORY_INLINE_INPUT_ID);
        };
        let stored = format!("{:02X}", self.snapshot.cpu.memory.read(address));
        // No-op if the buffer already matches storage: nothing to
        // revert and we should leave focus alone — rebuilding the
        // `text_input` would just snap the caret to the end.
        if self.memory_inline_value_input.eq_ignore_ascii_case(&stored) {
            return Task::none();
        }
        self.memory_inline_value_input = stored.clone();
        self.memory_value_input = stored;
        operation::focus(crate::app::MEMORY_INLINE_INPUT_ID)
    }

    pub(crate) fn toggle_opcode_dropdown(&mut self, address: u16) {
        if self.opcode_dropdown_address == Some(address) {
            self.opcode_dropdown_address = None;
            self.opcode_search_input.clear();
            return;
        }

        self.set_memory_address(address);
        self.opcode_dropdown_address = Some(address);
    }

    pub(crate) fn select_opcode(&mut self, address: u16, value: u8) {
        self.memory_address_input = format!("{address:04X}");
        self.memory_value_input = format!("{value:02X}");
        self.memory_inline_value_input = self.memory_value_input.clone();
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        // Picking an opcode commits a byte to RAM — same undo
        // contract as pressing Enter in the inline editor.
        self.undo_stack.break_coalescing();
        self.dispatch_with_undo(AppCommand::SetMemory(address, value));
    }

    pub(crate) fn hide_opcode_dropdown(&mut self) {
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
    }

    /// Writes the typed byte to the typed address. Does not scroll the
    /// memory list — callers that want the user moved to the new row must
    /// chain a scroll task themselves (see `apply_memory_and_jump`).
    pub(crate) fn apply_memory(&mut self) -> Task<Message> {
        match (
            parse_hex_u16(&self.memory_address_input),
            parse_hex_u8(&self.memory_value_input),
        ) {
            (Ok(address), Ok(value)) => {
                self.memory_inline_value_input = format!("{value:02X}");
                // Same contract as `apply_inline_memory_value`: Enter
                // is the commit gesture, so the worker dispatch is
                // wrapped in an undo entry and any in-flight text
                // coalescing is closed.
                self.undo_stack.break_coalescing();
                self.dispatch_with_undo(AppCommand::SetMemory(address, value));
                Task::none()
            }
            (Err(error), _) | (_, Err(error)) => {
                self.status = error;
                Task::none()
            }
        }
    }

    /// Plain Enter handler for the memory cell editor: writes the byte,
    /// then advances/steps back the address input. The memory list is not
    /// scrolled — Alt+Enter is the explicit "jump to this row" shortcut.
    /// Focus stays on the value field so the user can keep typing the
    /// next byte without reaching for the mouse.
    pub(crate) fn apply_memory_and_step(&mut self, backward: bool) -> Task<Message> {
        let write = self.apply_memory();
        self.step_address_in_input(backward);
        self.focused_input = Some(MEMORY_VALUE_INPUT_ID);
        write.chain(operation::focus(MEMORY_VALUE_INPUT_ID))
    }

    /// Alt+Enter handler for the memory value field: writes the byte and
    /// jumps the memory list to the same address.
    pub(crate) fn apply_memory_and_jump(&mut self) -> Task<Message> {
        let write = self.apply_memory();
        let jump = self.jump_memory_address();
        write.chain(jump)
    }

    pub(crate) fn jump_memory_address(&mut self) -> Task<Message> {
        match parse_hex_u16(&self.memory_address_input) {
            Ok(address) => {
                self.refresh_memory_value(address);
                // Only scroll if the target row is not already on screen,
                // so Alt+Enter on a visible address keeps the list still
                // instead of snapping the row to the top.
                if let Some(target_offset) = self.scroll_offset_to_reveal(address) {
                    self.scroll_memory(target_offset);
                    return scroll_memory_to(target_offset);
                }
                Task::none()
            }
            Err(error) => {
                self.status = error;
                Task::none()
            }
        }
    }

    /// Returns the scroll offset that would bring the row containing
    /// `address` into the visible portion of the memory list, or `None`
    /// if the row is already on screen. Mirrors the visibility check used
    /// by `step_memory_address` for ArrowUp/Down navigation.
    fn scroll_offset_to_reveal(&self, address: u16) -> Option<f32> {
        let viewport = self.memory_viewport_height;
        if viewport <= 0.0 {
            // No layout has been measured yet — fall back to scrolling
            // unconditionally so the very first jump still lands on the
            // requested row.
            return Some(address as f32 * MEMORY_ROW_HEIGHT);
        }

        let row_top = address as f32 * MEMORY_ROW_HEIGHT;
        let row_bottom = row_top + MEMORY_ROW_HEIGHT;
        let view_top = self.memory_scroll_offset;
        let view_bottom = view_top + viewport;

        if row_top < view_top {
            Some(row_top)
        } else if row_bottom > view_bottom {
            Some((row_bottom - viewport).max(0.0))
        } else {
            None
        }
    }

    /// Steps the address shown in `memory_address_input` by one, wrapping
    /// around the 64 KiB window. Refreshes the memory value input for the
    /// new address, exits the search context, but **does not** scroll the
    /// memory list and does not touch the focus. Callers decide which
    /// input to leave focused.
    fn step_address_in_input(&mut self, backward: bool) {
        let current = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let delta = if backward { -1 } else { 1 };
        let next = ((current + delta).rem_euclid(total)) as u16;

        self.memory_address_input = format!("{next:04X}");
        self.refresh_memory_value(next);
        // Plain Enter exits the search context: the user is now manually
        // moving through addresses, not iterating over a pattern match.
        self.memory_search_pattern = None;
    }

    /// Plain Enter handler from the address field: step the address by one
    /// and keep the address input focused.
    pub(crate) fn advance_memory_address(&mut self, backward: bool) -> Task<Message> {
        self.step_address_in_input(backward);
        self.focused_input = Some(MEMORY_ADDRESS_INPUT_ID);
        operation::focus(MEMORY_ADDRESS_INPUT_ID)
    }

    /// Walks the address space starting just after (or before) the
    /// currently selected cell and stops on the first address whose
    /// 4-digit hex form contains the cached search pattern. The pattern
    /// is captured from the address input on the very first invocation;
    /// subsequent calls reuse it so the user can iterate through every
    /// match (because each successful match rewrites the address input
    /// with a full 4-digit hex code, which would otherwise become the
    /// next search pattern). The search wraps around the 64 KiB window
    /// and always advances by at least one address in `backward`'s
    /// direction.
    pub(crate) fn find_next_memory_address_in_direction(
        &mut self,
        backward: bool,
    ) -> Task<Message> {
        if self.memory_search_pattern.is_none() {
            let pattern = self.memory_address_input.trim().to_ascii_uppercase();
            if pattern.is_empty() {
                self.status = "Введите hex-шаблон для поиска".to_owned();
                return Task::none();
            }
            self.memory_search_pattern = Some(pattern);
        }

        let pattern = match self.memory_search_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => pattern.to_owned(),
            _ => {
                self.status = "Введите hex-шаблон для поиска".to_owned();
                return Task::none();
            }
        };

        let start = parse_hex_u16(&self.memory_address_input).unwrap_or(0) as i32;
        let total = MEMORY_ADDRESS_COUNT as i32;
        let direction = if backward { -1 } else { 1 };

        let mut next_match = None;
        for step in 1..=total {
            let candidate = ((start + direction * step).rem_euclid(total)) as u16;
            if format!("{candidate:04X}").contains(&pattern) {
                next_match = Some(candidate);
                break;
            }
        }

        match next_match {
            Some(address) => {
                self.memory_address_input = format!("{address:04X}");
                self.refresh_memory_value(address);
                self.status = format!("Найден шаблон {pattern} по адресу {address:04X}");
                let target_offset = address as f32 * MEMORY_ROW_HEIGHT;
                self.scroll_memory(target_offset);
                scroll_memory_to(target_offset)
            }
            None => {
                self.status = format!("Нет адресов, соответствующих {pattern}");
                Task::none()
            }
        }
    }

    pub(super) fn set_memory_address(&mut self, address: u16) {
        self.memory_address_input = format!("{address:04X}");
        self.refresh_memory_value(address);
        self.sync_pc_to_cursor(address);
    }

    /// Mirrors the user-visible cursor into `cpu.pc` so single-stepping
    /// from a freshly clicked cell runs against that byte. Does nothing
    /// when an instruction is mid-flight (`tact_phase.is_some()`) because
    /// rewriting PC there would restart the current instruction from the
    /// cursor on every tact, never letting the boundary count down to
    /// zero. Also short-circuits when PC already matches, to avoid
    /// pointless `SetPc` round-trips on cosmetic updates (e.g. the
    /// `follow_pc_into_memory_list` reflow after a step).
    ///
    /// Halt is the third early-return: after HLT the CPU sits on the
    /// byte past the halt opcode, but every step/run gesture is
    /// halt-blocked (Variant A) until the user resets the registers.
    /// In that state PC is *not* the address the next step will run
    /// from — there is no next step — so syncing the cursor into PC
    /// is meaningless. Worse, it's actively harmful: a freshly clicked
    /// `SetPc(addr)` blocks on a `StateChanged`, which on a halted CPU
    /// is the same post-halt snapshot the worker keeps republishing
    /// (PC = halt_pc + 1). `apply_snapshot` then has nothing to
    /// preserve from `memory_address_input` because the spinner only
    /// just changed, so the visible address jumps to PC = `addr + 1`.
    /// That is the "click any row, get bumped one cell forward" bug
    /// reported on halted snapshots. Skipping the dispatch entirely
    /// here lets the user browse memory freely after HLT — and the
    /// next reset reattaches PC to whatever they end up clicking.
    pub(super) fn sync_pc_to_cursor(&mut self, address: u16) {
        if self.snapshot.cpu.tact_phase.is_some()
            || self.snapshot.cpu.halted
            || self.snapshot.cpu.pc == address
        {
            return;
        }
        self.dispatch_sync(AppCommand::SetPc(address));
    }

    /// Dispatches a single-instruction step and then jumps the memory
    /// list / address spinner to the new program counter so the user can
    /// see exactly which cell the CPU will execute next. Mirrors the
    /// "follow PC" affordance of the reference KR-580 emulator's
    /// step-instruction toolbar button: after each press the highlight
    /// auto-advances by however many bytes the executed opcode consumed.
    ///
    /// Two subtle rules:
    ///
    /// 1. The address shown in the memory spinner is treated as the
    ///    user-visible cursor — if it differs from `cpu.pc`, we sync PC
    ///    to it *before* stepping so "step from where I clicked" works.
    /// 2. We use `dispatch_sync` rather than the fire-and-forget
    ///    `dispatch`, otherwise the read of `self.snapshot.cpu.pc`
    ///    races the worker channel and the user has to click twice
    ///    before the highlight catches up.
    pub(crate) fn step_instruction_and_advance(&mut self) -> Task<Message> {
        // Halt-block latch (Variant A in docs/ui_app.md). Once a
        // run/step gesture has tripped HLT, every further execution
        // attempt is a no-op until the user explicitly resets
        // registers or clears the halt bit. The action-panel button
        // is already disabled by the latch (its `Message` lands as
        // `None`, so `on_press` is dropped), but the keyboard
        // shortcut Ctrl+T and the menu entry still route here, so
        // we re-raise the notice and bail. Re-raising rather than
        // staying silent matters when the 8-second fade has already
        // dismissed the original overlay — without this the user
        // would press the shortcut and see nothing happen, with no
        // on-screen explanation for why.
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        // After HLT the CPU sits one byte past the HLT opcode and a
        // fresh `StepInstruction` would just walk into the byte after
        // forever — exactly the bug Variant A is meant to block. Per
        // docs/ui_app.md, refuse the gesture and surface a top-center
        // notification (the floating frame above the schematic) about
        // the only way out — register reset. The action panel buttons
        // stay clickable on purpose so the user can read the
        // explanation by pressing them.
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.dispatch_sync(AppCommand::StepInstruction);
        self.follow_pc_into_memory_list()
    }

    /// Dispatches a single-tact step and follows PC into the memory
    /// list only when the executed tact lands on an instruction
    /// boundary. PC is first synced to the address shown in the spinner
    /// (cursor takes precedence) so a fresh tact runs against the byte
    /// the user is looking at.
    ///
    /// The 8080 core mutates PC on the *first* tact and then merely
    /// counts the remaining T-states down to zero, so comparing PC
    /// before/after would teleport the cursor on every press. The
    /// handler watches `last_tact_was_boundary` instead, which the
    /// event consumer raises on `TactAdvanced { instruction_boundary:
    /// true }`. The flag is cleared *before* dispatch so a stale value
    /// from a previous press cannot leak through.
    pub(crate) fn step_tact_and_maybe_advance(&mut self) -> Task<Message> {
        // Halt-block latch (Variant A): same gating story as
        // `step_instruction_and_advance` — the keyboard shortcut
        // Ctrl+Y and the menu entry still route here even with
        // the action-panel button already disabled, so we re-raise
        // the notice and bail. See the matching comment in
        // `step_instruction_and_advance` for why re-raising is the
        // right move once the 8-second fade has eaten the overlay.
        if self.run_blocked_after_halt {
            self.raise_halt_notice();
            return Task::none();
        }
        // Same halt-block as `step_instruction_and_advance`: a tact
        // step on a halted CPU would just spin the cycle counter on
        // the byte past HLT. See Variant A in docs/ui_app.md.
        if self.snapshot.cpu.halted {
            self.raise_halt_notice();
            return Task::none();
        }
        self.last_tact_was_boundary = false;
        self.dispatch_sync(AppCommand::StepTact);
        if !self.last_tact_was_boundary {
            return Task::none();
        }
        self.last_tact_was_boundary = false;
        self.follow_pc_into_memory_list()
    }

    /// Refresh the memory spinner / inline editor / scroll position to
    /// point at the current `cpu.pc`. Shared by the step-instruction and
    /// step-tact handlers so both feel the same after the CPU moves.
    /// Visibility is `pub(super)` so the undo runtime can also reach
    /// for it: replaying a `Cpu` undo entry rewinds PC, and the user
    /// expects the highlighted row to come along for the ride.
    pub(super) fn follow_pc_into_memory_list(&mut self) -> Task<Message> {
        let pc = self.snapshot.cpu.pc;
        self.select_memory(pc);

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }

        let Some(target_offset) = self.scroll_offset_to_reveal(pc) else {
            return Task::none();
        };

        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    /// "Follow PC" affordance for the paced `Run` loop. Called from
    /// `Message::Tick` after `pull_events` has folded the latest worker
    /// snapshot in: if the run is armed and the visible spinner address
    /// has fallen behind `cpu.pc`, drag the highlight forward so the user
    /// sees the program walking through the memory list one cell at a
    /// time. Returns the scroll task so iced re-anchors the viewport on
    /// the same frame.
    ///
    /// Differs from [`follow_pc_into_memory_list`] in two ways:
    ///
    /// 1. It does **not** call `sync_pc_to_cursor`. During a paced run
    ///    PC is the source of truth — pushing `SetPc(pc)` back at the
    ///    worker on every tick would be a useless round-trip and could
    ///    race the next instruction step. The `select_memory` path is
    ///    avoided for the same reason; we only mirror its cosmetic
    ///    side-effects (clear the opcode dropdown, refresh the inline
    ///    value buffer).
    /// 2. It refuses to overwrite `memory_inline_value_input` only when
    ///    the buffer holds *unsaved* user input — i.e. the buffer
    ///    differs from the byte that actually lives under the old
    ///    highlight. The naïve "skip if MEMORY_INLINE_INPUT_ID is
    ///    focused" guard was too aggressive: the cosmetic
    ///    `focused_input` tracker frequently stays armed on the inline
    ///    id long after the user has stopped typing (it is set on every
    ///    click into a memory row), so `Run` would freeze the inline
    ///    cell on the byte from whichever address the user clicked
    ///    last, while the address column kept stepping forward. The
    ///    rendered effect was "all values shift by one row" — exactly
    ///    the symptom from the screenshot.
    pub(crate) fn follow_pc_during_run(&mut self) -> Task<Message> {
        // After HLT the 8080 advances PC past the halt opcode (PC sits
        // one byte after HLT), but the user expects the highlight to
        // land *on* the HLT row — that's the row that gets the red
        // halt-styling in the memory list, and it matches the mental
        // model "the program ended on this instruction". Aim one byte
        // back when halted; guard against a `pc == 0` underflow that
        // could only happen if a program halted at address 0, which
        // is degenerate but cheap to defend against.
        let target = if self.snapshot.cpu.halted && self.snapshot.cpu.pc > 0 {
            self.snapshot.cpu.pc.wrapping_sub(1)
        } else {
            self.snapshot.cpu.pc
        };
        let current_address = parse_hex_u16(&self.memory_address_input).ok();
        if current_address == Some(target) {
            return Task::none();
        }

        // Snapshot the byte that lives under the old highlight *before*
        // we move the spinner. If the inline buffer matches it, the
        // user has nothing unsaved there — we can safely migrate the
        // buffer onto the new address. If they diverge, the user typed
        // something we must not stomp on.
        let inline_was_clean = match current_address {
            Some(addr) => {
                let stored = format!("{:02X}", self.snapshot.cpu.memory.read(addr));
                self.memory_inline_value_input.eq_ignore_ascii_case(&stored)
            }
            // No parseable address means whatever sits in the buffer
            // can't be tied to a memory cell anyway — treat as clean.
            None => true,
        };

        // Cosmetic half of `select_memory` — clear the opcode dropdown
        // context so the previous row's hover state does not bleed onto
        // the new highlight.
        self.opcode_dropdown_address = None;
        self.opcode_search_input.clear();
        self.memory_address_input = format!("{target:04X}");
        self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(target));

        if inline_was_clean {
            self.memory_inline_value_input = self.memory_value_input.clone();
        }

        if self.memory_viewport_height <= 0.0 {
            return Task::none();
        }

        let Some(target_offset) = self.scroll_offset_to_reveal(target) else {
            return Task::none();
        };

        self.scroll_memory(target_offset);
        self.memory_scroll_visible_ticks = MEMORY_SCROLL_VISIBLE_TICKS;
        scroll_memory_to(target_offset)
    }

    pub(super) fn refresh_memory_value(&mut self, address: u16) {
        self.memory_value_input = format!("{:02X}", self.snapshot.cpu.memory.read(address));
        self.memory_inline_value_input = self.memory_value_input.clone();
    }
}
