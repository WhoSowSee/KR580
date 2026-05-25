//! Разбиение T-states инструкции на машинные циклы (M-циклы) по
//! таблице 8080 datasheet (Appendix A, табл. «Instruction Set Summary»).
//!
//! Зачем это нужно: наш исполнитель не моделирует M-циклы (см.
//! `docs/assumptions.md`) — мы атомарно выполняем инструкцию, потом
//! раздаём её T-states по тактам через `step_tact`. Но школьный
//! референсный эмулятор (по которому пользователь сверяется) и
//! табло на физической стойке КР-580 показывают **именно M-цикл и
//! T-фазу внутри M-цикла**, а не «общий T-states счётчик». Чтобы UI
//! мог показать ту же пару M/T, что и референс, нужна таблица:
//! «инструкция X состоит из машинных циклов длиной 4, 3, 3 такта».
//! Переводим линейный счётчик `tact_phase` (0..total) в `(m_cycle,
//! t_in_cycle)` через эту таблицу.
//!
//! Источник чисел: Intel 8080A/8085 Datasheet, табл. «Instruction
//! Set Summary», столбец «STATES». Длины M-циклов взяты из
//! «Machine Cycle» секции там же:
//!
//! - **M1 (opcode fetch)** — всегда **4** такта (5 для тех инструкций,
//!   у которых datasheet помечает M1 как 5T: INX/DCX/DAD/PCHL и т.п.,
//!   плюс некоторые branch-таken).
//! - **MR/MW (memory read/write)** — **3** такта.
//! - **IO read/write** — **3** такта (по сути MR/MW на портовом
//!   адресном пространстве).
//! - **STACK READ/WRITE** — **3** такта.
//! - **BUS IDLE** — присутствует у некоторых инструкций (DAD, INX,
//!   и т.п.) как «padding» внутри M1=5 или дополнительный M-цикл.
//!
//! Таблица ниже даёт **последовательность длин M-циклов** для
//! каждой документированной инструкции 8080. Сумма длин совпадает с
//! `InstructionTiming::t_states_taken`. Для условных инструкций
//! предусмотрены **две** последовательности (taken / not-taken):
//! например, `Cxxx a16` taken = 17T = 5+3+3+3+3 (M1=5, fetch_lo=3,
//! fetch_hi=3, push_hi=3, push_lo=3); not-taken = 11T = 5+3+3 (M1=5,
//! fetch_lo=3, fetch_hi=3, без push).
//!
//! Для несуществующих/нелегальных опкодов возвращаем `&[]` — UI
//! интерпретирует это как «нет данных, показываем 1/1».

use crate::decode::is_undocumented_opcode;

/// Длины M-циклов для одного варианта исполнения (taken либо
/// not-taken) одной инструкции. `&[4, 3, 3]` означает три M-цикла
/// длиной 4, 3, 3 такта соответственно (всего 10 тактов).
pub type MachineCycleLengths = &'static [u8];

/// Расклад M-циклов с учётом ветки (taken / not-taken).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineCycleLayout {
    pub taken: MachineCycleLengths,
    pub not_taken: Option<MachineCycleLengths>,
}

impl MachineCycleLayout {
    /// Layout с одной веткой (taken только, без branch). Публичный
    /// чтобы UI мог собрать собственный datasheet-layout для строки
    /// «T-фаза»: для HLT школьный layout = `[4]` (только видимый M1),
    /// но в UI хочется показывать честные 7 тактов, поэтому UI
    /// конструирует `MachineCycleLayout::fixed(&[7])`. Для остальных
    /// опкодов layout_for и так совпадает с datasheet, оверрайд не
    /// нужен.
    pub const fn fixed(cycles: MachineCycleLengths) -> Self {
        Self {
            taken: cycles,
            not_taken: None,
        }
    }

    const fn branch(taken: MachineCycleLengths, not_taken: MachineCycleLengths) -> Self {
        Self {
            taken,
            not_taken: Some(not_taken),
        }
    }

    /// Сумма тактов по выбранной ветке.
    pub fn total_t_states(self, branch_taken: bool) -> u8 {
        let cycles = if branch_taken {
            self.taken
        } else {
            self.not_taken.unwrap_or(self.taken)
        };
        let mut sum = 0u8;
        let mut i = 0;
        while i < cycles.len() {
            sum += cycles[i];
            i += 1;
        }
        sum
    }
}

