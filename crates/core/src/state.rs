use crate::{CoreError, Flags, Memory64K, PortBus, RegisterName, Registers};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpu8080State {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub flags: Flags,
    pub memory: Memory64K,
    pub interrupt_request_pending: bool,
    pub interrupt_enable: bool,
    pub interrupt_enable_pending: bool,
    pub halted: bool,
    pub cycle_count: u64,
    pub interrupt_vector_byte: Option<u8>,
    pub tact_phase: Option<u8>,
    /// Последняя выполненная T-фаза текущей (только что завершённой
    /// или активной) инструкции. Раньше UI читал только `tact_phase`,
    /// но это «следующая T в текущем M1, или None между инструкциями».
    /// На границе инструкции `tact_phase` сбрасывается в `None`, и
    /// блок «Машинный цикл» / «T-states (наша)» падает в `-`. Школьный
    /// эталон вместо этого замораживает экран на **последней
    /// выполненной T** (после HLT — на T4 первого M1, на T7 для NOP
    /// и т.д.). Чтобы UI совпадал, отдельно фиксируем здесь линейную
    /// фазу `total - 1` в момент завершения инструкции — её и рисуем
    /// в строке «Такт» блока «Машинный цикл», и она же служит
    /// fallback-ом для «Фаза T (наша)», когда `tact_phase == None`.
    /// `None` означает «ни одной инструкции ещё не выполнено»
    /// (холодный старт / Reset).
    pub last_completed_tact_phase: Option<u8>,
    pub(crate) active_tacts_remaining: u8,
    pub(crate) active_tacts_total: u8,
    /// Последний опкод, успешно прочитанный с шины во время M1
    /// (instruction fetch). Зеркало физического Регистра Команд (РК):
    /// после загрузки опкода РК на чипе хранит этот байт до **начала
    /// следующего M1**, а не до момента, когда PC изменится. Для UI
    /// это семантическая разница: «байт по `cpu.memory.read(pc)`» —
    /// это look-ahead в RAM (что *будет* прочитано), а здесь — то,
    /// что *уже* прочитано и сейчас декодируется. Особенно заметно
    /// после `HLT`: PC шагнул на 0009, в RAM по 0009 лежит 00, и
    /// look-ahead показал бы NOP — а реальный РК хранит 76 (HLT),
    /// потому что это была последняя загруженная инструкция и
    /// `M1` после неё не произойдёт.
    pub last_fetched_opcode: u8,
    /// Последний байт, прошедший по шине данных D7-D0, в любую
    /// сторону (read из памяти, write в память, IN, OUT). Зеркало
    /// физического Буфера Данных: на чипе это латч между внутренней
    /// шиной и внешними пинами, и он держит **последний** байт до
    /// следующего обмена. Для UI заменяет старый «look-ahead» вывод
    /// `cpu.memory.read(pc)`: после `HLT` буфер должен показывать
    /// 76 (опкод HLT, последний байт через шину), а не 00 (что лежит
    /// в RAM после HLT).
    pub last_data_bus_byte: u8,
    /// Последний адрес, выставленный на шину A0-A15. Зеркало
    /// физического Буфера Адреса: на чипе это латч между внутренней
    /// 16-битной шиной и внешними пинами, и через него по очереди
    /// проходят PC (во время M1 fetch), HL (operand fetch), SP
    /// (push/pop), 16-битный прямой адрес (LDA/STA/LHLD/SHLD/JMP/
    /// CALL). Мы не моделируем машинные циклы (см.
    /// `docs/assumptions.md`), но обновление этого поля при каждом
    /// memory access даёт UI правильный «последний адрес на шине»
    /// без сложности M-цикл-модели. После `HLT` PC=0009, но
    /// последний выставленный адрес был 0008 (адрес самого HLT'а) —
    /// именно его UI показывает в «Буфер адреса», совпадая со
    /// школьным эмулятором.
    pub last_address_bus: u16,
}

