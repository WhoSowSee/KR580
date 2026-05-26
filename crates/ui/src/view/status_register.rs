//! «Регистр состояния» — статусный байт T1 + русская расшифровка.
//!
//! Школьный референсный эмулятор КР-580 (по которому пользователь
//! сверяется) рисует в верхнем левом углу плиты блок «Регистр состояния»:
//! 8 бит того статусного байта, который чип защёлкивает в T1 каждого
//! M-цикла и выкатывает на D7-D0 вместе с импульсом SYNC, плюс
//! текстовая расшифровка («Чтение памяти», «Запись в порт», и т.д.).
//! Раскладка битов — Intel 8080A datasheet, рис. «Status Information»:
//!
//! ```text
//! D7  D6  D5    D4    D3   D2  D1   D0
//! MEM INP M1   OUT   HLTA STK WO   INTA
//! R           Read         Bar
//! ```
//!
//! Мы не моделируем M-циклы внутри исполнителя (см.
//! `docs/assumptions.md` — инструкция атомарна, T-states раздаются
//! счётчиком), но `last_completed_tact_phase` + таблица из
//! `core::machine_cycle` дают достаточно чтобы UI выдавал тот же
//! статусный байт на той же T-фазе, что и физическая стойка КР-580.
//! Преобразование: линейная T-фаза → `position_for(layout, taken,
//! phase)` → индекс M-цикла → `kind_at(opcode, idx, taken)` → тип
//! M-цикла → `status_byte()` + `label_ru()`.
//!
//! Два рантайм-оверрайда поверх таблицы:
//!
//! - `cpu.halted` ⇒ `HaltAck`. После `HLT`-fetch чип переходит в
//!   состояние с MEMR=1, HLTA=1, WO=1 на шине статуса и виснет до
//!   прерывания. В таблице `kinds_for` HaltAck не лежит специально —
//!   физическая M-цикл-таблица для опкода 0x76 = `[4]` (только
//!   M1Fetch, 4 такта), а HLTA-цикл — это уже **следующий** M-цикл
//!   который чип никогда не закончит, потому что `READY` снят. UI
//!   проще выставить статус по флагу `cpu.halted`, чем расширять
//!   layout HLT'а на «второй M-цикл с бесконечной длиной».
//! - `cpu.interrupt_request_pending && cpu.interrupt_enable` ⇒
//!   `InterruptAck`. На физическом 8080 INTE-pin = 1 + INT-pin = 1
//!   запускает специальный M1, в котором чип защёлкивает INTA=1,
//!   M1=1, WO=1 и читает RST n / CALL прямо с шины устройства,
//!   минуя PC. У нас нет PIC-чипа, но семантика та же: пока
//!   `interrupt_request_pending` стоит и IF разрешён, чип
//!   собирается принять прерывание на следующем M1.

use iced::widget::{Space, column, container, row, tooltip};
use iced::{Element, Length, Padding, alignment};
use k580_core::{
    Cpu8080State, MachineCycleKind, MachineCycleLayout, kind_at, layout_for, position_for,
};
use std::time::Duration;

use super::styles::status_tooltip_style;
use super::theme::{TOKYO_BLUE, TOKYO_TEXT, ui_text};
use crate::app::Message;

/// Угадываем «занят ли branch» так же, как делает `view::cycles`:
/// если `last_completed_tact_phase` указывает на сумму tact'ов одной
/// из веток — берём её. Для условных инструкций (Cxxx, Rxxx, Jxxx)
/// taken и not-taken различаются по числу T-states (17 vs 11, 11 vs
/// 5, etc.), поэтому однозначно восстанавливается. Для безусловных
/// инструкций ветки совпадают — taken=true возвращает тот же layout.
fn branch_taken_from_phase(layout: MachineCycleLayout, phase: u8) -> bool {
    // Сначала пробуем not-taken: если фаза в его пределах — это
    // not-taken. Иначе taken (он всегда длиннее или равен).
    if let Some(not_taken) = layout.not_taken {
        let not_taken_total: u8 = not_taken.iter().sum();
        if phase < not_taken_total {
            return false;
        }
    }
    true
}