/// Текущая позиция исполнения внутри инструкции в терминах
/// M-цикл / T-фаза. M-цикл нумеруется с 1 (как на табло КР-580 и в
/// школьном эмуляторе: «M1, M2, M3»), T-фаза — с 1 внутри своего
/// M-цикла (тоже как у референса).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineCyclePosition {
    pub m_cycle: u8,
    pub t_in_cycle: u8,
    pub m_cycle_length: u8,
}

/// Перевести линейную T-фазу (0..total) в позицию M/T. `linear_phase`
/// — наша внутренняя `tact_phase` (0 = T1 первого M-цикла, 1 = T2
/// первого M-цикла, и т.д.). Возвращает `None` если `linear_phase
/// >= total_t_states` или таблица пуста.
pub fn position_for(
    layout: MachineCycleLayout,
    branch_taken: bool,
    linear_phase: u8,
) -> Option<MachineCyclePosition> {
    let cycles = if branch_taken {
        layout.taken
    } else {
        layout.not_taken.unwrap_or(layout.taken)
    };
    if cycles.is_empty() {
        return None;
    }
    let mut consumed = 0u8;
    for (idx, &length) in cycles.iter().enumerate() {
        if linear_phase < consumed + length {
            return Some(MachineCyclePosition {
                m_cycle: (idx as u8) + 1,
                t_in_cycle: linear_phase - consumed + 1,
                m_cycle_length: length,
            });
        }
        consumed += length;
    }
    None
}

