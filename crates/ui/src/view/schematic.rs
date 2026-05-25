//! Left-hand "board" that imitates a CPU schematic with live readouts.
//!
//! Everything in this module is a pure function of the latest
//! `AppSnapshot`: status strip, registers, multiplexer, control lamps,
//! and the I/O device row. The multiplexer panel lives in `mux.rs` and
//! the lamp strip in `lamps.rs` so this file stays focused on the
//! framing logic that ties the panels together.

use iced::widget::{Space, column, container, mouse_area, row};
use iced::{Element, Length, alignment};
use k580_core::{Cpu8080State, RegisterName, decode_opcode};

use super::chips::{
    device_chip, flag_strip, functional_block, schematic_mnemonic_readout, schematic_readout,
};
use super::icons;
use super::lamps::control_lamps;
use super::styles::schematic_board_style;
use super::theme::{
    TOKYO_BLUE, TOKYO_CYAN, TOKYO_GREEN, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_RED, TOKYO_TEXT,
    TOKYO_YELLOW, mono_text, ui_text,
};
use crate::app::{DesktopApp, Message, SpeedTier};

impl DesktopApp {
    pub(super) fn schematic_panel(&self) -> Element<'_, Message> {
        let cpu = &self.snapshot.cpu;

        let halt_indicator = mouse_area(mono_text(
            // The label is `HLT` (the mnemonic of the Intel 8080
            // instruction that flips the halt flip-flop), not
            // `HALT` and not `HLDA`. `HALT` was a generic English
            // word for the internal state — fine in prose, but in
            // a strip of three-letter chip-style readouts next to
            // PC/SP/T it read as "name of a pin on the chip" and
            // the user pointed out the mismatch with both their
            // mental model and with `halt_notice`, which already
            // talks about "флаг HLT". `HLDA` is something else
            // entirely — that's the Hold Acknowledge pin (output
            // 21 on the 8080 corner), wired to the DMA arbiter
            // and unrelated to the halt flip-flop; it lives on
            // the control-lamp row at the bottom of the panel
            // where it actually belongs. Using `HLT` here keeps
            // the indicator, the halt-notice, and the register
            // editor all calling the same thing the same name.
            //
            // The chip is wrapped in a `mouse_area` so the press
            // edge dispatches `Message::ToggleHalt` — the user
            // asked the indicator itself to be the affordance
            // for flipping the bit, instead of a passive readout
            // hidden behind a menu entry / shortcut. `Pointer`
            // interaction telegraphs the click target the way
            // every other `mouse_area`-wrapped chip on the panel
            // does (memory rows, register chips). We use
            // `mouse_area` rather than `button` for the same
            // reason `mux_register` does: press-edge handlers
            // snap the toggle in the same frame the click lands,
            // whereas `button` only fires on release.
            if cpu.halted {
                "HLT ВКЛ"
            } else {
                "HLT ВЫКЛ"
            },
            13,
            if cpu.halted { TOKYO_RED } else { TOKYO_GREEN },
        ))
        .on_press(Message::ToggleHalt)
        .interaction(iced::mouse::Interaction::Pointer);

        let status_strip = row![
            super::status_register::status_register_inline(cpu),
            Space::new().width(Length::Fixed(18.0)),
            mono_text(format!("PC {:04X}", cpu.pc), 13, TOKYO_BLUE),
            mono_text(format!("SP {:04X}", cpu.sp), 13, TOKYO_CYAN),
            mono_text(format!("T {}", cpu.cycle_count), 13, TOKYO_YELLOW),
            halt_indicator,
            Space::new().width(Length::Fill),
            ui_text("Статус", 12, TOKYO_MUTED),
            mono_text(&self.status, 13, TOKYO_TEXT),
        ]
        .spacing(14)
        .align_y(alignment::Vertical::Center);

        let top_bus_row = row![
            // PSW = программно-видимая регистровая пара (A, F), 16 бит.
            // Именно её 8080 кладёт в стек по `PUSH PSW` и достаёт по
            // `POP PSW`: hi-байт = аккумулятор, lo-байт = регистр
            // признаков (F) в формате `S Z 0 AC 0 P 1 C`. Раньше здесь
            // выводился только `flags.to_psw()` как `{:04X}`, что давало
            // `00C5` вместо `C5` (лишние нули) и при этом дублировало F,
            // уже нарисованный в блоке «Регистр признаков» ниже. Склейка
            // (A << 8) | F даёт настоящие 16 бит PSW и совпадает с
            // datasheet Intel 8080A — пользователь видит ровно тот байт,
            // что попадёт в стек при `PUSH PSW`.
            schematic_readout(
                "PSW",
                format!(
                    "{:04X}",
                    ((cpu.registers.a as u16) << 8) | cpu.flags.to_psw() as u16,
                ),
                TOKYO_GREEN,
            ),
            flag_strip(cpu),
            Space::new().width(Length::Fill),
            schematic_readout(
                "Буфер данных",
                // `last_data_bus_byte` — зеркало физического Буфера
                // Данных: последний байт, прошедший по D7-D0 в любую
                // сторону. Раньше здесь стоял `cpu.memory.read(pc)` —
                // look-ahead в RAM по новому PC, который после `HLT`
                // показывал 00 (NOP в очищенной RAM), а школьный
                // эталон показывает 76 (опкод HLT, последний байт
                // через шину). Семантика теперь совпадает с
                // референсом для всех инструкций: после `STA 0x4000`
                // здесь записанное значение A, после `LDA 0x4000` —
                // прочитанный байт, после `HLT` — 76.
                format!("{:02X}", cpu.last_data_bus_byte),
                TOKYO_GREEN,
            ),
        ]
        .spacing(18)
        .align_y(alignment::Vertical::Center);

