//! Два блока на нижней управляющей строке схематика — рядом с
//! линиями шинных сигналов (F1/F2/SYNC/READY/...). Показывают, где
//! сейчас находится исполнение текущей инструкции, в двух разных
//! углах зрения: «по машинным циклам» и «по сквозным тактам».
//!
//! # Блок 1: «Цикл и такт» — по машинным циклам
//!
//! Каждая инструкция КР-580 разбивается на машинные циклы (M-циклы),
//! каждый M-цикл — на такты (T-фазы). Простая инструкция (NOP, ADD r)
//! состоит из одного M-цикла на 4-5 тактов; сложная (CALL) — из
//! нескольких M-циклов общим объёмом до 17 тактов. Этот блок
//! показывает где мы сейчас в этой иерархии:
//!
//! - **Цикл** — номер текущего M-цикла внутри инструкции, с 1.
//!   Берётся через `core::machine_cycle::layout_for`. Для NOP / MOV r,r /
//!   ADD r / HLT — всегда 1. Для LXI / JMP / MVI A,M — 2 или 3. Для
//!   CALL — до 5.
//! - **Такт** — номер T-фазы внутри текущего M-цикла, с 1 (T1, T2,
//!   T3, T4, ...). Берётся из `last_completed_tact_phase` через тот
//!   же layout с клампингом — после остановки на индикаторе остаётся
//!   ровно последняя выполненная T-фаза. Для HLT клампится до T4
//!   (halt-acknowledge цикл M2 в этом блоке не показывается, чтобы
//!   индикатор не «перепрыгивал» после уже остановленной инструкции).
//!
//! # Блок 2: «Внутренний тайминг» — сквозные такты
//!
//! Точные значения из технического описания по полной длительности
//! инструкции, без разбиения на M-циклы. Три строки:
//!
//! - **Тактов** — `cpu.cycle_count`. Сквозной счётчик всех T-states
//!   от начала программы (или с последнего сброса). Растёт на 4 за
//!   NOP, на 7 за HLT (полная длительность по техническому описанию,
//!   включая halt-acknowledge), на 17 за CALL и т.д.
//! - **Такт инструкции** — номер такта внутри текущей инструкции по
//!   полной длительности из технического описания Intel, с 1. Для
//!   HLT даёт 7 (включая halt-acknowledge цикл, который Блок 1
//!   скрывает). Для остальных опкодов совпадает с «Тактом» из Блока
//!   1, потому что длительность из технического описания совпадает с
//!   layout-суммой. Берётся через служебный layout
//!   `full_duration_layout` который для HLT возвращает `[7]`, для
//!   остального — `layout_for`.
//! - **Фаза** — `cpu.last_completed_tact_phase` (или
//!   `cpu.tact_phase` если идёт исполнение). Это **индекс с 0**:
//!   0..total-1, где total = полная длительность инструкции по
//!   техническому описанию. Для NOP это 0..3, для HLT — 0..6, для
//!   CALL — 0..16. Звёздочка `*` после числа означает «инструкция
//!   уже завершена, активного исполнения нет, показано последнее
//!   зафиксированное значение». Это поле сохраняется в `.580`
//!   snapshot, поэтому формат — индекс с 0 (как у массивов).
//!
//! # Зачем два блока
//!
//! «Цикл и такт» удобен для пошаговой отладки на уровне M-циклов:
//! видно какой именно шаг сложной инструкции (fetch, чтение операнда,
//! push на стек) сейчас выполняется. «Внутренний тайминг» нужен для
//! сверки с техническим описанием Intel и измерения времени программ
//! — там важна полная длительность, а не разбиение на M-циклы. Они
//! расходятся на HLT (4 vs 7 такта) и на сложных инструкциях (где
//! «Такт» в Блоке 1 сбрасывается на каждом новом M-цикле, а «Фаза» в
//! Блоке 2 растёт сквозно). Для простых однотактовых инструкций
//! (NOP, MOV r,r, ADD r) показывают почти то же самое — отличаются
//! на ±1 из-за разной нумерации (с 1 vs с 0).
//!
//! # Tooltip-подсказки
//!
//! Подписи строк короткие («Цикл», «Такт», «Тактов», «Такт
//! инструкции», «Фаза»), полное объяснение — в hover-tooltip над
//! каждой строкой. Tooltip переиспользует ту же chrome
//! (`inset_style`) что и action-panel chips и edit-button, чтобы
//! визуально читался как часть одной семьи UI-элементов.

