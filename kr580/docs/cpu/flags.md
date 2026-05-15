# Flag rules

| Op family            | S | Z | AC                                            | P | CY                          |
| -------------------- |:-:|:-:| --------------------------------------------- |:-:| --------------------------- |
| `ADD/ADC/ADI/ACI`    | r | r | carry-out of bit 3                            | r | carry-out of bit 7          |
| `SUB/SBB/SUI/SBI`    | r | r | `(a & 0xF) >= (b & 0xF) + cy_in` (8080 subt.) | r | `a < b + cy_in`             |
| `CMP/CPI`            | r | r | as SUB                                        | r | as SUB                      |
| `INR r/M`            | r | r | `(v & 0xF) == 0xF` (low-nibble overflow)      | r | unchanged                   |
| `DCR r/M`            | r | r | `(result & 0xF) != 0xF`                       | r | unchanged                   |
| `ANA/ANI`            | r | r | `bit3(A) | bit3(operand)` *before* AND        | r | cleared                     |
| `ORA/ORI/XRA/XRI`    | r | r | cleared                                       | r | cleared                     |
| `DAD rp`             | – | – | unchanged                                     | – | overflow out of bit 15      |
| `RLC/RRC/RAL/RAR`    | – | – | unchanged                                     | – | rotated bit                 |
| `STC`                | – | – | unchanged                                     | – | set                         |
| `CMC`                | – | – | unchanged                                     | – | inverted                    |
| `CMA`                | – | – | unchanged                                     | – | unchanged                   |
| `DAA`                | r | r | recomputed from low-nibble adjustment         | r | sticky / re-set on adjust   |

`r` = recomputed from the result byte.

## PSW byte layout

```
bit:   7  6  5  4  3  2  1  0
       S  Z  0  AC 0  P  1  CY
```

* On store (`PUSH PSW`, snapshot): bits 3 and 5 are zero, bit 1 is one.
* On load (`POP PSW`): bits 3 and 5 are ignored; bit 1 forced on next store.

## Tests

The semantics are covered in:

* `kr580-core::flags::tests` — PSW round-trip.
* `kr580-core::execute::alu::tests` — SUB AC `1-0` and `0-1`, ADD AC,
  INR/DCR AC.
* `kr580-core::execute::logic::tests` — ANA / ORA / XRA AC + CY.
* `kr580-core::execute::misc::tests` — STC, CMC, CMA flag invariance.
* `kr580-core::execute::rotates::tests` — only CY + rotated bits change.
