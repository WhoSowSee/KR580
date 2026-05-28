# App and UI

`k580-app` owns the emulator and exposes PascalCase commands such as
`ResetCpu`, `StepTact`, `RunForTStates`, `StepInstruction`, `Run`, `Stop`,
`SetStepInterval`, `SetRunMode`, `SetRegister`, `SetMemory`, `ReadPort`,
`WritePort`, `LoadSnapshot`, `SaveSnapshot`, `LoadSubprogram`, and direct
export commands.

`k580-ui` is an iced application shell. It renders an `AppSnapshot`, sends
`AppCommand` values to the actor, and drains `AppEvent` notifications.
Register and memory edits are parsed and validated before commands are
sent.

Native file dialogs use `rfd`. The UI exposes `.580` open/save and
`.txt`/`.xlsx` export and import actions. It does not serialize files,
run CPU instructions directly, or store emulator state in widgets.

## UI module split

- `main.rs` initializes tracing, declares the iced subsystem and window
  options, sets the app-level theme/style, and wires the embedded window
  icon. It also pins the Windows subsystem to GUI on release builds (see
  "Console window suppression").
- `app/` defines `DesktopApp`, message routing, theme, and the keyboard
  subscription:
  - `app/mod.rs` — state container, `update`, `subscription`, and
    `handle_arrow_key`, the router that maps a raw ArrowUp/ArrowDown
    press to the editor that currently owns focus.
  - `app/messages.rs` — the `Message` enum.
  - `app/constants.rs` — widget identifiers, register order, and the
    name lookup helpers. Re-exported from `crate::app::*` so the rest
    of the crate keeps importing them by short path.
- `runtime/` contains app-facing command dispatch, event draining, file
  dialogs, and the per-panel update logic. The methods all hang off
  `impl DesktopApp` and are grouped by responsibility:
  - `runtime/mod.rs` — `dispatch`, `pull_events`, `apply_snapshot`,
    file-dialog helpers.
  - `runtime/register.rs` — register name/value editing, including
    `step_register_value_input` for ArrowUp/ArrowDown ±1 stepping.
  - `runtime/memory.rs` — memory list, address spinner, inline editor,
    Ctrl+Enter pattern search, and the matching value-step helpers
    (`step_memory_value_input`, `step_inline_memory_value_input`).
  - `runtime/focus.rs` — Tab/Shift+Tab cycling between fields.
  - `runtime/parse.rs` — small free helpers (hex parsing, normalization,
    `saturating_step_u8`).
- `view.rs` renders the current snapshot, lays out every panel, and owns
  every widget style.
- `platform.rs` is a Windows-only helper used by `app/mod.rs` for DWM
  cloaking during launch (see "Launch flash mitigation"). On non-Windows
  targets it compiles down to a no-op.

## Event handling

The actor publishes `StateChanged`, `InstructionBoundaryReached`,
`TactAdvanced`, `PortRead`, `PortWritten`, `HaltStateChanged`,
`ErrorRaised`, and `Stopped`. Events are notifications only; the latest
`AppSnapshot` remains the authoritative render source.

## Top menu chrome

The custom top menu bar is a flat 34 px strip on `TOKYO_BOARD`. It no
longer draws a visible bottom hairline divider: the existing 1 px
divider slot paints `TOKYO_BOARD`, so the top of the app reads as one
quieter surface while the dropdown offsets stay unchanged. Dropdowns
still open at the same 34 px vertical offset and keep their own framed
panel border.

The visible top-level categories are localized as `Файл`,
`МП-Система`, `Вид`, `Настройки`, and `Справка`. `Файл` and
`МП-Система` open dropdowns; the last three are still inactive
placeholders, but they use the same Russian UI language as the rest of
the menu bar.

Legacy `.580` rows in the file dropdown keep the primary action as the
main label (`Открыть` / `Сохранить`) and render `старый формат` as a
small muted note beside it, not as parenthesized label text.

The empty title-bar band between the menu labels and caption buttons is
also the window drag handle. When no transient UI is open it dispatches
`iced::window::drag`; when a dropdown or opcode picker is open, the same
press is treated as a dead-space click and closes that popup instead of
starting a drag. That keeps title-bar whitespace consistent with the
rest of the inactive app surface.

## Side panel layout

The right-hand 330 px column stacks four legend-framed panels in this
order, top to bottom:

1. **«Список ячеек ОЗУ»** — virtualised memory list with the inline
   value editor and the opcode dropdown.
2. **«Ячейка ОЗУ и ее значение»** — address spinner + value field +
   `↵` apply button.
3. **«Регистр и его значение»** — register name spinner + value field +
   `↵` apply button.
4. **«Управление»** — action button strip described below.

### «Управление» action panel

A single horizontal strip of square 38×38 SVG icon buttons that mirror
the toolbar of the reference KR-580 emulator. The execution group sits
on the left, then a thin vertical divider, then the memory-state
(reset) group on the right (the divider colour matches the surrounding
panel border so it reads as a piece of the frame):

| Group | Icon | Message | Accent | Tooltip |
|---|---|---|---|---|
| run  | `play.svg` / `pause.svg` | `Message::ToggleRun`       | green / red | Выполнить программу / Пауза |
| run  | `step-forward.svg` / `refresh-ccw.svg` | `Message::StepInstruction` / `Message::RestartProgram` | blue | Выполнить команду / Перезапустить программу |
| run  | `redo-dot.svg`        | `Message::StepTact`        | yellow  | Выполнить такт |
| reset | `reset-ram.svg`       | `Message::ResetRam`        | red     | Сброс ОЗУ |
| reset | `reset-registers.svg` | `Message::ResetCpu`        | magenta | Сброс регистров |

The first two buttons are tumblers driven by `DesktopApp::running`.

The leftmost (run/pause) button mirrors the reference KR-580 emulator.
At rest it paints `play.svg` in green; once armed it swaps to
`pause.svg` in red. **Pause is unconditional**: a click while the run
is armed always sends `AppCommand::Stop`, regardless of where PC has
walked to. This matters because a paced run carries PC through
whatever bytes follow the user's program — once it walks off the
loaded code into a stretch of `0x00` (the default RAM fill), any
gate that compares the *current* `cpu.memory.read(pc)` against zero
would mistake the running program for an empty page and silently
swallow the click. The user reported this as «не могу остановить
программу, только сбросом регистров»: typed `13` at 0x0000, ran it,
PC walked through INX D + NOPs, and pause did nothing because the
byte at the new PC was zero. The current handler returns Stop first
and only then reaches the run-arming gates, so Stop is reachable
from any execution state.

Run-arming is the gated half. With the toggle disarmed, the handler
checks both the halted bit and the byte at `cpu.pc`: a halted CPU
surfaces the reset-registers notice (Variant A — see below), and a
blank page (byte is `0x00`) yields a `Нет программы по адресу <PC>`
hint with no worker activity. Tying the visual flag to the same
condition that gates the dispatch prevents the desync the user
reported earlier as
«программа выглядит будто работает, но ничего не выполняется»:
previously, an unconditional icon flip on an empty page survived a
subsequent `Import` / `OpenSnapshot`, leaving the panel painted red
while the worker sat idle. Two extra safeguards back this up —
`load_snapshot_from_path` and `import_file` both clear `self.running`
before they touch the worker, so any prior cosmetic state from
before the document changed is dropped. This also avoids the older
bug where every click on an empty RAM page burned ~100k T-states
inside `cpu.run_until_halt(&mut bus, 100_000)`.

`AppCommand::Run` no longer blocks the worker. It only flips an
`is_running` flag on the emulator; the actual instruction-by-instruction
advance is driven by the worker's `select!` loop (see
`docs/architecture.md`), which fires one `tick()` per
`step_interval`. Each `tick()` executes a single instruction, publishes
`InstructionBoundaryReached` + `StateChanged`, and decrements an internal
`MAX_INSTRUCTIONS_PER_RUN` budget so a runaway program eventually pauses
itself instead of burning the worker thread. `AppCommand::Stop` clears
the flag and emits `Stopped`. The combined effect: the highlighted memory
cell, register readouts, PC, and cycle counter all animate live as the
program executes — no more "click play, see only the final state".

The second (step / restart) button is `step-forward` at rest and
`refresh-ccw` while running. At rest it sends
`Message::StepInstruction`, which dispatches a single
`AppCommand::StepInstruction` and then jumps the memory list / address
spinner to the new program counter so the highlighted cell follows the
CPU as the user steps through code. While running it sends
`Message::RestartProgram`, which dispatches `AppCommand::ResetCpu`
followed by `AppCommand::Run`: the registers and flags are wiped, the
program counter goes back to `0x0000`, the run state stays armed, and
the program executes again from the beginning. Memory is preserved
(no `ResetRam`), so the loaded program survives the restart.

