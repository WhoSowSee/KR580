//! Регрессионные тесты на «последние шинные» латчи: РК
//! (`last_fetched_opcode`), Буфер данных (`last_data_bus_byte`) и
//! Буфер адреса (`last_address_bus`). Это четыре блока схематика,
//! которые до этой задачи показывали look-ahead в RAM по PC и
//! расходились со школьным эталоном после `HLT`, после операций
//! записи и после fetch операнда. Здесь фиксируем семантику:
//!
//! - после fetch'а опкода РК хранит **тот** байт, не RAM[PC];
//! - после записи в память буфер данных хранит записанный байт,
//!   а адресный — адрес записи (не PC);
//! - после `HLT` PC шагнул, но РК продолжает держать `0x76`
//!   до следующего M1 (которого не будет, пока не сбросим halt).

use k580_core::{Cpu8080State, NullBus};

fn step(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
}

fn put_program(cpu: &mut Cpu8080State, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
}

/// После выполнения `MVI A, 0x42` РК должен хранить `0x3E` (опкод
/// MVI A), а буфер данных — `0x42` (последний байт через шину, это
/// immediate-операнд, прочитанный после опкода). Если бы мы по-
/// прежнему читали РК как `memory.read(pc)`, после инструкции PC
/// шагнул бы на следующий байт и UI показал бы байт по новому PC,
/// а не реально загруженный опкод.
#[test]
fn mvi_records_opcode_in_ir_and_immediate_in_data_buffer() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x3E, 0x42]); // MVI A, 0x42
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(
        cpu.last_fetched_opcode, 0x3E,
        "РК хранит опкод MVI A, не байт по новому PC"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x42,
        "Буфер данных хранит immediate-операнд (последний байт через шину)"
    );
    // Адрес последней операции — адрес immediate-байта (PC=1), не
    // новый PC=2. Раньше мы показывали бы новый PC.
    assert_eq!(
        cpu.last_address_bus, 0x0001,
        "Буфер адреса хранит адрес immediate-операнда"
    );
}

/// `STA 0x4000` записывает A в RAM[0x4000]. После операции буфер
/// данных должен хранить записанный байт (значение A), а буфер
/// адреса — `0x4000`, не новый PC. Это самый явный случай, где
/// look-ahead по PC даст совершенно «не тот» байт.
#[test]
fn sta_records_written_byte_and_target_address() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x77;
    put_program(&mut cpu, &[0x32, 0x00, 0x40]); // STA 0x4000
    step(&mut cpu);
    assert_eq!(cpu.memory.read(0x4000), 0x77);
    assert_eq!(
        cpu.last_data_bus_byte, 0x77,
        "Буфер данных хранит записанный байт"
    );
    assert_eq!(
        cpu.last_address_bus, 0x4000,
        "Буфер адреса хранит адрес назначения, не PC"
    );
    assert_eq!(
        cpu.last_fetched_opcode, 0x32,
        "РК хранит опкод STA до следующего M1"
    );
}

/// После `HLT` (опкод `0x76`) PC шагает на следующий байт, но
/// **следующий M1 не наступит** до запроса прерывания или сброса —
/// значит РК должен продолжать держать `0x76`. Раньше readout
/// «Регистр команд» использовал `memory.read(pc)`, и после HLT
/// показывал `0x00` (NOP в очищенной RAM по новому PC). Это и
/// был один из 11 расхождений со школьным эталоном.
#[test]
fn hlt_freezes_ir_at_seventy_six() {
    let mut cpu = Cpu8080State::default();
    // Программа: 8 NOP, затем HLT по адресу 0x0008.
    put_program(
        &mut cpu,
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76],
    );
    // Прокручиваем 8 NOP.
    for _ in 0..8 {
        step(&mut cpu);
    }
    assert_eq!(cpu.pc, 0x0008);
    // Выполняем HLT.
    step(&mut cpu);
    assert!(cpu.halted, "HLT поднял halt-флаг");
    assert_eq!(cpu.pc, 0x0009, "PC шагнул за HLT (datasheet behaviour)");
    assert_eq!(
        cpu.last_fetched_opcode, 0x76,
        "РК продолжает держать 0x76 после HLT (M1 не наступит)"
    );
    // Адрес последней шинной операции — адрес самого HLT'а (0x0008),
    // не новый PC=0x0009. Школьный эмулятор показывает именно 0x0008
    // в «Буфер адреса» после HLT.
    assert_eq!(
        cpu.last_address_bus, 0x0008,
        "Буфер адреса = адрес HLT, не новый PC"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x76,
        "Буфер данных = опкод HLT (последний байт через шину)"
    );
}

/// `MOV A, B` — чисто внутреннее перемещение между регистрами.
/// Шину не трогает (кроме fetch'а самого опкода), значит латчи
/// после операции хранят M1-fetch: РК = опкод, адрес = PC опкода,
/// буфер данных = опкод (это последний байт, прошедший по D7-D0).
/// Этот тест ловит регрессию, где `read_reg_code` для регистровых
/// кодов (не M=110) случайно начнёт обновлять шинные латчи.
#[test]
fn mov_register_to_register_does_not_touch_bus_beyond_m1() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.b = 0xAB;
    put_program(&mut cpu, &[0x78]); // MOV A, B
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0xAB);
    assert_eq!(cpu.last_fetched_opcode, 0x78);
    // Адрес последней шинной операции — это M1-fetch на PC=0.
    assert_eq!(cpu.last_address_bus, 0x0000);
    assert_eq!(cpu.last_data_bus_byte, 0x78);
}

/// `MOV A, (HL)` — чтение через индирект. Тут шина дёргается:
/// HL выставляется на адресный буфер, прочитанный байт идёт через
/// буфер данных. После операции латчи должны показывать HL и
/// прочитанный байт, а не M1-fetch.
#[test]
fn mov_a_from_hl_indirect_records_hl_and_byte() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.set_hl(0x2000);
    cpu.memory.write(0x2000, 0x5A);
    put_program(&mut cpu, &[0x7E]); // MOV A, (HL)
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x5A);
    assert_eq!(cpu.last_fetched_opcode, 0x7E, "РК = опкод MOV A,(HL)");
    assert_eq!(
        cpu.last_address_bus, 0x2000,
        "Буфер адреса хранит HL, не PC опкода"
    );
    assert_eq!(
        cpu.last_data_bus_byte, 0x5A,
        "Буфер данных хранит прочитанный из (HL) байт"
    );
}

/// `Reset` обязан обнулить шинные латчи — иначе после загрузки
/// программы UI покажет «остатки» с предыдущей сессии в РК и
/// буферах. `Cpu8080State::default()` тоже даёт нули, так что
/// проверяем оба пути.
#[test]
fn reset_clears_bus_latches() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x3E, 0x42]); // MVI A
    step(&mut cpu);
    assert_ne!(cpu.last_fetched_opcode, 0);
    cpu.reset_cpu();
    assert_eq!(cpu.last_fetched_opcode, 0);
    assert_eq!(cpu.last_data_bus_byte, 0);
    assert_eq!(cpu.last_address_bus, 0);
}
