# Execution and schematic

The backend owns execution; the UI selects a run mode and displays confirmed
CPU snapshots. See [Core CPU](core.md) for instruction, IRQ and partial-step
contracts, and [Architecture](architecture.md#actor-pacing) for scheduling.

## Run modes and speed

| UI tier | Backend mode | Meaning |
|---|---|---|
| Slow | `Paced` | Nominal 5 instructions/s, 200 ms interval |
| Medium | `Paced` | Nominal 20 instructions/s, 50 ms interval |
| High | `Paced` | Nominal 120 instructions/s; the default UI tier |
| Max | `Burst { slice: 16 ms }` | Execute instruction batches within a wall-time budget |

`apply_speed_tier` sends both `SetStepInterval` and `SetRunMode`, including at
startup. Saved speed therefore reaches the actor before the first UI Tick.
Paced intervals and burst slices have a 1 ms minimum.

The nominal `tier_hz` value for Max is 1000 and is displayed by the speed gauge.
It does not cap burst throughput. Actual throughput depends on the program,
devices and machine; Max publishes one final snapshot per batch rather than
per-instruction boundary events.

The actor pauses at HLT, execution failure or the per-arming instruction budget.
Stop is handled between execution slices. Queueing and OS scheduling affect
latency; the 16 ms budget is not a real-time response guarantee.

UI Tick cadence is separate: it has a 16 ms floor while running and is slower
while idle. The snapshot mailbox may coalesce intermediate states even at High.
CPU timing is not derived from rendered frame count or the monitor refresh rate.

## Execution actions

The run button toggles Run/Stop. The instruction button steps while paused and
restarts while running; restart resets CPU state, preserves RAM and rearms Run
after the reset completion. Pausing cancels a queued restart follow-up.
The tact button advances one T-state.

Edits, resets and stepping use correlated requests. The UI moves the cursor and
updates its undo state after confirmation, preserving a newer selection or draft
made while that request was pending.

An empty program location produces a hint instead of arming execution.
Run and step attempts while halted show the HLT notice rather than advancing CPU.

## Follow PC and HLT

With Follow PC enabled, the RAM selection follows confirmed execution boundaries
and the latest running snapshot. A terminal event sets `pending_follow_pc`, so
the final PC is followed once even when Run stops before the next UI Tick.
Focused memory drafts are protected while snapshots continue to arrive.

After HLT, PC points past the opcode, but the UI selects `pc - 1` and marks that
row as halted. Single stepping and the final running update use the same target.

The HLT notice expires after eight seconds or user dismissal. The execution
lock remains until a non-halted snapshot, CPU reset or Clear HLT removes it.
Fading the notice does not re-enable run/step controls.

Clear HLT changes only the halt state. CPU reset follows the reset contract in
[Core CPU](core.md), preserving RAM. The HLT chip is an undoable toggle and
does not move the selected RAM row. Clearing HLT alone does not rewind PC.

## Schematic readouts

The schematic distinguishes the next instruction from the last observed bus
operation. These values can legitimately differ after a jump, stack operation,
HLT or a newly loaded document.

| Readout | Source and meaning |
|---|---|
| PC / SP | Current program counter and stack pointer |
| Instruction register | `last_fetched_opcode`, retained until another fetch |
| Instruction decoder | Mnemonic for the instruction-register byte |
| Current command | Opcode, operands, size, category and addressing at current `memory[PC]` |
| Data buffer | `last_data_bus_byte` |
| Address buffer | `last_address_bus`, including memory, stack and mirrored I/O addresses |
| W / Z | Internal temporary register pair |
| Flags | PSW result bits, including its fixed reserved-bit format |
| Increment/decrement display | Decoded instruction size as the displayed PC increment |

Register copies share their draft and selection rules with the
[register editor](ui_app.md#register-editor). Undocumented bytes show a neutral
placeholder when no mnemonic or timing layout is available.

Bus latches reflect successful boundary operations. In particular, IN/OUT
place the port on both address halves and the transferred byte on the data bus;
the core [execution API](core.md#execution-api) defines the full contract.

## Timing panels

`view/cycles.rs` renders two presentations of the same deterministic T-state
walk. The core commits architectural and device effects at instruction boundaries,
rather than simulating every internal transistor transition.

| Panel | Interpretation |
|---|---|
| Cycle and tact | M-cycle number and T-position using the school-convention layout |
| Internal timings | Total T-states, T-position in the full-duration layout and zero-based linear phase |

For HLT, the school layout exposes its 4T fetch while the full-duration display
covers the 7T instruction. Other opcodes use the core machine-cycle table.
The layout difference is intentional; CPU accounting still includes all HLT
T-states.

The active display uses `tact_phase`, falling back to `last_completed_tact_phase`
at a boundary. The completed-tact row uses the last completed phase directly.
Linear phase spans the whole instruction; the T-position restarts with each
M-cycle in its selected layout. It is not a second total-instruction counter.

`timing_opcode()` selects the latched active opcode during a partial walk.
`timing_branch_taken()` selects the recorded branch during that walk and resolves
completed layout choice from the phase afterward. The view clamps a completed
phase to its layout's last position, so HLT remains visible at school T4.

Machine-cycle lengths and kinds are defined under `crates/core/src/machine_cycle/`.
Their sums are checked against decoder timing, with the school HLT exception.
Snapshot format persistence is defined in [Persistence](persistence.md); timing
metadata is not part of a `.580` file.

## Control lamps and status byte

The lamps are derived from the current CPU timing/interrupt state, with an idle
fetch presentation between instructions.

| Lamp | Modelled indication |
|---|---|
| F1 / F2 | Alternating phases during an active walk; F2 at idle fetch |
| SYNC | T1 status latch; active at idle fetch |
| READY | Ready unless the model is in WAIT/HLT |
| WAIT | Halted or in the halt-ack presentation |
| HOLD | Off: external DMA requests are not modelled |
| INT | Pending interrupt request |
| INTE | Actual interrupt-enable state |
| DBIN | Read machine-cycle strobe after T1 |
| WR | Write machine-cycle strobe after T1 |
| HLDA | The emulator's HLT latch indication; no DMA acknowledgement is simulated |

The status-register readout encodes the active machine-cycle status byte.
Its tooltip identifies the cycle without duplicating the bit display. The
emulator's HLDA convention is a presentation choice, not a complete hardware
HOLD/HLDA model. See [model limitations](assumptions.md).