/// Расклад M-циклов для документированного опкода. Для нелегальных
/// опкодов возвращает layout с пустым массивом — UI интерпретирует
/// это как «нет данных».
pub fn layout_for(opcode: u8) -> MachineCycleLayout {
    if is_undocumented_opcode(opcode) {
        return MachineCycleLayout::fixed(&[]);
    }

    // MOV r1,r2 (0x40..=0x7F, кроме HLT=0x76): 5T если оба регистра
    // НЕ M=(HL), иначе 7T (один MR=3 для (HL)).
    if (0x40..=0x7F).contains(&opcode) && opcode != 0x76 {
        let dst = (opcode >> 3) & 7;
        let src = opcode & 7;
        return if dst == 6 || src == 6 {
            // M1=4 (fetch) + 3 (memory) = 7T.
            MachineCycleLayout::fixed(&[4, 3])
        } else {
            // M1=5 (fetch + internal). По datasheet для MOV r1,r2 без
            // (HL) общий T = 5, и это ровно один удлинённый M1.
            MachineCycleLayout::fixed(&[5])
        };
    }

    // ALU r (ADD/ADC/SUB/SBB/ANA/XRA/ORA/CMP, 0x80..=0xBF): 4T если
    // источник не (HL), 7T если (HL).
    if (0x80..=0xBF).contains(&opcode) {
        let src = opcode & 7;
        return if src == 6 {
            MachineCycleLayout::fixed(&[4, 3])
        } else {
            MachineCycleLayout::fixed(&[4])
        };
    }

    // INR/DCR r — 5T для регистра, 10T для M=(HL) (M1=4 + MR=3 + MW=3).
    if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
        let reg = (opcode >> 3) & 7;
        return if reg == 6 {
            MachineCycleLayout::fixed(&[4, 3, 3])
        } else {
            MachineCycleLayout::fixed(&[5])
        };
    }

    // MVI r,d8 — 7T (M1=4 + MR=3) для регистра, 10T (M1=4 + MR=3 +
    // MW=3) для (HL).
    if opcode & 0xC7 == 0x06 {
        let reg = (opcode >> 3) & 7;
        return if reg == 6 {
            MachineCycleLayout::fixed(&[4, 3, 3])
        } else {
            MachineCycleLayout::fixed(&[4, 3])
        };
    }

    // LXI rp,d16 — 10T (M1=4 + MR_lo=3 + MR_hi=3).
    if opcode & 0xCF == 0x01 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    // INX rp — 5T (один удлинённый M1).
    if opcode & 0xCF == 0x03 {
        return MachineCycleLayout::fixed(&[5]);
    }
    // DAD rp — 10T (M1=4 + два внутренних bus-idle цикла по 3T).
    if opcode & 0xCF == 0x09 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    // DCX rp — 5T (один удлинённый M1).
    if opcode & 0xCF == 0x0B {
        return MachineCycleLayout::fixed(&[5]);
    }

    // Rcond — taken=11T (M1=5 + pop_lo=3 + pop_hi=3), not-taken=5T
    // (просто M1=5, без обращений к стеку).
    if opcode & 0xC7 == 0xC0 {
        return MachineCycleLayout::branch(&[5, 3, 3], &[5]);
    }
    // Jcond — оба пути 10T (M1=4 + fetch_lo=3 + fetch_hi=3): на
    // 8080 операнд читается всегда, ветка только меняет PC.
    if opcode & 0xC7 == 0xC2 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    // Ccond — taken=17T (M1=5 + fetch_lo=3 + fetch_hi=3 + push_hi=3
    // + push_lo=3), not-taken=11T (M1=5 + fetch_lo=3 + fetch_hi=3).
    if opcode & 0xC7 == 0xC4 {
        return MachineCycleLayout::branch(&[5, 3, 3, 3, 3], &[5, 3, 3]);
    }
    // RST n — 11T (M1=5 + push_hi=3 + push_lo=3).
    if opcode & 0xC7 == 0xC7 {
        return MachineCycleLayout::fixed(&[5, 3, 3]);
    }
    // POP rp — 10T (M1=4 + pop_lo=3 + pop_hi=3).
    if opcode & 0xCF == 0xC1 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    // PUSH rp — 11T (M1=5 + push_hi=3 + push_lo=3).
    if opcode & 0xCF == 0xC5 {
        return MachineCycleLayout::fixed(&[5, 3, 3]);
    }

    match opcode {
        0x00 => MachineCycleLayout::fixed(&[4]),           // NOP
        0x02 | 0x12 => MachineCycleLayout::fixed(&[4, 3]), // STAX B/D = M1=4 + MW=3
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => {
            // RLC/RRC/RAL/RAR/DAA/CMA/STC/CMC — 4T, один M1.
            MachineCycleLayout::fixed(&[4])
        }
        0x0A | 0x1A => MachineCycleLayout::fixed(&[4, 3]), // LDAX B/D = M1=4 + MR=3
        0x22 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]), // SHLD = 16T
        0x2A => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]), // LHLD = 16T
        0x32 => MachineCycleLayout::fixed(&[4, 3, 3, 3]),  // STA = 13T
        0x3A => MachineCycleLayout::fixed(&[4, 3, 3, 3]),  // LDA = 13T
        0x76 => MachineCycleLayout::fixed(&[4]), // HLT: school table показывает только M1=4 (fetch). Реальные 7T даташита (M1=4 fetch + M2=3 halt-ack) идут в `cycle_count` через `decode.rs`, но на табло КР-580 школьный эталон M2 не отрисовывает — фиксируется на T4 первого M1. Поэтому layout-сумма (4) расходится с `t_states_taken` (7); тест `layout_sums_match_decode_timing_for_all_documented_opcodes` для HLT делает исключение.
        0xC3 => MachineCycleLayout::fixed(&[4, 3, 3]), // JMP = 10T
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
            // ADI/ACI/SUI/SBI/ANI/XRI/ORI/CPI = 7T (M1=4 + MR=3).
            MachineCycleLayout::fixed(&[4, 3])
        }
        0xC9 => MachineCycleLayout::fixed(&[4, 3, 3]), // RET = 10T
        0xCD => MachineCycleLayout::fixed(&[5, 3, 3, 3, 3]), // CALL = 17T
        0xD3 => MachineCycleLayout::fixed(&[4, 3, 3]), // OUT = 10T (M1=4 + MR=3 + IOW=3)
        0xDB => MachineCycleLayout::fixed(&[4, 3, 3]), // IN = 10T (M1=4 + MR=3 + IOR=3)
        0xE3 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3, 2]), // XTHL = 18T (последний M — 2T idle)
        0xE9 => MachineCycleLayout::fixed(&[5]),                // PCHL = 5T
        0xEB => MachineCycleLayout::fixed(&[5]),                // XCHG = 4T в datasheet, у нас 5T
        0xF3 | 0xFB => MachineCycleLayout::fixed(&[4]),         // DI / EI = 4T
        0xF9 => MachineCycleLayout::fixed(&[5]),                // SPHL = 5T
        _ => MachineCycleLayout::fixed(&[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Расклад должен суммироваться ровно в `t_states_taken` нашего
    /// `decode_opcode().timing`, иначе UI покажет M-цикл/T-фазу, не
    /// совпадающие с total. Этот тест ловит регрессии в обеих
    /// таблицах: и в нашей `decode.rs`, и здесь.
    ///
    /// Исключение — HLT (0x76): datasheet даёт 7T (M1 fetch=4 + M2
    /// halt-ack=3), но школьный эмулятор КР-580 на табло M2 не
    /// отрисовывает — фиксируется на T4 первого M1, потому что M2
    /// HLT — это «бесконечное ожидание прерывания», не реальный bus
    /// cycle с обращением к памяти. Чтобы UI совпал со школьным,
    /// layout HLT — `[4]` (только видимый M1), а `cycle_count`
    /// продолжает инкрементироваться на честные 7T из `decode.rs`.
    /// Расхождение layout-суммы (4) и `t_states_taken` (7) — намеренное.
    #[test]
    fn layout_sums_match_decode_timing_for_all_documented_opcodes() {
        for opcode in 0u8..=255 {
            if is_undocumented_opcode(opcode) {
                continue;
            }
            let info = crate::decode::decode_opcode(opcode).unwrap();
            let layout = layout_for(opcode);
            assert!(
                !layout.taken.is_empty(),
                "documented opcode {opcode:#04X} must have a layout"
            );
            if opcode == 0x76 {
                // HLT — намеренное расхождение, см. doc выше.
                assert_eq!(layout.total_t_states(true), 4);
                assert_eq!(info.timing.t_states_taken, 7);
                continue;
            }
            assert_eq!(
                layout.total_t_states(true),
                info.timing.t_states_taken,
                "taken-sum mismatch for {opcode:#04X}"
            );
            if let Some(not_taken) = info.timing.t_states_not_taken {
                assert_eq!(
                    layout.total_t_states(false),
                    not_taken,
                    "not-taken-sum mismatch for {opcode:#04X}"
                );
            }
        }
    }

    /// `MOV A,B` — один M-цикл из 5 тактов, фазы 1..5.
    #[test]
    fn mov_register_register_maps_to_single_m_cycle() {
        let layout = layout_for(0x78); // MOV A,B
        for t in 0..5 {
            let pos = position_for(layout, true, t).unwrap();
            assert_eq!(pos.m_cycle, 1);
            assert_eq!(pos.t_in_cycle, t + 1);
            assert_eq!(pos.m_cycle_length, 5);
        }
        assert!(position_for(layout, true, 5).is_none());
    }

    /// `LXI B,d16` (10T = 4+3+3): T0-T3 → M1, T4-T6 → M2, T7-T9 → M3.
    #[test]
    fn lxi_three_machine_cycles() {
        let layout = layout_for(0x01); // LXI B,d16
        assert_eq!(position_for(layout, true, 0).unwrap().m_cycle, 1);
        assert_eq!(position_for(layout, true, 3).unwrap().m_cycle, 1);
        assert_eq!(position_for(layout, true, 4).unwrap().m_cycle, 2);
        assert_eq!(position_for(layout, true, 6).unwrap().m_cycle, 2);
        assert_eq!(position_for(layout, true, 7).unwrap().m_cycle, 3);
        assert_eq!(position_for(layout, true, 9).unwrap().m_cycle, 3);
        let last = position_for(layout, true, 9).unwrap();
        assert_eq!(last.t_in_cycle, 3);
        assert_eq!(last.m_cycle_length, 3);
    }

    /// `RZ` (Rcond): taken=11T=5+3+3, not-taken=5T=5. Two layouts.
    #[test]
    fn rcond_branch_layouts_differ() {
        let layout = layout_for(0xC8); // RZ
        assert_eq!(layout.total_t_states(true), 11);
        assert_eq!(layout.total_t_states(false), 5);
        // Not-taken: только M1=5T.
        assert_eq!(position_for(layout, false, 4).unwrap().m_cycle, 1);
        assert!(position_for(layout, false, 5).is_none());
        // Taken: три M-цикла.
        assert_eq!(position_for(layout, true, 5).unwrap().m_cycle, 2);
        assert_eq!(position_for(layout, true, 8).unwrap().m_cycle, 3);
    }
}