The SVG sources live under `assets/icons/actions/` and are embedded at
build time with `include_bytes!`; `crates/ui/src/view/icons.rs` holds
one `LazyLock<svg::Handle>` per icon and exposes a thin getter for each.
The `play` and `step-forward` icons come from the Lucide set; the
`redo-dot` icon is also Lucide and reads as "step one tick around the
loop"; `reset-ram` and `reset-registers` are custom KR-580 glyphs (a
DIMM silhouette and a stacked-register block, each annotated with a
circular reset arrow). The speed stepper additionally uses Lucide
`chevrons-left` / `chevrons-right` action SVGs for adjacent-tier
switching. All action SVG files declare
`stroke="currentColor"`, so the iced `svg` widget tints them at
runtime via `svg::Style { color: Some(accent) }` — the accent is the
glyph colour at rest and the border colour on hover/press, while the
surface stays on the neutral `TOKYO_BOARD` / `TOKYO_BORDER` palette of
the editor `↵` button and input shells; hover uses the darker
`TOKYO_SURFACE` tone (`#1D2030`) so the feedback stays visible without
reading as a raised light card. Tooltip bodies use
`inset_style`; it shares the same darker `TOKYO_BOARD` fill as the
`Регистр состояния` tooltip, so all hover tips now use one surface tone.
The same actions
remain available from the top menu bar — this panel is a discoverable
in-context surface for the same commands; no new `AppCommand` or
`Message` variants were added. iced's `svg` Cargo feature is enabled
in `crates/ui/Cargo.toml` so the renderer pulls in the resvg backend.

### Быстрый доступ (bottom of the schematic plate)

The bottom strip of the schematic plate now carries a legend-framed
**«Быстрый доступ»** panel with five peripheral chips in this fixed
left-to-right order:

| Slot | Glyph | Accent | Tooltip |
|---|---|---|---|
| Монитор | `device_monitor` (`assets/icons/devices/monitor.svg`) | green | Отобразить монитор |
| Дисковод | `device_floppy` (`devices/floppy.svg`) | cyan | Отобразить буфер дисковода |
| Диск | `device_hdd` (`devices/hdd.svg`) | blue | Отобразить буфер жёсткого диска |
| Адаптер | `device_network` (`devices/network.svg`) | yellow | Отобразить буфер сетевого адаптера |
| Принтер | `device_printer` (`devices/printer.svg`) | magenta | Отобразить буфер принтера |

Each chip is rendered by `view::chips::device_chip`: a tinted SVG
glyph centred inside the same neutral button chrome as the action
buttons (38×38, the
same footprint and icon scale as the action buttons in «Выполнение» /
«Сброс») plus a hover tooltip that reuses the editor `inset_style` so
it visually belongs to the same chrome family as the action-panel
tooltips. Hover uses the shared dark `TOKYO_SURFACE` fill and keeps the
neutral frame colour, matching the current action-button feedback while
leaving the chips command-neutral.

The older "I/O Controller" capsule that used to sit on the right of
the same row was removed: it duplicated the role of the device strip
without carrying any live readout of its own.

The speed switch sits on the same bottom row, aligned to the right of
the quick-access frame, matching the reference layout where the
peripheral shortcuts and run-speed control occupy the lower band
outside the main CPU schematic frame. The left schematic column uses a
smaller `MAIN_TO_BOTTOM_SPACING` and no bottom board padding so this
bottom row sits on the same horizontal level as the right-side
«Выполнение» / «Сброс» panels. Its outer left padding is also removed:
the shared app root already supplies the 8 px window gutter, so the left
edge now mirrors the right edge.

### Schematic block chrome

Фреймы внутри левой схемы теперь outline-only: `schematic_block_style`,
`mux_panel_style`, `mux_chip_style`, `mux_header_style`,
`schematic_readout`, `schematic_wide_readout`, device chips,
`legend_panel_left` и общая рамка верхней области не задают resting
fill. Структуру держат только `TOKYO_BORDER`-линии, поэтому блоки не
выглядят как отдельные залитые карточки на плате.

Интерактивные элементы сохраняют feedback: register chips в группе
«Регистры и операнды» поднимают fill до общего `TOKYO_SURFACE` на hover,
но активный chip получает `TOKYO_SELECTION_BLUE` — ту же синюю заливку,
что выбранная строка ОЗУ. РОН ячейки мультиплексора используют тот же
тёмный hover-fill и тот же selected-blue, но активная цель хранится как
`RegisterInlineTarget`, поэтому выбор `B` в верхнем буферном блоке не
подсвечивает `B` в мультиплексоре и наоборот.

### Lamps strip — F1/F2/SYNC/READY/WAIT/HOLD/INT/INTE/DBIN/WR/HLDA

Нижний ряд ламп управления — pure function от `&Cpu8080State`,
живёт в `view::lamps`. Раньше это был статический массив с красной
точкой на каждой подписи, всегда «горящей»; пользователь попросил
сделать индикаторы интерактивными, как флаги Z/S/P/C/AC.

В состоянии покоя (`tact_phase == None`, между инструкциями) панель
повторяет силуэт референсного эмулятора КР-580: с момента открытия
окна горят `F2`, `SYNC`, `READY`, `INTE`, `WR` — это и есть
«опорный» T1 первого M1-фетча, который в учебниках всегда нарисован
рядом с распиновкой чипа. При пошаговой прокрутке (`step_tact`)
лампы переключаются на фазно-зависимые значения.

| Лампа | Что значит на 8080            | Из чего берём (idle / в такте)                   |
|-------|-------------------------------|--------------------------------------------------|
| F2    | Такт фазы 2                   | idle: горит; в такте: `Some(p)` где `p` нечётно  |
| F1    | Такт фазы 1                   | idle: тёмная; в такте: `Some(p)` где `p` чётно   |
| SYNC  | Защёлка статус-байта (T1)     | idle: горит; в такте: `tact_phase == Some(0)`    |
| READY | Память/IO готовы              | `!cpu.halted` (мы не блокируем шину)             |
| WAIT  | Процессор ждёт / остановлен   | `cpu.halted`                                     |
| HOLD  | Запрос DMA-захвата            | всегда `false` — DMA не моделируется             |
| INT   | Запрос прерывания             | `cpu.interrupt_request_pending`                  |
| INTE  | Прерывания разрешены          | idle: горит; иначе `cpu.interrupt_enable`        |
| DBIN  | Строб чтения шины данных      | всегда `false` — состояние машинного цикла не моделируется |
| WR    | Строб записи                  | idle: горит (M1-fetch); иначе тёмная             |
| HLDA  | Подтверждение DMA-захвата     | всегда `false` — DMA не моделируется             |

`HOLD`/`HLDA`/`DBIN` остаются тёмными намеренно: эмулятор
работает на уровне границы инструкций и не выводит наружу состояние
машинного цикла, которое эти выводы переключают. Показать их «всегда
выключенными» честнее, чем имитировать тайминг-диаграммы из учебника.

Подписи у ламп теперь горизонтальные и расположены **сверху** над
точкой внутри framed-панели «Сигналы управления». Это повторяет
пример 2: строка читается как обычная таблица сигналов, без SVG-
поворота текста. Точка под подписью красится в `TOKYO_RED`, когда
сигнал активен, и в `TOKYO_TEXT`, когда нет — та же идиома, что у
`flag_dot` для Z/S/P.

The separate Z/S/P/C/AC flag strip is centred inside its framed block
with symmetric `Length::Fill` spacers on both edges and fixed gaps
between dot columns. That keeps the first and last lamp the same visual
distance from the frame sides even when the PSW block leaves the strip
with a wide remaining row.

### Левая панель: расщепление модулей

`view/schematic.rs` пробивал лимит 400 строк (был 727). Расщеплено
на focused-модули, каждый под потолок:

- `view/schematic.rs` (~390 строк) — каркас левой панели: общая
  верхняя рамка с full-width статусной шапкой, CPU-группой слева и
  центральной колонкой (`Буфер данных` / `Регистр признаков` /
  `Мультиплексор` / `Регистр состояния`) справа, затем нижний ряд
  `Быстрый доступ` + speed switch. `mux_panel` и
  `speed_panel` остаются one-liner-делегатами в свои модули.
- `view/chips.rs` (~250 строк) — чистые widget-билдеры под одиночные
  плашки на плате: `schematic_readout` (134×60, 20 px hex value),
  `schematic_wide_readout` для растянутого блока признаков,
  `schematic_mnemonic_readout`, `flag_strip` / `flag_dot`,
  `device_chip` и `functional_block`.
- `view/mux.rs` (~300 строк) — мультиплексор: внешний заголовок,
  отдельные framed-группы W/Z и РОН grid B/C/D/E/H/L, SP/PC
  footer, новый ряд «Схема инкремента-декремента».
- `view/lamps.rs` (~170 строк) — ряд из 11 control-ламп с живой
  привязкой к сигналам и горизонтальными подписями.
- `view/current_command.rs` (~170 строк) — блок «Текущая команда»
  под сигналами управления: код, мнемоника, операнд, длина, тип и
  адресация команды по текущему `PC`.
- `view/speed.rs` (~384 строки) — четырёх-ступенчатый переключатель
  скорости, segment gauge и тесты его геометрии/цветов.
- `view/notices.rs` (~90 строк) — пассивные floating notice overlays
  для HLT, file error и legacy-format heads-up.
- `view/menu.rs` (~350 строк) — top title/menu bar, menu visibility
  toggle, divider gap, dropdown routing and caption buttons.