/// Определяет тип машинного цикла, в котором сейчас (по последней
/// выполненной T-фазе) находится CPU. Возвращаемое значение — то,
/// что школьный эталон рисует в блоке «Регистр состояния»: статусный
/// байт + текстовая расшифровка.
pub(super) fn derive_status_kind(cpu: &Cpu8080State) -> MachineCycleKind {
    // Рантайм-оверрайды — раньше всех табличных проверок.
    //
    // INTA проверяем до HLT: если CPU висит в HLT, но устройство
    // подняло INT и IF разрешён, чип на следующем такте поднимет
    // INTA-цикл и сбросит HLT — статус-байт уже должен это
    // отражать, потому что школьный эталон именно так и рисует
    // («подтверждение прерывания», а не «подтверждение останова»,
    // как только INT поднялся).
    if cpu.interrupt_request_pending && cpu.interrupt_enable {
        return MachineCycleKind::InterruptAck;
    }
    if cpu.halted {
        return MachineCycleKind::HaltAck;
    }

    // Холодный старт: ни одной инструкции не выполнено,
    // `last_completed_tact_phase == None`. На физическом чипе после
    // RESET первая T1 — это уже M1 первой инструкции (PC=0000), и
    // статусный байт = M1Fetch. Это совпадает со школьным эталоном:
    // блок «Регистр состояния» при холодном старте показывает «Загрузка
    // опкода» / `1010 0010`, а не пустоту.
    let Some(phase) = cpu.last_completed_tact_phase else {
        return MachineCycleKind::M1Fetch;
    };

    let layout = layout_for(cpu.last_fetched_opcode);
    let taken = branch_taken_from_phase(layout, phase);
    let Some(position) = position_for(layout, taken, phase) else {
        // Layout пуст (нелегальный опкод) или фаза вылезла за
        // пределы — возвращаем M1Fetch как нейтральный дефолт.
        // На реальном чипе нелегальный опкод всё равно проходит
        // через M1, просто потом исполнитель ловит его как NOP.
        return MachineCycleKind::M1Fetch;
    };

    let m_cycle_idx = (position.m_cycle - 1) as usize;
    kind_at(cpu.last_fetched_opcode, m_cycle_idx, taken).unwrap_or(MachineCycleKind::M1Fetch)
}

fn status_bits(byte: u8) -> String {
    format!(
        "{}{}{}{} {}{}{}{}",
        (byte >> 7) & 1,
        (byte >> 6) & 1,
        (byte >> 5) & 1,
        (byte >> 4) & 1,
        (byte >> 3) & 1,
        (byte >> 2) & 1,
        (byte >> 1) & 1,
        byte & 1,
    )
}

pub(super) fn status_register_bits(cpu: &Cpu8080State) -> String {
    let kind = derive_status_kind(cpu);
    let byte = kind.status_byte();
    status_bits(byte)
}

fn status_register_tooltip_body_lines(cpu: &Cpu8080State) -> [String; 2] {
    let label = derive_status_kind(cpu).label_ru();
    [
        "Статусный байт T1: что процессор делает на текущем такте.\n\
         Биты слева направо: чтение памяти, ввод, загрузка опкода, \
         вывод, останов, стек, запись, подтверждение прерывания."
            .to_owned(),
        format!("Статус: {label}"),
    ]
}

pub(super) fn status_register_tooltip<'a>(
    cpu: &Cpu8080State,
    face: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let [description, status_line] = status_register_tooltip_body_lines(cpu);
    let status_label = status_line
        .strip_prefix("Статус: ")
        .unwrap_or(&status_line)
        .to_owned();

    let body = container(
        column![
            ui_text(description, 12, TOKYO_TEXT),
            Space::new().height(Length::Fixed(6.0)),
            row![
                ui_text("Статус: ", 12, TOKYO_TEXT),
                ui_text(status_label, 12, TOKYO_BLUE),
            ]
            .spacing(0)
            .align_y(alignment::Vertical::Center),
        ]
        .width(Length::Fill),
    )
    .padding(Padding {
        top: 4.0,
        right: 8.0,
        bottom: 4.0,
        left: 8.0,
    })
    .max_width(280.0)
    .style(status_tooltip_style);

    tooltip(face, body, tooltip::Position::Bottom)
        .gap(4.0)
        .padding(12.0)
        .delay(Duration::from_millis(600))
        .snap_within_viewport(true)
        .into()
}

#[cfg(test)]
mod tests {
    //! Тесты привязаны к `derive_status_kind` — единственная чисто
    //! логическая часть модуля (рендер-функции возвращают iced
    //! `Element` и не тестируются юнитами). Каждый тест строит
    //! `Cpu8080State` руками, а не через `step_instruction`, чтобы
    //! зафиксировать ровно то состояние, которое нужно проверить:
    //! `last_completed_tact_phase`, `last_fetched_opcode`,
    //! `halted`, `interrupt_request_pending`, `interrupt_enable`.
    //! Это и есть «контракт» между core и блоком «Регистр состояния».

    use super::*;

    fn cpu_with(opcode: u8, phase: Option<u8>) -> Cpu8080State {
        let mut cpu = Cpu8080State::default();
        cpu.last_fetched_opcode = opcode;
        cpu.last_completed_tact_phase = phase;
        cpu
    }