use iced::widget::{Space, column, container, row, tooltip};
use iced::{Element, Length, Padding, alignment};
use k580_core::{
    Cpu8080State, MachineCycleLayout, MachineCyclePosition, decode_opcode, layout_for, position_for,
};

use super::styles::{inset_style, schematic_block_style};
use super::theme::{TOKYO_GREEN, TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};
use crate::app::Message;

/// Ширина левого блока «Цикл и такт». Узкий — там всего две короткие
/// строки с однозначным числом справа. Подобрано так чтобы заголовок
/// «Цикл и такт» помещался в одну строку и не переносился.
const CYCLE_BLOCK_WIDTH: f32 = 140.0;

/// Ширина правого блока «Внутренний тайминг». Шире левого — у него
/// три строки и одна из них («Такт инструкции») ощутимо длиннее, плюс
/// сами числа могут быть многозначные (`cycle_count` после долгой
/// программы — 5-7 цифр). Пользователь явно попросил «можешь сделать
/// правый блок чуть шире» — это та самая разница.
const TIMING_BLOCK_WIDTH: f32 = 200.0;

/// Layout с **полной длительностью** инструкции по техническому
/// описанию, без разбиения на M-циклы. Для HLT возвращает `[7]` (M1
/// fetch=4 + M2 halt-ack=3, единым блоком — нам нужна только сумма
/// для подсчёта позиции в строке «Такт инструкции»). Для остальных
/// опкодов возвращает тот же `layout_for(opcode)` — там layout-сумма
/// уже совпадает с `t_states_taken`. Это служебный layout **только
/// для UI-строки `Такт инструкции`**, чтобы дать «честный» номер T
/// в инструкции по техническому описанию (HLT → 1..7), не ломая
/// layout по M-циклам, который рулит блоком «Цикл и такт».
fn full_duration_layout(opcode: u8) -> MachineCycleLayout {
    if opcode == 0x76 {
        // HLT: 7T одним блоком. M1=4 + M2=3, но в строке полной
        // длительности нам нужно показать линейную позицию 1..7,
        // поэтому склеиваем их в один M-цикл. Корректная разбивка
        // M1/M2 для блока «Цикл и такт» живёт в `layout_for` (там
        // HLT = `[4]`, M2 halt-ack не показывается). Это разделение
        // — суть всей истории «Такт=4 в Блоке 1 vs Такт
        // инструкции=7 в Блоке 2».
        return MachineCycleLayout::fixed(&[7]);
    }
    // Все остальные документированные опкоды: layout-сумма совпадает
    // с `t_states_taken`, поэтому layout по M-циклам годится и для
    // семантики полной длительности.
    layout_for(opcode)
}