- `view/menu_dropdowns.rs` (~245 строк) — dropdown rows for `Файл` and
  `МП-Система`, including muted legacy-format notes and disabled HLT
  reset state.
- `view/modal.rs` (~250 строк) — modal overlay несохранённых
  изменений, его backdrop, кнопки и focused-button styling.
- `view/menu_labels.rs` (~10 строк) — localized labels for inactive
  top-level menu categories (`Вид`, `Настройки`, `Справка`) plus a
  regression test.
- `view/mod.rs` (~260 строк) — root stack: меню, основная раскладка,
  transient overlays и порядок слоёв.

### Left schematic layout

The left UI zone is split into two visual bands:

- the shared upper frame: CPU row (`PC` / `SP` / `T` / clickable
  `HLT`) plus the right-aligned textual `Статус` line across the full
  top row, then PSW/flags on the left, the `Регистры и операнды` grid,
  `Цикл и такт`, `Внутренние тайминги`, `Сигналы управления`,
  `Текущая команда`, plus the right-side central column with same-width
  `Буфер данных` and `Регистр признаков`, the multiplexer, and the
  `Регистр состояния` readout below it;
- the bottom row: «Быстрый доступ» on the left and the speed switch on
  the right. The main schematic frame uses 8 px spacing to this bottom
  row so its lower border lands closer to the lower edge of the
  right-side register editor.

The status-byte text is shown as its own `Регистр состояния` block under
the multiplexer. The textual status label (`Загрузка опкода`, `Чтение
памяти`, etc.) stays in the hover tooltip rather than in the top row.
The tooltip does not repeat the status-byte value; it uses the darker
`status_tooltip_style` surface, keeps a 12 px viewport padding so it
does not snap flush to the window edge, and colours only the current
status label blue. Regular hover tooltips use the same dark fill via
`inset_style`.

The free-form top-row `Статус` string is not constrained to the
multiplexer column anymore: it is rendered in the upper frame header, so
long open-file paths can use the full width left after the `PC` / `SP`
/ `T` / `HLT` cluster. If the status string ends with
`(старый формат)`, the suffix is displayed as a muted `старый формат`
note without parentheses.

This is a layout-only change. All readouts still come from the same
`AppSnapshot` / `Cpu8080State` fields, and the clickable chips still
emit the same `Message` variants as before.

### Russian labels on the schematic plate

The blocks on the left panel carry full Russian captions instead of
the older English/abbreviated set. Russian words run longer, so
`schematic_readout` and `functional_block` were resized from `110×56`
to `134×60` and the caption font dropped from 12 px to 11 px so
«Буферный регистр 1», «Регистр команд», и «Буфер данных» fit without
truncation. The 24 px / 20 px monospace value rows are unchanged.

| Слот | Старая подпись | Новая подпись |
|---|---|---|
| Аккумулятор (functional block) | `Accumulator` | `Аккумулятор` |
| Буферный регистр 1 (functional block) | `Buf. Reg 1` | `Буферный регистр 1` |
| Буферный регистр 2 (functional block) | `Buf. Reg 2` | `Буферный регистр 2` |
| Регистр признаков (readout) | `Flag Reg.` | `Регистр признаков` (8 бит `S Z 0 AC 0 P 1 C` — стандартный формат PSW low byte) |
| Регистр команд (readout) | `Instr. Reg` | `Регистр команд` (`last_fetched_opcode`) |
| Д/Ш команд (readout, рядом с РК) | — | `Д/Ш команд` (мнемоника `last_fetched_opcode`) |
| Буфер данных (центральная колонка) | `Data Buffer` | `Буфер данных` (`last_data_bus_byte`) |
| Буфер адреса (первый слот второго ряда «Регистры и операнды») | — | `Буфер адреса` (`last_address_bus`) |
| Цикл и такт (нижняя плашка, школьная семантика) | — | `Цикл (M)` / `Такт (T)` (синтез по `machine_cycle::layout_for`) |
| Внутренние тайминги (нижняя плашка, datasheet) | `Cycle` / `Tick` | `Тактов всего` / `Такт инстр. (datasheet)` / `Линейная фаза` |
| Текущая команда (под сигналами управления) | — | `Код` / `Команда` / `Операнд` / `Длина` / `Тип` / `Адресация` по байту `memory[PC]` |
| Индикатор останова в статус-строке | `HLT ON` / `HLT OFF` | `HLT ВКЛ` / `HLT ВЫКЛ` (кликабельный — переключает флаг HLT) |

### Reference-schematic blocks added to the left panel

Three читаемых блока были добавлены к левой плашке, чтобы её
геометрия совпала с референсной схемой КР-580, по которой
пользователь сверяется:

- **«Д/Ш команд»** — Дешифратор/Шифратор команд, instruction
  decoder. Сидит во втором ряду группы «Регистры и операнды»,
  прямо справа от «Регистр команд», и показывает **мнемонику** того байта, что
  лежит в IR: где РК показывает `3C`, Д/Ш показывает `INR A`.
  Декодируется через `decode_opcode(byte).mnemonic` — тот же
  путь, что и колонка мнемоник в memory list. На
  недокументированных байтах падает в `-`, чтобы readout никогда
  не оставался пустым. Рендерится через
  `schematic_mnemonic_readout` — ту же шасси что
  `schematic_readout` (134×60, 11 px подпись), но с 14 px моно
  значением вместо 20 px: длинные мнемоники типа `MVI B, d8`,
  `LXI B, d16`, `JNZ adr` (8–10 символов) перестали выходить за
  пределы капсулы.
- **«Текущая команда»** — summary strip под «Сигналы управления».
  Декодирует байт по текущему `PC` (`cpu.memory.read(cpu.pc)`) и
  показывает шесть центрированных колонок на weighted-сетке: короткие
  поля компактнее, `Тип` и `Адресация` немного шире, а зазор между ними
  адаптируется по длине текста.
  hex-код, команду, операнд, длину (`1 байт`, `2 байта`, `3 байта`),
  грубую категорию (`управление`, `пересылка`, `арифметика`,
  `логика`, `переход`, `стек`, `ввод/вывод`) и тип адресации
  (`неявная`, `регистровая`, `непосредств`, `прямая`,
  `косвенная`). На холодном старте `00` отображается как
  `NOP / - / 1 байт / управление / неявная`, как в референсной
  схеме. Это намеренно не IR: после загрузки файла IR ещё может
  хранить `00`, но текущая команда уже определяется байтом в RAM по
  адресу `PC`.
- **«Буфер адреса»** — стоит первым слотом второго ряда группы
  «Регистры и операнды» и зеркалит **физический Буфер Адреса**, не PC. Источник —
  `cpu.last_address_bus` (см. раздел «Last-bus tracking» ниже): на
  реальном чипе это латч между внутренней 16-битной шиной и
  внешними пинами A0–A15, и через него по очереди проходят PC (во
  время M1 fetch), HL/SP (operand fetch / стек), 16-битный
  immediate (LDA/STA/LHLD/SHLD/JMP/CALL). Раньше readout
  безусловно показывал `cpu.pc` — это совпадало со школьным
  эталоном **только** на M1 fetch и расходилось на STA/LDA/HLT;
  теперь совпадает на всех инструкциях.
- **«Схема инкремента-декремента»** — добавлена нижней строкой
  в `mux_panel` под SP/PC. На реальном чипе это вспомогательный
  сумматор, что шагает PC по длине текущей инструкции в фазу
  fetch (и через который HL/SP проходят на INX/DCX). Здесь
  показываем шаг PC: `+1` для одно-байтных опкодов, `+2` для
  опкодов с одним байтом операнда (`MVI r, d8`, `IN`, `OUT`),
  `+3` для опкодов с 16-битным операндом (`LXI`, `JMP`, `CALL`,
  `LDA`, `STA`, `LHLD`, `SHLD`). Считается через
  `decode_opcode(byte).size`, а на недокументированных байтах
  падает в `+1` — той же стратегии следует декодер мнемоник в
  memory list.

Панель «Мультиплексор» теперь рисуется как на учебной схеме: внешний
outline-фрейм держит только общий контур и центрированный заголовок, а
внутри лежат отдельные подблоки с небольшими вертикальными зазорами.
W/Z и РОН получили framed-группы в виде таблиц: общий контур, заголовок
секции и две колонки с внутренним вертикальным разделителем; РОН
остаётся двухколоночным, хотя референсная схема местами рисует три
колонки. Заголовки этих двух секций имеют чуть увеличенный вертикальный
padding, чтобы таблица читалась свободнее. Строки `УС`, `СК` и
`Инкремент-декремент` лежат в одной нижней
framed-группе с внутренними 1 px разделителями, а не в трёх отдельных
скруглённых chips. Hover/selected-fill у РОН-ячеек занимает всю ячейку
без пустых зазоров, а поверх него ячейка восстанавливает нейтральную
1 px grid-линию тем же цветом, что и разделители. Поэтому подсветка не
ломает контур framed-группы и не превращает рамку в цветной hover-border.
Одиночный клик по значению B/C/D/E/H/L переводит ячейку в
inline-редактор, как value-cell в «Содержимое ячеек ОЗУ». Одиночный
клик по остальной области ячейки только выбирает этот регистр в правом
редакторе, а двойной клик по любой точке ячейки тоже входит в
inline-редактирование. Сам inline input занимает тот же value-слот, что
и обычный текст, поэтому цифры не скачут влево при входе в режим
редактирования. Enter/Shift+Enter после коммита идут по этой же
визуальной сетке вперёд/назад, а на краях закрывают только режим
редактирования.
Зона заголовка уплотнена, поэтому общий размер мультиплексора не растёт
из-за этих промежутков.