        let core_left = column![
            row![
                functional_block(
                    "Аккумулятор",
                    format!("{:02X}", cpu.registers.a),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::A),
                ),
                Space::new().width(Length::Fill),
                functional_block(
                    "Буферный регистр 1",
                    format!("{:02X}", cpu.registers.b),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::B),
                ),
                Space::new().width(Length::Fill),
                schematic_readout(
                    "Регистр признаков",
                    // Восемь битов младшего байта PSW в стандартном
                    // порядке Intel 8080A: `S Z 0 AC 0 P 1 C`. Это тот
                    // самый формат, в котором F попадает в стек по
                    // `PUSH PSW` и достаётся по `POP PSW` — datasheet
                    // 8080A, рис. «Program Status Word». Раньше тут
                    // были только пять «живых» битов в порядке Z S P
                    // C AC (как у точек-индикаторов `flag_strip`), но
                    // школьный референсный эмулятор и советская
                    // документация на КР580ВМ80 показывают полный
                    // байт с аппаратными константами на позициях 5, 3
                    // (всегда 0) и 1 (всегда 1). Совпадает 1-в-1 с
                    // надписью «Рег.признаков 1000 0 01» в оригинале:
                    // верхний полубайт = `S Z 0 AC`, нижний = `0 P 1
                    // C`. `flag_strip` над шиной оставлен как был —
                    // там 5 точек по живым флагам, цель другая.
                    //
                    // Название блока — «Регистр признаков», как в
                    // советских учебниках по КР580ВМ80 и на схеме
                    // оригинального эмулятора. «Регистр флагов» —
                    // англо-калька (`Flag Register`), которая в
                    // отечественной литературе по 8080 не
                    // используется. F-регистр программно невидим
                    // (отдельной мнемоники для него нет), доступ
                    // только через PSW-пару, поэтому подпись блока
                    // именно «признаков», а не «F».
                    format!(
                        "{}{}{}{} {}{}{}{}",
                        u8::from(cpu.flags.sign),
                        u8::from(cpu.flags.zero),
                        0,
                        u8::from(cpu.flags.auxiliary_carry),
                        0,
                        u8::from(cpu.flags.parity),
                        1,
                        u8::from(cpu.flags.carry),
                    ),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
            row![
                functional_block(
                    "Буферный регистр 2",
                    format!("{:02X}", cpu.registers.c),
                    TOKYO_GREEN,
                    Message::RegisterSelected(RegisterName::C),
                ),
                Space::new().width(Length::Fill),
                schematic_readout(
                    "Регистр команд",
                    // `last_fetched_opcode` — зеркало физического РК.
                    // Раньше здесь стоял `memory.read(pc)`, look-ahead
                    // в RAM по новому PC: после `HLT` он показывал
                    // 00 (NOP в очищенной RAM), а школьный эталон —
                    // 76 (опкод HLT, последний загруженный байт). РК
                    // на чипе хранит опкод до **начала следующего
                    // M1**, а не до изменения PC, и теперь UI это
                    // отражает. Та же точка контроля закрывает Д/Ш
                    // команд и Буфер данных ниже.
                    format!("{:02X}", cpu.last_fetched_opcode),
                    TOKYO_GREEN,
                ),
                Space::new().width(Length::Fill),
                // "Д/Ш команд" — Дешифратор/Шифратор команд, the
                // instruction decoder. The reference schematic
                // paints it right next to «Регистр команд» as a
                // sibling readout that shows the **mnemonic** of
                // the byte the IR currently holds: where IR shows
                // `3C`, Д/Ш shows `INR A`. Decoded through
                // `decode_opcode(byte).mnemonic`, which already
                // produces the formatted "OP arg" form
                // (`MVI B, d8`, `JNZ adr`, `LXI B, d16`) used by
                // the memory-list mnemonic column. Falls back to
                // `-` on undocumented bytes so the readout never
                // goes blank — same idiom as the memory list.
                //
                // Декодируем тот же байт, что в РК
                // (`last_fetched_opcode`), а не `memory.read(pc)`:
                // декодер всегда работает с последним загруженным
                // опкодом, иначе после `HLT` Д/Ш показывал бы `NOP`
                // вместо `HLT` — расхождение со школьным эталоном.
                schematic_mnemonic_readout(
                    "Д/Ш команд",
                    decode_opcode(cpu.last_fetched_opcode)
                        .map(|info| info.mnemonic)
                        .unwrap_or_else(|_| "-".to_owned()),
                    TOKYO_GREEN,
                ),
            ]
            .spacing(14),
            // Standalone "Буфер адреса" row — the reference panel
            // paints it under the mux as a single readout that
            // mirrors whichever 16-bit address the chip is about
            // to drive on the А0–А15 bus. Источник — теперь
            // `last_address_bus` (физический Буфер Адреса), а не
            // PC: после `STA 0x4000` латч хранит 0x4000, после
            // `LDA 0x4000` — тоже 0x4000, после `HLT` PC=0x0009,
            // но адресный буфер хранит адрес самого HLT'а
            // (0x0008). Это закрывает четвёртое расхождение со
            // школьным эталоном и делает читаемой одну и ту же
            // ось адресов через все инструкции (M1 fetch, operand
            // fetch, stack push/pop, прямой LDA/STA/LHLD/SHLD).
            row![schematic_readout(
                "Буфер адреса",
                format!("{:04X}", cpu.last_address_bus),
                TOKYO_GREEN,
            )]
            .spacing(14),
        ]
        .spacing(14)
        .width(Length::FillPortion(3));