    #[test]
    fn cold_start_is_m1_fetch() {
        // Холодный старт: ни одной инструкции не выполнено,
        // last_completed_tact_phase == None. Школьный эталон при
        // RESET показывает «Загрузка опкода» / 1010 0010 — первая
        // T1 первой инструкции по PC=0000.
        let cpu = Cpu8080State::default();
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::M1Fetch);
    }

    #[test]
    fn status_register_bits_uses_t1_status_byte() {
        let cpu = Cpu8080State::default();
        assert_eq!(status_register_bits(&cpu), "1010 0010");
    }

    #[test]
    fn tooltip_body_does_not_repeat_status_register_bits() {
        let cpu = Cpu8080State::default();
        let lines = status_register_tooltip_body_lines(&cpu);
        assert!(
            !lines
                .iter()
                .any(|line| line.contains("Регистр состояния 1010 0010"))
        );
        assert!(lines.iter().any(|line| line == "Статус: Загрузка опкода"));
    }

    #[test]
    fn halt_overrides_table() {
        // HLT (опкод 0x76) после исполнения: cpu.halted=true. Школьный
        // эталон в этом состоянии рисует «Подтв. останова» / 1000
        // 1010, а не M1Fetch последней инструкции.
        let mut cpu = cpu_with(0x76, Some(3));
        cpu.halted = true;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::HaltAck);
    }

    #[test]
    fn interrupt_overrides_halt() {
        // INTE+INT поверх HLT: чип на следующем такте поднимет INTA
        // и сбросит HLT. Статус-байт уже должен показывать
        // «Подтв. прерывания», а не «Подтв. останова» — школьный
        // эталон рисует так же.
        let mut cpu = cpu_with(0x76, Some(3));
        cpu.halted = true;
        cpu.interrupt_request_pending = true;
        cpu.interrupt_enable = true;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::InterruptAck);
    }

    #[test]
    fn interrupt_pending_without_inte_uses_table() {
        // INT поднят, но IF=0 — чип игнорирует прерывание, продолжает
        // обычный цикл. Статус-байт берётся из таблицы M-циклов,
        // не из рантайм-оверрайда.
        let mut cpu = cpu_with(0x00, Some(0)); // NOP, T1 первого M1
        cpu.interrupt_request_pending = true;
        cpu.interrupt_enable = false;
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::M1Fetch);
    }

    #[test]
    fn first_phase_of_any_opcode_is_m1_fetch() {
        // T1 любого M1 — это всегда M1Fetch (datasheet 8080A,
        // обязательный invariant). Берём набор покрывающих опкодов:
        // NOP, MOV, MVI, LXI, STA, LDA, PUSH, POP, CALL, RET, IN, OUT.
        let opcodes = [
            0x00, 0x40, 0x06, 0x01, 0x32, 0x3A, 0xC5, 0xC1, 0xCD, 0xC9, 0xDB, 0xD3,
        ];
        for &op in &opcodes {
            let cpu = cpu_with(op, Some(0));
            assert_eq!(
                derive_status_kind(&cpu),
                MachineCycleKind::M1Fetch,
                "opcode {:#04X} phase 0 must be M1Fetch",
                op,
            );
        }
    }

    #[test]
    fn sta_second_m_cycle_is_memory_read_third_is_memory_write() {
        // STA addr (опкод 0x32) = 13T = M1(4) + MR(3) + MR(3) + MW(3).
        // M1 — fetch опкода (T1..T4 = phase 0..3). M2 — fetch lo
        // байта адреса (phase 4..6). M3 — fetch hi байта адреса
        // (phase 7..9). M4 — запись A в память (phase 10..12).
        // Школьный эталон на phase 4 рисует «Чтение памяти», на
        // phase 10 — «Запись в память».
        let cpu = cpu_with(0x32, Some(4)); // T1 второго M-цикла (MR)
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryRead);

        let cpu = cpu_with(0x32, Some(7)); // T1 третьего M-цикла (MR)
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryRead);

        let cpu = cpu_with(0x32, Some(10)); // T1 четвёртого M-цикла (MW)
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::MemoryWrite);
    }

    #[test]
    fn out_second_m_cycle_is_io_write() {
        // OUT port (опкод 0xD3) = 10T = M1(4) + MR(3) + IoWrite(3).
        // На phase 7 (T1 третьего M-цикла) школьный эталон рисует
        // «Запись в порт» / 0001 0000.
        let cpu = cpu_with(0xD3, Some(7));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::IoWrite);
    }

    #[test]
    fn in_second_m_cycle_is_io_read() {
        // IN port (опкод 0xDB) = 10T = M1(4) + MR(3) + IoRead(3).
        // На phase 7 школьный эталон рисует «Чтение из порта» /
        // 0100 0010.
        let cpu = cpu_with(0xDB, Some(7));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::IoRead);
    }

    #[test]
    fn push_writes_to_stack() {
        // PUSH B (опкод 0xC5) = 11T = M1(5) + StackWrite(3) + StackWrite(3).
        // На phase 5 (T1 второго M-цикла) школьный эталон рисует
        // «Запись в стек» / 0000 0100.
        let cpu = cpu_with(0xC5, Some(5));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::StackWrite);
    }

    #[test]
    fn pop_reads_from_stack() {
        // POP B (опкод 0xC1) = 10T = M1(4) + StackRead(3) + StackRead(3).
        // На phase 4 школьный эталон рисует «Чтение из стека» /
        // 1000 0110.
        let cpu = cpu_with(0xC1, Some(4));
        assert_eq!(derive_status_kind(&cpu), MachineCycleKind::StackRead);
    }
}
