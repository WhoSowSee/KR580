# Assumptions and Limitations

- External binary opcode suites (`CPUDIAG.BIN`, `8080EXM.COM`, `TST8080.COM`, `8080PRE.COM`, `CPUTEST.COM`) are not included in the repository, so automated verification currently uses local semantic tests and full opcode classification/execution smoke tests.
- Interrupt acknowledge accepts only single-byte `RST n` vectors. Other vectors return `DecodeError::InvalidInterruptVector`.
- Storage workers expose not-ready/disconnected/I/O errors on enqueue and worker paths and publish the last error in device snapshots. Native printer jobs publish completion or failure through a dedicated completion channel.
- Tact stepping keeps exact T-state accounting and instruction-boundary device semantics. It is a deterministic debug model, not a transistor-level decomposition of each 8080 machine cycle.