Блок `АЛУ` (220×86, лежал над мультиплексором) удалён: на референсной
схеме, по которой пользователь сверяется, отдельной плашки АЛУ
нет — арифметика читается через статус-строку (`PSW`, `T`) и через
зажигание ламп `F1`/`F2` на нижнем ряду. Дублирующая большая плашка
со значениями `A` и `HL` нагружала левую панель, не добавляя
информации, которую нельзя прочитать из соседних блоков.

### Last-bus tracking (РК / Д/Ш / Буфер данных / Буфер адреса)

Четыре readout'а на левой плашке — «Регистр команд» (РК),
«Дешифратор/Шифратор команд» (Д/Ш), «Буфер данных» и «Буфер
адреса» — раньше читались через look-ahead в RAM по PC
(`memory.read(pc)` для байта, `pc` для адреса). Это совпадало со
школьным референсным эмулятором **только** на T1 первого M1, до
выполнения инструкции, и расходилось во всех остальных случаях:

- после `HLT` PC шагает за HLT, RAM по новому PC лежит 00 (NOP),
  и наш RP показывал `00` — а школьный показывал `76` (опкод HLT,
  потому что РК на чипе хранит **последний загруженный** опкод
  до начала следующего M1, которого после HLT не будет);
- после `STA 0x4000` адресный латч на чипе хранит `0x4000`, а у
  нас readout зеркалил новый PC;
- после `MOV A, (HL)` буфер данных на чипе хранит прочитанный
  байт, а у нас был байт по PC.

`core` теперь несёт три «латч-зеркала» прямо в `Cpu8080State`:

- `last_fetched_opcode: u8` — зеркало физического Регистра Команд.
  Обновляется в одном-единственном месте — `state::fetch_opcode`,
  который вызывает `execute_instruction_boundary` на M1.
- `last_data_bus_byte: u8` — зеркало Буфера Данных D7-D0. Любой
  байт через шину (read/write) проходит через `state::bus_read`
  / `state::bus_write` и оседает здесь.
- `last_address_bus: u16` — зеркало Буфера Адреса A0-A15. Те же
  helper'ы пишут сюда адрес.

Дисциплина: исполнитель **обязан** ходить в память только через
`bus_read` / `bus_write` / `bus_read_word` / `fetch_opcode`. Прямой
`self.memory.read/write` остался запрещённой формой (есть `peek`
для UI/диагностики, который не трогает латчи). Регрессионные
тесты в `crates/core/tests/last_bus_residue.rs` пинят семантику для
`MVI`/`STA`/`HLT`/`MOV r,r`/`MOV A,(HL)`/Reset; если кто-то снова
обойдёт латчи, тест на `HLT` упадёт первым (после HLT РК должен
держать 76, не байт по новому PC).

UI читает четыре readout'а из этих полей напрямую:
`schematic_readout("Регистр команд", last_fetched_opcode)`,
`schematic_mnemonic_readout("Д/Ш команд",
decode_opcode(last_fetched_opcode))`, `schematic_readout("Буфер
данных", last_data_bus_byte)`, `schematic_readout("Буфер адреса",
last_address_bus)`. Это закрывает четыре расхождения со школьным
эталоном одной точкой контроля в core.

### Два счётчика: «Цикл и такт» + «Внутренние тайминги»

Нижняя управляющая строка под «Регистры и операнды» несёт **два**
блока с разной семантикой: «Цикл и такт» прижат к левому краю, а
«Внутренние тайминги» — к правому краю этой же строки. Раньше тут жил
один блок («Цикл / Такт»),
который читался как общий T-states счётчик + линейная фаза. Школьный
эталон в той же позиции рисует **M-цикл и T-фазу внутри M-цикла**
(M1, M2, M3 + T1..T5) — это физическое табло КР-580, и пользователь
сверяется именно с ним. Сравнение «наш vs школьный» спотыкалось:
на NOP школьный рисовал M=1, T=1..4, а наш — Цикл=4, Такт=0..3 (то
же по сути, но расходящееся числами и заголовками).

Решение — два блока с разной семантикой, чтобы было ясно, что
смотрит на школьный эталон, а что на datasheet:

| Блок | Заголовок | Что показывает | Источник |
|---|---|---|---|
| 1 | **«Цикл и такт»** | школьная семантика, как на физическом табло КР-580 | `core::machine_cycle::layout_for` (для HLT = `[4]`) |
| 2 | **«Внутренние тайминги»** | datasheet-точные значения нашей внутренней модели | `cpu.cycle_count`, `cpu.tact_phase`, `last_completed_tact_phase` + datasheet-layout |

#### Блок 1: «Цикл и такт» (школьная семантика)

Две строки. Совпадает датчик-в-датчик со школьным эмулятором —
это его главная задача. Внутри панели добавлены симметричные
невидимые spacer'ы сверху и снизу, поэтому внешний блок занимает ту
же высоту, что и соседние «Внутренние тайминги», а две строки стоят
по центру вертикально.

- **Цикл (M)** — номер текущего M-цикла внутри инструкции, **с 1**.
  Берётся через школьный layout `layout_for(opcode)` и
  `position_for(layout, taken, phase)`. Для NOP/MOV r,r/ADD r/HLT —
  всегда 1 (одна M1). Для LXI/JMP/MVI A,M — 2 или 3 (M1 fetch
  + M2/M3 чтение операнда). Для CALL — до 5 (M1 + 2 fetch
  операнда + 2 push). Когда инструкция стоит на границе
  (`tact_phase == None`) — fallback на `last_completed_tact_phase`,
  чтобы блок не сбрасывался в `-` после HLT.
- **Такт (T)** — номер T-фазы **внутри текущего M-цикла**, с 1
  (T1, T2, T3, T4, ...). Не сквозной номер по инструкции — каждая
  новая M фаза сбрасывает T в 1. Для NOP это T1..T4 в M1. Для
  LXI B,d16 (10T = 4+3+3): T1..T4 в M1, потом T1..T3 в M2, потом
  T1..T3 в M3. Источник — `cpu.last_completed_tact_phase` через тот
  же школьный layout с клампингом до `total_t_states - 1`.
  Школьное табло после остановки КР-580 удерживает на индикаторе
  ровно эту T-фазу — «горящий такт». Для HLT клампится до T4 (как
  делает школьный эмулятор — M2 halt-acknowledge не отображает,
  потому что это «бесконечное ожидание прерывания», не реальный
  bus cycle).

#### Блок 2: «Внутренние тайминги» (datasheet)

Три строки. Datasheet-точная информация по нашей внутренней модели,
без школьных сокращений. Чтобы видно было, что у нас на самом деле
происходит.

- **Тактов всего** — `cpu.cycle_count`. Сквозной T-states счётчик
  от начала программы (или с последнего сброса). Растёт на 4 за
  NOP, на 7 за HLT (полный datasheet, включая M2 halt-acknowledge),
  на 17 за CALL, на 18 за XTHL, и т.д. Это та же величина, что в
  колонке «STATES» Intel datasheet, только просуммированная по
  всем выполненным инструкциям. До первого выполненного такта
  отображается `-`, а не `0`, чтобы холодный старт не выглядел как
  уже измеренное время программы.
- **Такт инстр. (datasheet)** — номер такта **внутри текущей
  инструкции** по полной datasheet-длительности, **с 1**. Для HLT
  даёт 7 (а не 4 как школьный «Такт (T)»), потому что наш
  внутренний слой не обрезает halt-acknowledge цикл. Для остальных
  опкодов совпадает со школьным «Такт (T)» — datasheet-длительность
  по сумме layout'а равна `InstructionTiming::t_states_taken`.
  Берётся через служебный layout `datasheet_layout(opcode)`,
  который для HLT возвращает `[7]` (склеенный M1+M2 одним блоком),
  для остальных — `layout_for(opcode)`.
- **Линейная фаза** — `cpu.tact_phase` (если идёт исполнение) или
  `cpu.last_completed_tact_phase` (если завершено) — **индекс с 0**:
  `0..total-1`, где total = datasheet-длительность инструкции. Для
  NOP это 0..3, для HLT — 0..6, для CALL — 0..16, для XTHL — 0..17.
  Звёздочка `*` после числа означает «инструкция уже завершена,
  активного исполнения нет, показано последнее зафиксированное
  значение». Без звёздочки — активное исполнение. Это поле
  сохраняется в `.580` snapshot, поэтому формат — индекс с 0 (как
  у массивов), не с 1.

Зачем «Линейная фаза» нужна отдельно от «Такт инстр. (datasheet)»:
первая — индекс в нашем внутреннем буфере, который сохраняется в
snapshot и нумеруется с 0; вторая — человеко-читаемый номер такта
для отладки и сверки с Intel manual, нумеруется с 1. Они
отличаются на 1 (с 0 vs с 1), а звёздочка у «Линейной фазы»
сообщает, идёт ли исполнение прямо сейчас.