/// Ручной `Default`, потому что `#[derive(Default)]` дал бы `sp: 0`,
/// а нам нужен школьный дефолт `0xFFFF` (см. комментарий у
/// `Cpu8080State::RESET_SP`). Хочется, чтобы `Cpu8080State::default()`
/// и `reset_cpu()` приводили CPU к одному и тому же холодному
/// состоянию — иначе тесты, persistence-снапшоты и UI будут видеть
/// разные «нулевые» точки в зависимости от того, как было создано
/// состояние. Все остальные поля совпадают со значениями, которые
/// получил бы автодеривированный `Default`.
impl Default for Cpu8080State {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            pc: 0,
            sp: Self::RESET_SP,
            flags: Flags::default(),
            memory: Memory64K::default(),
            interrupt_request_pending: false,
            interrupt_enable: false,
            interrupt_enable_pending: false,
            halted: false,
            cycle_count: 0,
            interrupt_vector_byte: None,
            tact_phase: None,
            last_completed_tact_phase: None,
            active_tacts_remaining: 0,
            active_tacts_total: 0,
            last_fetched_opcode: 0,
            last_data_bus_byte: 0,
            last_address_bus: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionOutcome {
    pub opcode: Option<u8>,
    pub mnemonic: String,
    pub pc_before: u16,
    pub pc_after: u16,
    pub t_states: u8,
    pub halted: bool,
    pub interrupt_accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TactOutcome {
    pub tact_phase: u8,
    pub instruction_boundary: bool,
    pub cycle_count: u64,
}

impl Cpu8080State {
    /// Начальное значение SP после Reset. На физическом 8080 SP не
    /// инициализируется при сбросе — это «неопределённые биты после
    /// питания», и реальный код всегда начинается с `LXI SP, addr` /
    /// `SPHL` до первого PUSH/CALL. Эмуляторы выбирают разные
    /// стартовые значения, чтобы хотя бы первый случайный PUSH без
    /// явной инициализации не затоптал программу:
    ///
    /// - школьный референсный эмулятор КР-580 (по которому
    ///   пользователь сверяется, см. `t2.580` workflow) ставит
    ///   `0xFFFF` — вершину 64K-памяти. PUSH без `LXI SP` сначала
    ///   декрементирует SP, поэтому первая запись пойдёт в
    ///   `0xFFFE`/`0xFFFD`, в самый верх RAM, далеко от любой
    ///   разумной программы;
    /// - наш предыдущий дефолт `0x0000` приводил к тому, что первый
    ///   незаявленный PUSH писал в `0xFFFE` (декремент с
    ///   underflow) — численно совпадает с FFFF-моделью только из-за
    ///   wrapping, но визуально расходится: при загрузке
    ///   программы пользователь видел `SP 0000` в покое, а
    ///   школьный — `FFFF`, и сравнение «наш vs оригинал»
    ///   спотыкалось на первом же кадре.
    ///
    /// Меняем дефолт на `0xFFFF`, чтобы холодный старт визуально
    /// совпадал с референсом. Memory остаётся нетронутой
    /// (`reset_cpu` сохраняет содержимое RAM перед `Self::default()`
    /// — иначе сброс CPU убил бы загруженную программу).
    pub const RESET_SP: u16 = 0xFFFF;

    pub fn reset_cpu(&mut self) {
        let memory = core::mem::take(&mut self.memory);
        *self = Self {
            memory,
            sp: Self::RESET_SP,
            ..Self::default()
        };
    }

    pub fn reset_ram(&mut self) {
        self.memory.clear();
    }

    pub fn request_interrupt(&mut self, vector_byte: u8) {
        self.interrupt_request_pending = true;
        self.interrupt_vector_byte = Some(vector_byte);
    }

    pub fn set_register(&mut self, register: RegisterName, value: u8) {
        self.registers.set(register, value);
    }

    pub fn get_register(&self, register: RegisterName) -> u8 {
        self.registers.get(register)
    }

    pub fn set_memory(&mut self, address: u16, value: u8) {
        self.memory.write(address, value);
    }

    /// Чтение байта через шину A0-A15 / D7-D0 — единственный путь
    /// к памяти из исполнителя инструкций. На реальном чипе адрес
    /// сначала выставляется на адресный буфер, затем считанный байт
    /// проходит через буфер данных; зеркалим оба латча в
    /// `last_address_bus` и `last_data_bus_byte`, иначе UI после
    /// `HLT`/любой инструкции показывал бы «look-ahead» в RAM по PC,
    /// а не последний реально пройденный байт. Сам `Memory64K` не
    /// знает о латчах — это разделение даёт UI «правильный последний
    /// адрес/байт на шине» без моделирования M-циклов
    /// (см. `docs/assumptions.md`).
    pub(crate) fn bus_read(&mut self, address: u16) -> u8 {
        let value = self.memory.read(address);
        self.last_address_bus = address;
        self.last_data_bus_byte = value;
        value
    }

    /// Запись байта через шину. Симметрично `bus_read`: и адрес, и
    /// данные проходят через те же латчи, поэтому UI должен видеть
    /// именно записанный байт, а не RAM-байт по PC.
    pub(crate) fn bus_write(&mut self, address: u16, value: u8) {
        self.memory.write(address, value);
        self.last_address_bus = address;
        self.last_data_bus_byte = value;
    }

    /// Чтение 16-битного слова. На реальном чипе это два machine cycle
    /// подряд (low, потом high) — последняя пара (адрес+байт) и есть
    /// то, что задержится в буферах после операции, поэтому
    /// `last_address_bus`/`last_data_bus_byte` после `bus_read_word`
    /// показывают **верхний** байт по `address+1` (как и у школьного
    /// эталона: после `LXI`/`LHLD` буфер данных держит старший байт
    /// операнда).
    pub(crate) fn bus_read_word(&mut self, address: u16) -> u16 {
        let lo = self.bus_read(address);
        let hi = self.bus_read(address.wrapping_add(1));
        u16::from(lo) | (u16::from(hi) << 8)
    }

    /// Загрузка опкода (M1 fetch). Помимо обновления буферов адреса
    /// и данных, фиксирует байт в `last_fetched_opcode` — зеркало
    /// Регистра Команд (РК): после загрузки чип держит этот байт до
    /// **начала следующего M1**, а не до изменения PC. Поэтому
    /// после `HLT` (когда следующий M1 не наступит) РК должен
    /// продолжать показывать `0x76`, а look-ahead через
    /// `memory.read(pc)` показал бы `0x00` (NOP в очищенной RAM) —
    /// именно эта семантическая разница и закрывает четыре блока
    /// схематика одной точкой контроля.
    pub(crate) fn fetch_opcode(&mut self) -> u8 {
        let opcode = self.bus_read(self.pc);
        self.last_fetched_opcode = opcode;
        opcode
    }

    /// Чистое чтение операнда (HL-indirect / fetch_byte / fetch_word
    /// look-up в режиме «без побочных эффектов»). Используется только
    /// там, где нам нужен байт без обновления латчей — например, в
    /// дизассемблере UI или при дампе памяти. Внутри исполнителя
    /// **не использовать**: каждый доступ исполнителя обязан пройти
    /// через `bus_read*` / `bus_write*` / `fetch_opcode`.
    pub fn peek(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    pub fn step_instruction<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        if self.active_tacts_remaining > 0 {
            self.cycle_count += u64::from(self.active_tacts_remaining);
            // Перед обнулением `tact_phase` запомним последнюю
            // выполненную T-фазу (`total - 1`), иначе UI после
            // flush'а потеряет позицию и нарисует `-` в «Такте»
            // блока «Машинный цикл». Если `active_tacts_total == 0`,
            // значит инструкция вообще не запускалась через
            // `step_tact` (был только `step_instruction`) — тогда
            // оставляем то, что выставит ветка ниже (`outcome.t_states - 1`).
            if self.active_tacts_total > 0 {
                self.last_completed_tact_phase = Some(self.active_tacts_total - 1);
            }
            self.active_tacts_remaining = 0;
            self.active_tacts_total = 0;
            self.tact_phase = None;
            return Ok(InstructionOutcome {
                opcode: None,
                mnemonic: "TACT-COMPLETE".to_owned(),
                pc_before: self.pc,
                pc_after: self.pc,
                t_states: 0,
                halted: self.halted,
                interrupt_accepted: false,
            });
        }

        let outcome = self.execute_instruction_boundary(bus)?;
        self.cycle_count += u64::from(outcome.t_states);
        // Атомарный путь: `step_instruction` без предварительного
        // walking через `step_tact`. T-states раздаются разом, и
        // последняя выполненная фаза равна `t_states - 1`. Обновляем
        // только когда инструкция реально проиграла такты — пустой
        // `t_states == 0` бывает на TACT-COMPLETE flush'е выше или
        // если бы кто-то вызвал boundary без работы (сейчас
        // невозможно, но защищаемся на будущее).
        if outcome.t_states > 0 {
            self.last_completed_tact_phase = Some(outcome.t_states - 1);
        }
        Ok(outcome)
    }

    pub fn step_tact<B: PortBus>(&mut self, bus: &mut B) -> Result<TactOutcome, CoreError> {
        if self.active_tacts_remaining == 0 {
            let t_states = if self.halted && !self.can_accept_interrupt() {
                1
            } else {
                self.execute_instruction_boundary(bus)?.t_states.max(1)
            };
            self.active_tacts_total = t_states;
            self.active_tacts_remaining = t_states;
            self.tact_phase = Some(0);
        }

        let phase = self.active_tacts_total - self.active_tacts_remaining;
        self.active_tacts_remaining -= 1;
        self.cycle_count += 1;
        let boundary = self.active_tacts_remaining == 0;
        self.tact_phase = if boundary { None } else { Some(phase + 1) };
        // Walking-режим: фиксируем КАЖДУЮ выполненную T-фазу. Это
        // даёт UI «застывшую» позицию между двумя нажатиями шага
        // такта (`F7`-эквивалент в школьном) — там фаза на индикаторе
        // показывает то, что было только что выполнено, а не то,
        // что будет следующим. На boundary `phase` равно `total - 1`,
        // что и нужно школьному эталону для замораживания на
        // последнем такте инструкции.
        self.last_completed_tact_phase = Some(phase);

        Ok(TactOutcome {
            tact_phase: phase,
            instruction_boundary: boundary,
            cycle_count: self.cycle_count,
        })
    }

    pub fn run_for_t_states<B: PortBus>(
        &mut self,
        bus: &mut B,
        t_states: u64,
    ) -> Result<(), CoreError> {
        for _ in 0..t_states {
            self.step_tact(bus)?;
        }
        Ok(())
    }

    pub fn run_until_halt<B: PortBus>(
        &mut self,
        bus: &mut B,
        max_instructions: u64,
    ) -> Result<u64, CoreError> {
        let mut executed = 0;
        while !self.halted && executed < max_instructions {
            self.step_instruction(bus)?;
            executed += 1;
        }
        Ok(executed)
    }
}
