use iced::widget::{Column, container, stack, text};
use iced::{Element, Font, Length};

use super::theme::{MONO_FONT, UI_BOLD_FONT, UI_FONT, tokyo_text};
use crate::app::Message;

const WARMUP_SIZES: [f32; 9] = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 18.0, 24.0];
const WARMUP_BOLD_SIZES: [f32; 4] = [12.0, 14.0, 16.0, 18.0];
const WARMUP_MONO_SIZES: [f32; 8] = [8.0, 9.0, 12.0, 13.0, 14.0, 16.0, 20.0, 24.0];
const WARMUP_TEXT: &str = concat!(
    "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ ",
    "абвгдеёжзийклмнопрстуфхцчшщъыьэюя ",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz ",
    "0123456789 0000 FFFF PC SP HLT NOP LXI MVI OUT IN d8 d16 a16 ",
    "Файл МП-Система Вид Настройки Помощь Статус Импорт из ",
    "Содержимое ячеек ОЗУ Адрес Значение Команда Регистры и операнды ",
    "Аккумулятор Буферный регистр Цикл и такт Внутренние тайминги ",
    "Сигналы управления Текущая команда Операнд Длина Тип Адресация ",
    "Быстрый доступ Скорость Выполнение Сброс управление пересылка ",
    "непосредств неявная ввод/вывод D:\\kr-580\\test_ports.txt <>:;,.+-_/\\()[]"
);
const WARMUP_MONO_TEXT: &str = concat!(
    "0000 0001 0002 0003 0004 0005 0006 0007 0008 0009 000A 000B 000C ",
    "00 01 02 03 04 05 06 07 08 09 0A 0B 0C FF FFFF PC SP T HLT ВЫКЛ ",
    "NOP OUT IN LXI MVI INR ANI MOV CALL RET JMP d8 d16 a16 B C D E H L M ",
    "Z S P C AC READY WAIT HOLD INTE DBIN WR HLDA ▲▼ ✓ ● » << >>"
);

pub(super) fn wrap_startup<'a>(
    startup_frames_seen: u8,
    app: Element<'a, Message>,
) -> Element<'a, Message> {
    if startup_frames_seen >= 2 {
        return app;
    }

    stack![font_warmup_layer(), app]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn font_warmup_layer() -> Element<'static, Message> {
    let regular = WARMUP_SIZES
        .into_iter()
        .map(|size| font_warmup_line(UI_FONT, size));
    let bold = WARMUP_BOLD_SIZES
        .into_iter()
        .map(|size| font_warmup_line(UI_BOLD_FONT, size));
    let mono = WARMUP_MONO_SIZES
        .into_iter()
        .map(|size| mono_warmup_line(size));

    container(Column::with_children(regular.chain(bold).chain(mono)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn mono_warmup_line(size: f32) -> Element<'static, Message> {
    text(WARMUP_MONO_TEXT)
        .font(MONO_FONT)
        .size(size)
        .color(tokyo_text())
        .into()
}

fn font_warmup_line(font: Font, size: f32) -> Element<'static, Message> {
    text(WARMUP_TEXT)
        .font(font)
        .size(size)
        .color(tokyo_text())
        .into()
}