#### Чем отличаются блоки

«Цикл и такт» = школьный, для совпадения с физическим табло.
«Внутренние тайминги» = datasheet-точные, для отладки и сверки с
Intel manual. Они отличаются:

- **На HLT** — школьный «Такт» = 4 (только видимый M1), datasheet
  «Такт инстр.» = 7 (M1 fetch + M2 halt-ack), «Тактов всего»
  растёт на 7.
- **На сложных инструкциях** — школьный «Такт» сбрасывается на
  каждом новом M-цикле (T1..T4 в M1, потом T1..T3 в M2, ...), а
  datasheet «Такт инстр.» растёт сквозно (1..10 для LXI без
  сбросов).
- **На простых однотактовых** (NOP, MOV r,r, ADD r) — почти
  одинаково, отличаются только нумерацией: школьный T=1..4,
  «Линейная фаза» = 0..3, «Такт инстр.» = 1..4.

#### Поле `cpu.last_completed_tact_phase`

Источник — поле `cpu.last_completed_tact_phase: Option<u8>`, которое
core фиксирует в трёх точках (`step_instruction` атомарный путь,
`step_instruction` flush после walking, каждый `step_tact`). Без
него UI после `HLT` падал в `-`/`1`, а школьный эталон удерживает
на табло именно T4 первого M1 — последний горящий такт перед
остановкой.

Используется как fallback в обоих блоках:
- Школьный «Цикл (M)» при `tact_phase == None` берёт
  `last_completed_tact_phase`, чтобы не сбрасываться в `-` после
  HLT или на границе инструкции.
- Школьный «Такт (T)» — всегда читает `last_completed_tact_phase`
  (это его основной источник, не fallback), потому что школьное
  табло показывает именно «последнюю выполненную фазу».
- «Линейная фаза» при `tact_phase == None` рисует
  `last_completed_tact_phase` со звёздочкой (`6*`), чтобы
  отличить «активная фаза идёт» от «последняя выполненная,
  активной нет». Без fallback'а там стоял бы `-` и пользователь
  думал, что счётчик отвалился.

#### Persistence

timing-TLV (тег `0x08`) расширен до variable-length
`8 | 9 | 10` байт. Старые файлы (8 байт = только `cycle_count`,
9 байт = `+ tact_phase`) грузятся как раньше — `last_completed_tact_phase`
у них остаётся `None`. Новый 10-байтовый формат несёт обе фазы.
Особый случай (`tact_phase == None`, `last_completed_tact_phase == Some`)
закодирован sentinel'ом `0xFF` в slot[8]: реальная T-фаза 8080 не
превышает ~38, поэтому 0xFF свободна как маркер «активной нет».
Round-trip покрыт тестами `snapshot_roundtrips_last_completed_*` и
`snapshot_loads_legacy_v1_payload_without_last_completed`.

#### Источник раскладок M-циклов

Таблица расклада M-циклов лежит в `core/src/machine_cycle.rs`. Для
каждого документированного опкода известна последовательность
длин M-циклов (`&[4, 3, 3]` = M1=4T, MR_lo=3T, MR_hi=3T = 10T
всего). Для условных инструкций (Rcond / Ccond / Jcond)
предусмотрены **две** последовательности (taken / not-taken).
Сумма длин совпадает с `InstructionTiming::t_states_taken` —
это пинится `layout_sums_match_decode_timing_for_all_documented_opcodes`,
тестом, который проверяет все 244 документированных опкода. Если
кто-то поменяет тайминг в `decode.rs` без правки таблицы здесь
(или наоборот), тест упадёт.

Исключение — HLT (0x76): datasheet даёт 7T (M1=4 fetch + M2=3
halt-ack), но школьный layout = `[4]` (только видимый M1),
потому что школьный эмулятор M2 halt-ack не отрисовывает.
Расхождение layout-суммы (4) и `t_states_taken` (7) намеренное.
Поэтому UI имеет служебный `datasheet_layout(opcode)` в
`view/cycles.rs`, который для HLT возвращает `[7]` (склеенный
M1+M2), а для остальных — `layout_for(opcode)`. Это разделение —
суть всей истории «школьный Такт=4 vs datasheet=7».

#### UI-логика

Лежит в `view/cycles.rs` (~270 строк с расширенной документацией,
отдельный модуль, чтобы `schematic.rs` не пробил 400-строчный
потолок). Берёт байт из `cpu.last_fetched_opcode` (а не RAM по
PC — иначе после HLT расклад M-циклов был бы для NOP),
декодирует его, выбирает раскладку (школьную для блока 1,
datasheet для блока 2), переводит линейную фазу в M/T. Для
нелегальных опкодов оба блока показывают `-`. Решение taken vs
not-taken — эвристическое: пробуем сначала taken (полный путь,
в нём больше M-циклов и на ранних фазах M1/M2 ответы для taken
и not-taken совпадают), если не попали — берём not-taken. Это
даёт визуально правильный M/T для обоих исходов условных
инструкций без моделирования флаг-теста в середине инструкции.

Что осталось без перевода:

- `PC`, `SP`, `T` в статус-строке — стандартные мнемоники чипа
  (Program Counter, Stack Pointer, Tact). Они одни и те же в любом
  8080-учебнике на любом языке.
- Имена регистров `A`, `B`, `C`, `D`, `E`, `H`, `L`, `HL`, `W`, `Z` —
  обозначения регистров по даташиту 8080.
- Лампы управления (`F1`, `F2`, `SYNC`, `READY`, `WAIT`, `HOLD`,
  `INT`, `INTE`, `DBIN`, `WR`, `HLDA`) — английские мнемоники
  выводов корпуса 8080. Перевод их сделал бы строчку нечитаемой
  для всех, кто работал с чипом по любой документации.
- `PSW` — Program Status Word, аббревиатура в одном ряду с PC/SP.

### Status-bar messages

Свободный текст в нижнем статус-баре (`self.status`) переведён на
русский для всех состояний, которые видит пользователь:

| Контекст | Старый текст | Новый текст |
|---|---|---|
| Стартовое состояние (`DesktopApp::new`) | `Ready` | `Готов` |
| Worker отрапортовал `Stopped` | `Stopped` | `Остановлен` |
| Worker отрапортовал `HaltStateChanged` | `CPU halted` | `ЦП остановлен` |
| `toggle_run` на пустой странице | `No program at <PC>` | `Нет программы по адресу <PC>` |
| `TactAdvanced` (пошаговый такт) | `Tact <n> cycle <m>` | `Такт <n> цикл <m>` |
| Поиск по памяти, пустой ввод | `Enter a hex pattern to search for` | `Введите hex-шаблон для поиска` |
| Поиск нашёл совпадение | `Found pattern <p> at <addr>` | `Найден шаблон <p> по адресу <addr>` |
| Поиск ничего не нашёл | `No addresses match <p>` | `Нет адресов, соответствующих <p>` |

Не переведено сознательно:

- `IN <port> -> <value>` / `OUT <port> <- <value>` — формат логов
  ввода-вывода, имитирующий вывод 8080-эмуляторов. Любой текст в
  таблицах рядом (мнемоники инструкций) тоже остаётся английским,
  чтобы статус-бар не показывал смесь языков для одной и той же
  семантической операции.
- Мнемоники инструкций в `InstructionBoundaryReached` (`MOV at
  0123`, `JMP at 0050`) — это имена опкодов 8080 из даташита,
  такая же категория, как имена регистров и контактов.
- Текст ошибок от ядра/шины — попадает в статус-бар «как есть» и
  одновременно дублируется в плавающее уведомление через
  `humanize_error`, который сам переводит распознанные шаблоны на
  русский. Двойной перевод исходного текста разорвал бы соответствие
  между лог-баром и баг-репортами.

## «Содержимое ячеек ОЗУ» follows PC during Run

While the run is armed, the highlight in the memory list (and the
address spinner / inline value buffer) tracks `cpu.pc` automatically.
Implementation:

- The actor publishes one `StateChanged` per executed instruction.
- The 100 ms `Message::Tick` subscription folds those snapshots into
  `DesktopApp` via `pull_events` → `apply_snapshot`.
- After draining the events, `Tick` calls `follow_pc_during_run` when
  `self.running` is true. The helper compares the spinner's current
  address with `cpu.pc`; if they differ it rewrites the spinner,
  refreshes `memory_value_input`, scrolls the viewport to keep the new
  row in view, and returns the same `scroll_memory_to` task the
  step-instruction button uses.

Two guards protect interactive editing:

- `follow_pc_during_run` does **not** call `sync_pc_to_cursor`. PC is
  the source of truth during a paced run; pushing `SetPc(pc)` back at
  the worker would be a useless round-trip and could race the next
  instruction step.
- The inline value buffer (`memory_inline_value_input`) is left alone
  whenever `focused_input == Some(MEMORY_INLINE_INPUT_ID)`, so a user
  who is mid-edit on a faraway cell does not see their typing wiped
  out. Only the spinner moves; the inline editor catches up the moment
  focus leaves it.