        let multiplexer = mux_panel(cpu, self.selected_register);

        // `FillPortion(2)` оставлен: row забирает 2/N доступной
        // высоты плиты, остальное идёт bottom-блоку (циклы +
        // лампы + устройства). Высоты внутри `mux_panel` ужаты
        // (см. `mux.rs`: чипы 38→30 px, padding 10→6, заголовок
        // [8,7]→[4,7]) так чтобы интринзик мультиплексора (~265 px)
        // помещался в эту долю на дефолтном окне ≈720-800 px по
        // вертикали. Без ужимки нижние 2 строки мультиплексора
        // («Счётчик команд», «Схема инкремента-декремента»)
        // обрезались, без `FillPortion` — обрезались лампы и
        // устройства внизу плиты. Компромисс: компактнее чипы +
        // фракционная доля = всё помещается на любом размере.
        let core_plane = row![core_left, multiplexer]
            .spacing(16)
            .height(Length::FillPortion(2));

        let low_control = row![
            super::cycles::cycle_panels(cpu),
            Space::new().width(Length::Fill),
            speed_panel(self.speed_tier),
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);

        // Five peripheral chips, each rendered as a tinted SVG inside the
        // schematic block chrome and wrapped in a tooltip the same way the
        // action-panel buttons (`icon_action_button`) wire theirs. Order
        // matches the user's reference diagram: Монитор, Дисковод (FDD),
        // Диск (HDD), Адаптер (сетевой), Принтер. The `I/O Controller`
        // capsule that used to sit on the right of this row was removed —
        // it duplicated the role of the device strip without carrying any
        // live readout of its own.
        let devices = row![
            device_chip(icons::device_monitor(), TOKYO_GREEN, "Отобразить монитор",),
            device_chip(
                icons::device_floppy(),
                TOKYO_CYAN,
                "Отобразить буфер дисковода",
            ),
            device_chip(
                icons::device_hdd(),
                TOKYO_BLUE,
                "Отобразить буфер жёсткого диска",
            ),
            device_chip(
                icons::device_network(),
                TOKYO_YELLOW,
                "Отобразить буфер сетевого адаптера",
            ),
            device_chip(
                icons::device_printer(),
                TOKYO_MAGENTA,
                "Отобразить буфер принтера",
            ),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let bottom = column![low_control, control_lamps(cpu), devices,].spacing(10);

        let content = column![status_strip, top_bus_row, core_plane, bottom,]
            .spacing(16)
            .height(Length::Fill);

        container(content)
            .padding(18)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(schematic_board_style)
            .into()
    }
}

// ---------------------------------------------------------------------------
// schematic-specific helpers
// ---------------------------------------------------------------------------
//
// The pure widget builders that paint individual chips on this plate
// (`schematic_readout`, `schematic_mnemonic_readout`, `flag_strip`,
// `device_chip`, `functional_block`) live in `super::chips`. They were
// extracted to keep this file under the workspace's 400-line ceiling
// (see `AGENTS.md`) — call sites read `chips::schematic_readout(...)`
// the same way they used to read the local function.

/// Builds the "Мультиплексор" panel — implementation lives in
/// `super::mux` to keep this file under the workspace's 400-line
/// ceiling. Re-exported here as a one-liner so the call site reads
/// the same way it did before the split.
fn mux_panel(cpu: &Cpu8080State, selected: RegisterName) -> Element<'static, Message> {
    super::mux::mux_panel(cpu, selected)
}

/// Four-tier speed switch — implementation lives in `super::speed`
/// to keep this file under the 400-line ceiling. Re-exported here as
/// a one-liner so the call site reads the same way it did before
/// the split.
fn speed_panel(active: SpeedTier) -> Element<'static, Message> {
    super::speed::speed_panel(active)
}