/// Расчёт текущей позиции M-цикл / T-фаза по T-states-фазе из core.
/// `phase_source` — линейная фаза, по которой считать позицию:
///
/// - `cpu.tact_phase` для «активного» режима (показ позиции внутри
///   текущей инструкции, пока она ещё идёт через `step_tact`);
/// - `cpu.last_completed_tact_phase` для «замороженного» режима
///   (показ последней выполненной T-фазы после завершения).
///
/// `use_full_duration` управляет какой layout использовать:
///
/// - `false` → `layout_for(opcode)` (layout по M-циклам, для HLT даёт 4T в M1)
/// - `true` → `full_duration_layout(opcode)` (полный, для HLT даёт 7T)
///
/// Логика декодирования одна и та же: берём байт в РК
/// (`last_fetched_opcode`), декодируем для проверки легальности,
/// получаем layout и переводим линейную фазу в (M, T). Если фаза
/// `None` — возвращаем `None`, UI нарисует `-`.
///
/// Особый случай — HLT при `use_full_duration=false`: его layout по
/// M-циклам = `[4]` (только видимый M1), а `cycle_count` инкрементируется
/// на честные 7T. После завершения HLT `last_completed_tact_phase = 6`,
/// но в layout всего 4 такта — `position_for` для phase=6
/// вернёт `None`. Чтобы UI не падал в `-`, клампим фазу к
/// `total_t_states(layout) - 1`. Это «застывание» индикатора на
/// T4 после HLT. При `use_full_duration=true` HLT-layout = `[7]`,
/// клампинг тоже работает (но сам по себе ничего не режет, потому
/// что `last_completed_tact_phase = 6 < 7`).
fn position_at(
    cpu: &Cpu8080State,
    phase_source: Option<u8>,
    use_full_duration: bool,
) -> Option<MachineCyclePosition> {
    let opcode = cpu.last_fetched_opcode;
    decode_opcode(opcode).ok()?;
    let layout = if use_full_duration {
        full_duration_layout(opcode)
    } else {
        layout_for(opcode)
    };
    let phase = phase_source?;
    let taken_total = layout.total_t_states(true);
    let not_taken_total = layout.total_t_states(false);
    let clamped_taken = phase.min(taken_total.saturating_sub(1));
    let clamped_not_taken = phase.min(not_taken_total.saturating_sub(1));
    position_for(layout, true, clamped_taken)
        .or_else(|| position_for(layout, false, clamped_not_taken))
}

/// Обернуть строку «Подпись  ...  Значение» в tooltip с подробным
/// объяснением. Tooltip-body использует тот же `inset_style` что и
/// chips на нижней панели — визуально читается как часть одной семьи.
/// Задержка 600 мс — пользователь не получает мгновенный pop-up при
/// случайном пролёте мыши, но информацию видит без долгого ожидания.
///
/// `label_short` — короткая подпись для строки (помещается в узкий
/// блок без переноса). `value_text` — то что справа, обычно число
/// или `-`. `hint` — полное объяснение, появляется в tooltip.
fn labeled_row_with_tooltip(
    label_short: &'static str,
    value_text: String,
    hint: &'static str,
) -> Element<'static, Message> {
    use std::time::Duration;

    let face = row![
        ui_text(label_short, 12, TOKYO_MUTED),
        Space::new().width(Length::Fill),
        mono_text(value_text, 14, TOKYO_GREEN),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center);

    // Контейнер нужен чтобы `tooltip` принимал виджет, а не Row
    // напрямую: Row → Element имплементирует `Into`, но tooltip
    // ожидает widget с фиксированной шириной для корректного
    // якорения. `width(Fill)` гарантирует что hover-зона покрывает
    // всю строку, не только подпись.
    let face_container = container(face).width(Length::Fill);

    let body = container(ui_text(hint, 12, TOKYO_TEXT))
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .max_width(230.0)
        .style(inset_style);

    tooltip(face_container, body, tooltip::Position::Top)
        .gap(4.0)
        .padding(0.0)
        .delay(Duration::from_millis(600))
        .snap_within_viewport(true)
        .into()
}