When the worker auto-pauses — `HaltStateChanged`, `ErrorRaised`, or
`Stopped` — the `consume_event` handler clears `self.running`, so the
play/pause icon swaps back to green and the Tick branch stops chasing a
frozen counter.

### Final-tick follow-PC at high speed

At slow paces (e.g. the default 10 Hz) the worker delivers one
`StateChanged` per Tick, so the highlight inevitably catches up. At
high speed (e.g. 1000 Hz) it can deliver a long burst of `StateChanged`
followed by a terminal `HaltStateChanged` / `Stopped` inside the same
100 ms tick. By the time the Tick branch re-reads `self.running` the
flag is already `false`, so the closing `follow_pc_during_run` would
not fire and the highlight would be left on whichever row the last
per-instruction snapshot landed on — visibly mid-program even though
the CPU has actually halted.

`DesktopApp::pending_follow_pc` resolves this:

- `consume_event` sets `pending_follow_pc = true` in every auto-pause
  branch right after clearing `self.running`.
- The Tick handler treats `running || pending_follow_pc` as the
  condition for `follow_pc_during_run`, then consumes the flag
  (`pending_follow_pc = false`) so the helper runs exactly once after
  the run ends.
- When the auto-pause was a halt, `follow_pc_during_run` aims at
  `pc.wrapping_sub(1)` — PC sits one byte past the HLT opcode after
  the halt, but the user expects the highlight on the HLT row itself,
  which is what then gets the red row chrome.

### Halted-row highlight

When `cpu.halted` is true and `pc - 1` points at a `0x76` (HLT) byte,
that row in the memory list paints in red instead of the usual blue
selection: `view::styles::containers::memory_row_container_style`
takes a second `halted` argument and returns a red-tinted background
(`TOKYO_RED` at 0.22 alpha) with the same 6 px corner radius as the
regular selection — no extra border, so the highlight reads as a
peer of the blue selection rather than as competing chrome on top of
it. The address column on the same row also switches to `TOKYO_RED`
so the row reads as a single coherent "the program ended here"
banner. The byte check defends against corner cases where PC sits one
past an unrelated byte after a SetPc on a halted state — the halt
visual follows the actual opcode, not the counter alone.

### Halt-blocked controls (Variant A)

After HLT the action panel's run/pause toggle, `Выполнить команду`
(`step_instruction_and_advance`), and `Выполнить такт`
(`step_tact_and_maybe_advance`) all early-return without doing any CPU
work. Each refusal calls `DesktopApp::raise_halt_notice`, which fills
`halt_notice` with the canonical two-line body
(`Процессор остановлен командой HLT\nСбросьте регистры или флаг HLT`),
arms `halt_notice_dismiss_at = Instant::now() + 8s`, **and** sets
`run_blocked_after_halt = true` — all three in lockstep, from the
single chokepoint, so future halt-block sites cannot forget to arm
the latch. The view paints the string as a framed top-center floating
notice (`view::halt_notice_overlay`, `error_inset_style` body — the
same red-bordered chrome as the file-error overlay, so the user reads
"this is a blocking notice" from the frame alone). The text is
centred horizontally so the second (shorter) recommendation line
sits under the first instead of leaning against the left padding.
The body is wrapped in a `mouse_area` whose `on_press` emits
`Message::DismissHaltNotice`, the global Esc handler clears the
notice between `error_notice` and `pending_action`, and
`Message::Tick` polls `halt_notice_dismiss_at` to auto-dismiss
after 8 seconds — the same fade behaviour the user asked for the
file-error overlay to have, applied to its visual twin.

The notice sits above the menu bar's dropdown band — same `stack!`
pattern as the file menu — so it reads as a discrete UI element
instead of a status-bar line that blends into the dark schematic.
The first line is "what's wrong" (the CPU stopped on a HLT) and the
second line is "what unblocks it" (`Сброс регистров или флаг HLT`)
— diagnosis and recommendation on separate lines so the user can
read either half on its own. The recommendation lists both unblock
paths because the halt bit can be cleared either by `Сброс регистров`
(`AppCommand::ResetCpu`, which also rewinds PC to `0x0000`), by the
dedicated `Сбросить флаг HLT` entry at the bottom of the МП-Система
menu / `Ctrl+Shift+H` (`AppCommand::ClearHalt`, which flips *only*
the halt flip-flop and leaves PC, SP, registers, flags, RAM, and
`cycle_count` exactly where HLT left them), by toggling the HLT
flag from the register editor, or by clicking the `HLT ВКЛ` /
`HLT ВЫКЛ` chip on the status strip directly (the indicator is a
`mouse_area`-wrapped `mono_text` whose press dispatches
`Message::ToggleHalt` — the same chip that reads the halt state out
also flips it). RAM is preserved by all of them, so the loaded
program survives the unblock and runs again from whatever PC it
ends up at.

`Message::ToggleHalt` resolves the toggle direction on the UI side
(reads `cpu.halted`, dispatches `AppCommand::ClearHalt` for the
halted→running leg and the new `AppCommand::SetHalted(true)` verb
for the running→halted leg), then routes both legs through
`dispatch_with_undo` so a press is reversible. The worker's
`SetHalted` handler is the symmetric counterpart to `ClearHalt`:
it disarms the run loop the same way, treats the supplied value as
authoritative, and only emits `HaltStateChanged` when the bit
actually flips so a redundant press is a true no-op for the halt
notice.

`run_blocked_after_halt` is the second half of the contract. The
first time the user attempts a run/step gesture against a halted
CPU they get the explanatory overlay; from that moment on, every
execution chip in the action panel (`Выполнить программу /
Пауза`, `Выполнить команду / Перезапустить программу`, `Выполнить
такт`) renders disabled — `editors::actions_panel` calls
`icon_action_button` with `None` for those four messages, and
iced 0.14 paints the button without hover and ignores clicks when
no `on_press` is attached. The two reset chips (`Сброс ОЗУ`,
`Сброс регистров`) keep their `Some(...)` because the resets are
the way *out* of the latch. The latch outlives the 8-second halt
notice on purpose: the user's contract was "до тех пор пока не
сброшу флаг или регистры" — a fade is not an unblock. Repeat
attempts (e.g. through the menu) re-raise the notice via the same
chokepoint so the explanation comes back even if the original
overlay already faded.

The latch is cleared along three independent edges, mirroring the
notice itself: `apply_snapshot` clears both `halt_notice` and
`run_blocked_after_halt` whenever the new snapshot has
`cpu.halted == false`, so any gesture that flips the halt bit off
(snapshot load, register-editor HLT toggle, etc.) re-enables the
chips automatically; the explicit `Message::ResetCpu` arm clears
the latch before dispatching `AppCommand::ResetCpu`; and the new
`Message::ClearHalt` arm clears the latch before dispatching
`AppCommand::ClearHalt`. The first two paths land through the
worker as a non-halted snapshot anyway — clearing the latch
synchronously in the message arm just makes the next view tick
paint live chips before the round-trip completes, instead of one
frame of stale-disabled chrome.

The notice itself is cleared along the same edges plus the user's
Esc / click and the 8-second deadline; see `clear_halt_notice` in
`app/mod.rs`. The two pieces of state are *armed* together in
`raise_halt_notice` and *cleared* together everywhere except the
fade timer (which clears the notice but leaves the latch armed —
that is the whole point of the latch).

`runtime::memory::sync_pc_to_cursor` also early-returns on halt. The
function normally mirrors a freshly-clicked memory cell into `cpu.pc`
so a subsequent step runs against that byte, but on a halted CPU
there is no next step (the gestures are blocked), and the
`SetPc` round-trip is actively harmful: `dispatch_sync` waits for a
`StateChanged`, which on a halted CPU is the same post-halt snapshot
the worker keeps republishing (`pc = halt_pc + 1`); `apply_snapshot`
then reads that PC back into the spinner and the visible address
jumps one cell forward on every click. Skipping the dispatch lets
the user browse memory freely after HLT — the next reset reattaches
PC to whatever address they end up clicking on.

### Esc reverts unsaved inline memory edits

Inside the inline `Значение` editor, Esc discards a byte the user has
typed but not yet committed with Enter, restoring the buffer to the
byte that actually lives in memory at the highlighted address. The
spinner-side `memory_value_input` is restored alongside the inline
buffer so the side panel stays in sync, and focus stays on the inline
editor so the user can keep typing.

The keyboard subscription emits `Message::EscPressed`; the `update`
router checks `self.focused_input` and calls
`cancel_inline_memory_edit` when the inline editor owns focus,
falling back to closing the open top menu, then `hide_opcode_dropdown`
(the legacy Esc binding) otherwise. Routing in `update` rather than the
`Fn` event listener keeps the listener stateless. With the inline buffer
already matching storage the handler is a no-op so a stray Esc does not
snap the caret to the end of the field.

### Unsaved-changes modal

Discard paths (`Open`, legacy open, `New`, `Import`, and window close)
route through `DesktopApp::pending_action` when `dirty` is set. While
that field is `Some`, the modal layer captures user interaction before
the main update router sees it: emulator shortcuts, arrow keys, opcode
picker gestures, menu actions, and title-bar dragging are ignored until
the modal closes. Passive bookkeeping such as `Tick`, cursor tracking,
and window state updates still flows so the app does not stall.

