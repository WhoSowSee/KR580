# Assumptions and Limitations

- External binary opcode suites named in the prompt (`CPUDIAG.BIN`, `8080EXM.COM`, `TST8080.COM`, `8080PRE.COM`, `CPUTEST.COM`) were not present in the workspace, so automated verification currently uses local semantic tests and full opcode classification/execution smoke tests.
- Interrupt acknowledge accepts only single-byte `RST n` vectors. Other vectors return `DecodeError::InvalidInterruptVector`, matching the prompt scope.
- Storage and printer async workers expose not-ready/disconnected errors on enqueue paths and publish last enqueue errors in device snapshots. Worker-side completion/error channels remain a possible future enhancement for deeper operator feedback.
- Tact stepping keeps exact T-state accounting and instruction-boundary device semantics. It is a deterministic debug model, not a transistor-level decomposition of each 8080 machine cycle.
