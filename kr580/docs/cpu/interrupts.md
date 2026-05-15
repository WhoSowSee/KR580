# Interrupts

The interrupt model implements exactly the rules in
`prompt/02_cpu_core.md`.

## State

```rust
pub interrupt_enable: bool,
pub interrupt_enable_pending: bool,
pub interrupt_request_pending: bool,
pub interrupt_vector_byte: Option<u8>,
pub halted: bool,
```

## Rules

1. **`EI` is deferred.** `EI` sets `interrupt_enable_pending = true`. At the
   *next* instruction boundary (before fetch) the latch is promoted to
   `interrupt_enable = true` and cleared. This matches real-silicon behaviour
   and prevents an immediate interrupt from stealing the next instruction.
2. **`DI` is immediate.** Both the live enable and the pending latch are
   cleared.
3. **Pending while disabled.** If `IRQ` is set while `IE = 0`, the request
   stays pending until acceptance. No automatic timeout.
4. **Acceptance.** At an instruction boundary, when `IRQ && IE`, the CPU:
   * clears `interrupt_enable`,
   * clears `interrupt_enable_pending`,
   * clears `interrupt_request_pending`,
   * clears `halted`,
   * consumes `interrupt_vector_byte`.
5. **Vector byte.** Only single-byte vectors (`RST n`, encoding
   `0b11_xxx_111`) are honoured. Other byte values are accepted but only
   consume 4 T-states; they are not decoded as multi-byte sequences.
6. **HLT.** Cleared by reset or by an accepted interrupt while IE = 1.

## Test coverage

* `interrupt::tests::ei_is_deferred_one_instruction`
* `interrupt::tests::pending_irq_with_disabled_stays_pending`
* `interrupt::tests::enabled_irq_routes_through_rst_vector`
* `interrupt::tests::irq_unhalts_cpu`