The modal owns a two-button keyboard ring. Focus starts on `Отменить`,
Tab and Shift+Tab cycle around cancel / confirm, Enter activates the
focused button, and Esc cancels. The confirm label matches the queued
action (`Открыть` for open-file paths, `Создать` for new-file, `Закрыть`
for window close). Legacy open uses the same muted `старый формат` note
as the file menu instead of parenthesized title text. The focused button
reuses the same surface fill as the hover state while keeping the
regular neutral border; mouse clicks on either button still dispatch the
same `CancelDiscard` / `ConfirmDiscard` messages.

The opcode/mnemonic picker uses `opcode_dropdown_style` with a 7 px
radius on all four corners. The popup floats over the memory rows, so
the top edge keeps the same rounding as the bottom edge instead of
looking clipped against the search field.

## Speed switch (left schematic panel)

The schematic board on the left edge of the window carries a paced-run
control in the bottom row, to the right of the «Быстрый доступ»
peripheral frame. It is a four-position stepper inside a legend-framed
`Скорость` panel: the title stays in the frame, the body has a
left-chevron button, a segmented centre gauge, a `N инстр/сек` readout,
and a right-chevron button. The step buttons are compact 36 x 36 px
squares with the same resting background as the app plate, so both
chevrons fit inside the fixed-width frame without adding a grey chip
surface. Their outline uses the same `TOKYO_BORDER` tone as the
surrounding schematic frames. The bottom strip aligns these framed
blocks by their lower border, matching the right-side action panels.
Clicking left/right emits
`Message::SpeedTierChanged(previous/next tier)` and clamps at Slow/Max,
so the underlying four-mode model is unchanged.

The switch replaces an earlier freeform slider (`MIN_STEP_HZ=1` …
`MAX_STEP_HZ=1000`). The slider was honest about the underlying
`SetStepInterval` knob but dishonest about the user-visible result:
above the monitor's refresh rate the same row would still appear
highlighted for one frame regardless of how fast the worker stepped,
so dragging the slider past ~60 Hz only made the program *finish*
faster, not *animate* faster. Named tiers communicate the real
trade-off — pacing for reading, pacing for "just run it", and the
explicit "don't bother animating" opt-in — without inviting the user
to chase a sweet spot that doesn't exist.

| Tier | Label | Resolved Hz | Use |
|---|---|---|---|
| `SpeedTier::Slow`   | Медленно | `SLOW_TIER_HZ = 5`        | One step every 200 ms — read every memory row as PC walks across it. |
| `SpeedTier::Medium` | Средне   | `MEDIUM_TIER_HZ = 20`     | Default. Visibly "the program is running" while the eye still keeps up with each PC update. |
| `SpeedTier::High`   | Высоко   | primary monitor's refresh rate, fallback `HIGH_TIER_FALLBACK_HZ = 60`, capped at `HIGH_TIER_CEILING_HZ = 240` | One instruction per painted frame — finishes as fast as the screen can paint without skipping rows. |
| `SpeedTier::Max`    | Максимум | `MAX_TIER_HZ = 1000`      | Switches the worker to **burst mode** (`RunMode::Burst`). The CPU runs instructions in a tight inner loop bounded by a 16 ms wall-time slice and the per-session budget; the UI sees one coalesced snapshot per slice instead of one per instruction. The opt-in for "доведи программу до конца, мне не нужно смотреть на каждый шаг" — and unlike the earlier "fast slider" attempt, this one is *actually* faster than High, because it stops paying the per-instruction timer + crossbeam + redraw round-trip. |

| Property | Value |
|---|---|
| Storage | `DesktopApp::speed_tier: SpeedTier` |
| Default | `DEFAULT_SPEED_TIER = SpeedTier::Medium` |
| Emit | `Message::SpeedTierChanged(SpeedTier)` from left/right chevron buttons |
| Resolve | `tier_hz(tier) -> u32` (in `app/mod.rs`) |
| Dispatch | `AppCommand::SetStepInterval(Duration::from_micros(1_000_000 / hz))` followed by `AppCommand::SetRunMode(...)` |

Switching to a tier emits **two** worker commands: the
`SetStepInterval` keeps the per-instruction delay honest for the
paced tiers, and the `SetRunMode` toggles between
`RunMode::Paced` (Slow / Medium / High — one instruction per tick,
one snapshot per instruction) and `RunMode::Burst { slice: 16 ms }`
(Max — tight inner loop bounded by wall-time, one coalesced
snapshot per slice). The two-command shape lets the actor pick its
deadline based on `run_mode()` without inferring it from the
interval value, which keeps the worker honest if the UI ever wants
to mix burst with a non-default slice.

`tier_hz` encapsulates the platform query for the High tier:
`platform::primary_monitor_refresh_hz()` reads `dmDisplayFrequency`
from `EnumDisplaySettingsW` on Windows; on non-Windows it is a stub
returning `None`. Either way callers fall through to
`HIGH_TIER_FALLBACK_HZ = 60`, then clamp into
`[HIGH_TIER_FALLBACK_HZ, HIGH_TIER_CEILING_HZ]` so a virtual / headless
display reporting `0` or `1` Hz cannot make the worker run at one
instruction per second, and a stuck driver claiming, say, 999 Hz
cannot make it burn cycles on UI ticks the panel can't actually
display. The Max tier sidesteps this resolver entirely and returns
`MAX_TIER_HZ` directly — its semantics are "saturate the worker, ignore
the monitor".

The centre gauge always renders 20 small vertical segments. The active
tier lights a centred band of 5 / 10 / 15 / 20 segments for Slow /
Medium / High / Max respectively. Every segment keeps a fixed height
from the whole-gauge wave envelope, so changing tiers only recolours
segments instead of resizing them. Neighbouring inactive bars receive
a very light magenta halo that fades back into `TOKYO_SURFACE_2` over
four segments. The text below shows the resolved
`tier_hz(active)` value as CPU instructions per second: the paced run loop
executes one CPU instruction per worker tick, while `step_tact` remains
the separate tact-level debug control. The chevron buttons use the same
dark `TOKYO_SURFACE` hover fill as the other control chips, with the
panel border colour left unchanged. The handler in `app/mod.rs`
stashes the tier on `DesktopApp::speed_tier`, resolves it through
`tier_hz`, and ships `SetStepInterval` + `SetRunMode` to the worker.
The worker clamps once more at `MIN_STEP_INTERVAL = 1ms` (re-exported
from `k580_app::actor`) before it overwrites `Emulator::step_interval`.
The next `select!` iteration re-arms the timer with the new interval
(paced) or slice (burst), so clicking a chevron while a program is
running adjusts the visible animation rate immediately — without
restarting the program or losing the run-armed state.

The UI subscription tick interval is also coupled to the resolved Hz
(see `subscription` in `app/mod.rs`) with a 16 ms floor. Slow / Medium
/ High deliver one snapshot per tick because the worker is paced and
matches the subscription cadence one-to-one. Max keeps the same 16 ms
subscription, but the worker is no longer paced — each Tick drains a
single coalesced snapshot that already represents *thousands* of
executed instructions. That visible jumpiness across memory rows is
the honest signal that "Максимум" gave up frame fidelity to gain wall
time. A `Stop` press still lands within one slice (≤ 16 ms) because
the actor's `select!` re-arms its deadline to `slice` rather than
parking inside the inner loop.

### Run modes (`RunMode`)

`k580_app::RunMode` controls how the worker dispatches CPU work
during a paced `Run`. Two variants:

- `RunMode::Paced` — the default, used by the Slow / Medium / High
  speed tiers. `Emulator::tick()` executes exactly one instruction,
  publishes one `InstructionBoundaryReached` and one `StateChanged`,
  and the actor re-arms its `after(step_interval)` deadline. The UI
  sees every step. This is the path that lets the highlighted memory
  row walk one cell at a time.