pub(super) fn cycle_panels(cpu: &Cpu8080State) -> Element<'static, Message> {
    // «Активная» позиция: где сейчас находится исполнение. Для
    // отображения по M-циклам (M / T в текущем M-цикле) при
    // `tact_phase == None` мы fallback-имся на
    // `last_completed_tact_phase`, чтобы блок «Цикл и такт» не
    // сбрасывался в `-` после `HLT` или на границе инструкции —
    // удерживаем на индикаторе последнюю выполненную пару M/T.
    let active_phase = cpu.tact_phase.or(cpu.last_completed_tact_phase);

    // «Цикл» — layout по M-циклам (для HLT даёт M=1, т.к. M2
    // halt-ack не показывается). Берём m_cycle, t_in_cycle игнорируем.
    let cycle_active = position_at(cpu, active_phase, false);

    // «Такт» — layout по M-циклам, всегда из
    // `last_completed_tact_phase` без fallback на active_phase. Для
    // HLT даёт T=4 (клампится). Это «горящий такт» — последняя
    // выполненная T-фаза, удерживаемая на индикаторе после остановки.
    let tact_last_completed = position_at(cpu, cpu.last_completed_tact_phase, false);

    // «Такт инструкции» — layout полной длительности: для HLT
    // layout=[7] и `position_for([7], true, 6)` даёт `t_in_cycle = 7`.
    // Это «честный такт по техническому описанию» включая
    // halt-acknowledge цикл HLT. Берём t_in_cycle, m_cycle игнорируем
    // (для склеенного layout полной длительности он всегда 1 —
    // единый блок).
    let full_duration_active = position_at(cpu, active_phase, true);

    let cycle_text = match cycle_active {
        Some(pos) => pos.m_cycle.to_string(),
        None => "-".to_owned(),
    };
    let tact_text = match tact_last_completed {
        Some(pos) => pos.t_in_cycle.to_string(),
        None => "-".to_owned(),
    };
    let tact_full_text = match full_duration_active {
        Some(pos) => pos.t_in_cycle.to_string(),
        None => "-".to_owned(),
    };

    // Заголовок блока — отдельная строка с центрированной подписью.
    // Width(Fill) внутри контейнера фиксированной ширины растягивает
    // Row на всю ширину блока, align_x(Center) для текста ставит
    // подпись по центру.
    let cycle_header = container(ui_text("Цикл и такт", 11, TOKYO_MUTED))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center);

    // Блок 1: «Цикл и такт» — позиция по машинным циклам. Две
    // строки: номер M-цикла и T-фаза внутри него (с клампингом для
    // HLT). Каждая строка обёрнута в tooltip с полным объяснением.
    let cycle_block = container(
        column![
            cycle_header,
            labeled_row_with_tooltip(
                "Цикл",
                cycle_text,
                "Какой по счёту шаг сейчас выполняет команда. \
                 Простые команды делают всё за один шаг, сложные \
                 (например вызов подпрограммы) – за несколько.",
            ),
            labeled_row_with_tooltip(
                "Такт",
                tact_text,
                "Номер такта внутри текущего шага команды. \
                 После остановки удерживается на последнем \
                 выполненном такте.",
            ),
        ]
        .spacing(6),
    )
    .padding(10)
    .width(Length::Fixed(CYCLE_BLOCK_WIDTH))
    .style(schematic_block_style);

    // Блок 2: «Внутренний тайминг» — модель полной длительности.
    // Три строки:
    //   - Тактов: сквозной cycle_count от начала программы.
    //   - Такт инструкции: номер такта внутри текущей инструкции по
    //     полной длительности из технического описания (для HLT — 7,
    //     включая halt-acknowledge).
    //   - Фаза: индекс tact_phase или last_completed (с 0).
    //     Звёздочка `*` означает «инструкция завершена, показано
    //     последнее значение».
    let linear_phase_text = match (cpu.tact_phase, cpu.last_completed_tact_phase) {
        (Some(phase), _) => phase.to_string(),
        (None, Some(last)) => format!("{last}*"),
        (None, None) => "-".to_owned(),
    };

    let timing_header = container(ui_text("Внутренний тайминг", 11, TOKYO_MUTED))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center);

    let our_block = container(
        column![
            timing_header,
            labeled_row_with_tooltip(
                "Тактов",
                cpu.cycle_count.to_string(),
                "Сколько тактов всего прошло с начала программы. \
                 Сбрасывается при сбросе процессора.",
            ),
            labeled_row_with_tooltip(
                "Такт инструкции",
                tact_full_text,
                "Номер такта внутри текущей команды по полной \
                 длительности из технического описания. Считает все \
                 такты команды подряд, в том числе те, что блок \
                 «Цикл и такт» скрывает (например у HLT – 7, а не 4).",
            ),
            labeled_row_with_tooltip(
                "Фаза",
                linear_phase_text,
                "То же, что «Такт инструкции», но считается с нуля. \
                 Звёздочка после числа – команда уже завершилась, \
                 показано последнее значение.",
            ),
        ]
        .spacing(6),
    )
    .padding(10)
    .width(Length::Fixed(TIMING_BLOCK_WIDTH))
    .style(schematic_block_style);

    row![
        cycle_block,
        Space::new().width(Length::Fixed(12.0)),
        our_block
    ]
    .into()
}
