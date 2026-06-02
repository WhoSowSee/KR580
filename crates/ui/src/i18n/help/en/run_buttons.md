Execution button group (right panel):

- Run / Pause (Ctrl+R) - starts/stops automatic execution. Button color: green (run) <-> red (pause).
Checks: program exists at PC (byte != 00h), HLT not blocking.

- Step Instruction (Ctrl+T) - executes one instruction, then stops. PC advances to next instruction.
Also works as Restart: Ctrl+click resets PC to start before execution.

- Step Tact (Ctrl+Y) - executes one machine tact. Allows observing microprogram execution.

Execution speed is set via the Speed panel (left, bottom) and in settings.