- `RunMode::Burst { slice }` — used by the Max speed tier. `tick()`
  enters a tight inner loop that keeps stepping until any of:
  - `slice` wall-time has elapsed (re-checked every 64 instructions
    so `Instant::now()` doesn't dominate the hot loop),
  - the per-session `MAX_INSTRUCTIONS_PER_RUN` budget is exhausted,
  - the CPU halts,
  - or an instruction errors.
  Only the **final** snapshot is published; the per-instruction
  `InstructionBoundaryReached` events are deliberately suppressed so
  the iced side stops paying the per-step redraw cost. The actor
  re-arms its `after(slice)` deadline, which doubles as the
  responsiveness floor for `Stop`: a press lands within one slice.

`AppCommand::SetRunMode(mode)` switches between them. The emulator
stores the mode in `Emulator::run_mode` and exposes it through
`run_mode()` so the actor can pick the deadline (`step_interval` for
paced, `slice` for burst) on the next `select!` iteration. The
worker also floors `slice` at 1 ms to mirror the `SetStepInterval`
floor — out-of-range zero would degenerate the timer arm into a busy
loop.

## Keyboard shortcuts

The UI exposes the following shortcuts. Modifier names follow iced's
`Modifiers::command()` convention: Ctrl on Windows/Linux, ⌘ on macOS.

### Memory cell editor (address + value pair)

| Shortcut | Effect |
|---|---|
| Enter (in address field) | Jump to the typed address; remembers the typed substring as the search pattern. |
| Enter (in value field) | Write the typed byte into the currently selected address. |
| Ctrl+Enter | Find the next address whose 4-digit hex form contains the cached search pattern, advancing past the current cell and wrapping around 64 KiB. The pattern is captured before the first plain Enter so iterating after an initial jump uses the original short hex (`FF`) rather than the matched address (`00FF`). The pattern is reset whenever the user edits the address field by hand. |
| Alt+Enter | Step to the next sequential address (same as ArrowDown). Never writes memory, never touches the search pattern cache. |
| ArrowUp / ArrowDown (in address field) | Step the highlighted address by one. |
| ArrowUp / ArrowDown (in value field) | Bump the byte in the value field by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to memory until Enter; ArrowUp on `FF` and ArrowDown on `00` are no-ops. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

### Register editor (name + value pair)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the typed register. |
| ArrowUp / ArrowDown (in name field) | Cycle to the previous/next register in `A B C D E H L`. |
| ArrowUp / ArrowDown (in value field) | Bump the byte in the value field by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to the register until Enter; ArrowUp on `FF` and ArrowDown on `00` are no-ops. |
| Tab / Shift+Tab | Cycle focus between the two fields of this panel only. |

The schematic register chips mirror the memory-row editing contract:
single-click on the numeric value of `A`, `B`, `C`, `D`, `E`, `H`, or
`L` opens the inline hex editor; single-click on the rest of the chip
only selects that register in the right-side editor. Double-click
anywhere inside the same chip opens the inline editor. Enter commits
through the same `SetRegister` path as the right-side «Регистр и его
значение» panel, then advances to the next inline target in the same
visual group (`A → B → C` for the schematic buffer row and
`B → C → D → E → H → L` for the mux grid). Shift+Enter commits and
walks the same group backward. At either edge, Enter/Shift+Enter closes
only the inline editor and leaves the register cell selected. Esc
discards the pending byte, closes only the inline editor, and also keeps
the register cell selected. When a register cell is selected but not
editing, Enter opens the inline editor. ArrowUp/ArrowDown inside the
inline field bumps the buffered byte without committing it; outside edit
mode, arrows move only inside the active visual group. The schematic
buffer row responds only to Left/Right (`A/B/C`); the mux grid responds
to Left/Right between columns and Up/Down between rows (`B/C`, `D/E`,
`H/L`). While the inline register input owns the caret, Ctrl+Arrow uses
the same cell navigation instead of moving the text cursor inside the
two hex digits; the target cell remains in inline edit mode. All
readouts for the active register share the same pending
`register_value_input`: typing `B=7F` in the right-side editor updates
the top «Буферный регистр 1» and the mux `B` cell immediately, and
typing inside either schematic copy updates the right-side editor and
the other schematic copy immediately. The CPU register is still written
only on Enter. Selecting either group clears the memory inline focus so
those keys no longer move «Содержимое ячеек ОЗУ». Selecting a memory row
does the inverse: it clears the active register target, so subsequent
unfocused ArrowUp/ArrowDown presses browse the memory list again rather
than continuing to move the register highlight.

### Memory list (the inline value cell of the selected row)

| Shortcut | Effect |
|---|---|
| Enter | Apply the typed value to the selected address. |
| Tab / Shift+Tab | Move the selection to the next/previous address and refocus the inline editor for the new row. |
| Esc | Discard the unsaved byte typed into the inline editor and restore it to the value currently in memory. With no pending edit, falls through to closing the opcode dropdown. |
| ArrowUp / ArrowDown (inline editor focused) | Bump the byte in the inline editor by ±1, saturating at `0x00`/`0xFF`. The byte is *not* written to memory until Enter. |
| ArrowUp / ArrowDown (no editor focused) | Move the highlighted address by one. |
| PageUp / PageDown | Move the highlighted address by 16. |

### Global

| Shortcut | Effect |
|---|---|
| Esc | Routed by `Message::EscPressed`: cancels the unsaved-changes modal first, then closes passive notices / open top menus, reverts an unsaved inline-edit byte when the inline editor owns focus, otherwise hides the opcode dropdown if it is open. |
| Tab / Shift+Tab | Normal focus-cycle inside editor groups; while the unsaved-changes modal is open, cycles cancel / confirm in a closed ring. |
| Enter | Normal submit / inline-edit recovery; while the unsaved-changes modal is open, activates the focused modal button. |
| ArrowUp / ArrowDown | Routed by `DesktopApp::handle_arrow_key` to the editor that currently owns focus (see the panel-specific tables above). With nothing tracked focused they fall back to memory list navigation. |
| PageUp / PageDown | Move the highlighted address by 16, regardless of focus. |

## Focus rings and styling

The two paired panels each form a closed two-element focus ring that Tab
and Shift+Tab simply swap. Focus is moved via
`iced::widget::operation::focus(id)`, and the destination is decided in
`runtime::cycle_focus` based on the id reported by
`iced::advanced::widget::operation::focusable::find_focused()`. The
inline-editor case handles Tab as "advance the selected address and
refocus the same id", which iced re-renders against the new row.

The address spinner and the register spinner are wrapped in `mouse_area`
to surface hover events. Focus state is not exposed by `text_input` in
iced 0.14, so `DesktopApp::focused_input` is a best-effort cosmetic
marker: it is updated whenever the user types into a known input, when
they explicitly Tab to one, or when they click an inline memory row. This
drives the same blue/cyan/border colour scheme that iced applies to the
plain right-hand text input, so both visual styles match.

Two gestures clear the caret without going through any of the
acquire-side write paths and so leave `focused_input` stale on their
own: Esc (iced consumes it by clearing `state.is_focused` on the
focused text_input) and a left-click in dead space (every text_input
that does not contain the click runs the same clearing branch in
`text_input::update`). Both paths chain a `find_focused_optional()`
operation onto whatever else they do — Esc fires it from
`Message::EscPressed`, dead-space clicks fire it from the
`FocusReconciled(None)` branch — and the `Message::ResolveFocusedTracker`
handler clears `focused_input` iff iced reports no focusable still owns
the caret. The `_optional` variant lives in `runtime::focus_ops` because
the built-in `iced::advanced::widget::operation::focusable::find_focused`
returns `Outcome::None` when nothing is focused, which would silently
drop the message exactly when we need it most. Wrapping the answer in
`Option<Id>` and returning `Outcome::Some(option)` makes the report
unconditional.

The `FocusReconciled(Some(_))` branch never needs the poll: the
two-pass click reconciler in `runtime::focus_ops` already chained
`unfocus_except(hit)` and updated `focused_input` to the resolved id
on the same frame.

## Launch flash mitigation (Windows)

On Windows the desktop window manager paints the client area with the
default white system brush between window creation and the first
GPU-presented frame. To suppress that flash:

1. The window starts with `visible: false` (winit/iced).
2. On `Event::Window::Opened` the app cloaks the HWND through DWM
   (`DWMWA_CLOAK = TRUE`) and then asks iced to switch the window to
   windowed mode. The window now exists and is laid out, but the
   compositor does not show it.
3. After the second `RedrawRequested` the app uncloaks the HWND. By that
   point the wgpu swapchain has presented at least one frame of real
   content.

Cross-platform fall-backs:

- The application root style explicitly paints the iced background with
  `#121320` (`TOKYO_BOARD`) so any frame the OS shows before our wgpu
  surface presents is already in-theme.
- On non-Windows targets `platform::cloak_window` is a no-op.

## Console window suppression

Release builds set `windows_subsystem = "windows"` (via a top-of-file
`#![cfg_attr(...)]`) so launching `k580.exe` from Explorer does not spawn
a stray console window. Debug builds keep the default console subsystem
so `tracing` output stays visible during `cargo run`.

## Window icon

The runtime icon is loaded from `assets/icons/icon-64.png` via
`include_bytes!` and `iced::window::icon::from_file_data`. The full icon
fan-out (`16/32/48/64/128/256` PNGs and a multi-resolution `icon.ico`)
lives in `assets/icons/` and is regenerated by the scripts in
`scripts/`. On Windows, `crates/ui/build.rs` additionally embeds
`icon.ico` into the PE resource section so the `.exe`, Explorer, the
taskbar, and the Start menu all show the icon. See `docs/assets.md` for
the asset pipeline.

## Crate dependencies relevant to the UI

`crates/ui/Cargo.toml`:

- `iced` with the `image` feature (icon decoding) and the `advanced`
  feature (`find_focused` and the `operate` task wrapper) enabled.
- `rfd` for native file dialogs.
- `tracing` and `tracing-subscriber` for diagnostic logging.
- `windows-sys` (Windows only) with `Win32_Foundation`,
  `Win32_Graphics_Dwm`, and `Win32_Graphics_Gdi` features — the first
  two for DWM cloaking and the rounded-corner attribute, the third for
  `EnumDisplaySettingsW` (used by the High speed tier to read the
  primary monitor's refresh rate).
- `winresource` (Windows-only build dependency) for the PE icon resource.
