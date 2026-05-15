# Assumptions and Limitations

- External binary opcode suites named in the prompt (`CPUDIAG.BIN`, `8080EXM.COM`, `TST8080.COM`, `8080PRE.COM`, `CPUTEST.COM`) were not present in the workspace, so automated verification currently uses local semantic tests and full opcode classification/execution smoke tests.
- Interrupt acknowledge accepts only single-byte `RST n` vectors. Other vectors return `DecodeError::InvalidInterruptVector`, matching the prompt scope.
- Storage and printer async workers log worker-side I/O errors and expose not-ready/disconnected errors on enqueue paths. A later UI can surface worker completion channels if deeper operator feedback is needed.
- The initial UI exposes `.580` open/save and `.txt` export. `.xlsx` and `.docx` exporters are implemented in `k580-persistence` and wired through `AppCommand`; additional UI buttons can call them without changing core architecture.
