use std::fmt::Write;

use crate::backend::{DeviceStatus, PrinterState, decode_oem_text};
use iced::widget::{Space, column, container, mouse_area, opaque, row, scrollable, stack};
use iced::{Element, Length, Padding, alignment};

use super::icons;
use super::storage::chrome::{
    device_backdrop_style, device_buffer_style, icon_button, window_controls,
};
use super::storage::status_label;
use super::styles::{panel_style, scrollable_style};
use super::theme::{MONO_FONT, tokyo_muted, tokyo_text, ui_text};
use crate::app::{Message, ToolWindowKind};
use crate::i18n::{Key, Lang, PrinterKey, lowercase_initial};

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 340.0;

pub(in crate::view) fn printer_window_overlay(
    state: &PrinterState,
    text_view: bool,
    printer_target: String,
    lang: Lang,
) -> Element<'_, Message> {
    let backdrop: Element<'_, Message> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(device_backdrop_style),
    )
    .on_press(Message::ClosePrinter)
    .into();
    let dialog = container(printer_content(
        state,
        text_view,
        printer_target,
        false,
        false,
        lang,
    ))
    .padding(16)
    .style(panel_style)
    .width(Length::Fixed(WINDOW_WIDTH))
    .height(Length::Fixed(WINDOW_HEIGHT));

    stack![opaque(backdrop), centered(opaque(dialog))]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(in crate::view) fn printer_window(
    state: &PrinterState,
    text_view: bool,
    printer_target: String,
    always_on_top: bool,
    lang: Lang,
) -> Element<'_, Message> {
    container(printer_content(
        state,
        text_view,
        printer_target,
        true,
        always_on_top,
        lang,
    ))
    .padding(16)
    .style(panel_style)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn printer_content(
    state: &PrinterState,
    text_view: bool,
    printer_target: String,
    detached: bool,
    always_on_top: bool,
    lang: Lang,
) -> Element<'_, Message> {
    let body = column![
        buffer_panel(state, text_view, lang),
        footer(state, printer_target, lang)
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill);
    column![
        header(state, text_view, detached, always_on_top, lang),
        Space::new().height(Length::Fixed(12.0)),
        body,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn header(
    state: &PrinterState,
    text_view: bool,
    detached: bool,
    always_on_top: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let busy = state.status == DeviceStatus::Busy;
    let (print_enabled, clear_enabled) = printer_actions_enabled(state);
    row![
        window_controls(ToolWindowKind::Printer, detached, always_on_top, lang),
        icon_button(
            icons::type_icon(),
            Some(Message::TogglePrinterBufferView),
            lang.t(Key::Printer(if text_view {
                PrinterKey::ShowBytes
            } else {
                PrinterKey::ShowText
            })),
            text_view,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::settings(),
            Some(Message::ConfigurePrinterSession),
            lang.t(Key::Printer(PrinterKey::ConfigureSession)),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::device_printer(),
            print_enabled.then_some(Message::PrintPrinterNative),
            lang.t(Key::Printer(PrinterKey::PrintNative)),
            busy,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            clear_enabled.then_some(Message::ClearPrinterBuffer),
            lang.t(Key::Printer(PrinterKey::ClearBuffer)),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Some(Message::ClosePrinter),
            lang.t(Key::MonitorClose),
            false,
            None,
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

fn printer_actions_enabled(state: &PrinterState) -> (bool, bool) {
    (state.status != DeviceStatus::Busy, true)
}

fn buffer_panel<'a>(state: &'a PrinterState, text_view: bool, lang: Lang) -> Element<'a, Message> {
    let content = if text_view {
        format_printer_text(&state.spool)
    } else {
        format_printer_buffer(&state.spool)
    };
    let empty = content.is_empty();
    let scroll = scrollable(
        container(
            iced::widget::text(content)
                .font(MONO_FONT)
                .size(11)
                .color(tokyo_text())
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .padding(if empty { [34, 12] } else { [12, 12] })
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme, status| scrollable_style(true, theme, status));
    let frame = container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(device_buffer_style)
        .clip(true);
    if empty {
        let label = container(ui_text(
            lang.t(Key::Printer(PrinterKey::BufferContents)),
            13,
            tokyo_muted(),
        ))
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .width(Length::Fill);
        stack![frame, label]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        frame.into()
    }
}

fn footer(state: &PrinterState, printer_target: String, lang: Lang) -> Element<'static, Message> {
    let status = lowercase_initial(&match &state.status {
        DeviceStatus::Error(_) => lang.t(Key::Printer(PrinterKey::PrintFailed)).to_owned(),
        status => status_label(status, lang),
    });
    let status = format!(
        "{}: {}",
        lang.t(Key::Printer(PrinterKey::Status)),
        compact_footer_value(&status, 34),
    );
    let buffered = format!(
        "{}: {}",
        lang.t(Key::Printer(PrinterKey::BytesBuffered)),
        state.bytes_buffered,
    );
    let target = format!(
        "{}: {}",
        lang.t(Key::Printer(PrinterKey::Target)),
        compact_footer_value(&printer_target, 30),
    );
    container(
        iced::widget::text(format!("{status}   {buffered}   {target}"))
            .font(MONO_FONT)
            .size(12)
            .color(tokyo_text())
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .width(Length::Fill)
    .clip(true)
    .into()
}

fn compact_footer_value(value: &str, maximum: usize) -> String {
    let mut characters = value.chars();
    let head = characters.by_ref().take(maximum).collect::<String>();
    if characters.next().is_none() {
        head
    } else {
        format!(
            "{}...",
            head.chars()
                .take(maximum.saturating_sub(3))
                .collect::<String>()
        )
    }
}

fn centered<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            content,
            Space::new().width(Length::Fill)
        ],
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn format_printer_buffer(bytes: &[u8]) -> String {
    let mut output = String::new();
    for (line, chunk) in bytes.chunks(16).enumerate() {
        if line != 0 {
            output.push('\n');
        }
        let _ = write!(output, "{:04X}:", line * 16);
        for byte in chunk {
            let _ = write!(output, " {byte:02X}");
        }
    }
    output
}

fn format_printer_text(bytes: &[u8]) -> String {
    decode_oem_text(bytes).replace('\t', "    ")
}

#[cfg(test)]
mod tests {
    use super::{
        compact_footer_value, format_printer_buffer, format_printer_text, printer_actions_enabled,
    };
    use crate::backend::{DeviceStatus, PrinterState};
    use std::path::PathBuf;

    #[test]
    fn printer_buffer_uses_hex_offsets_and_sixteen_bytes_per_line() {
        let bytes = (0..18).collect::<Vec<_>>();

        assert_eq!(
            format_printer_buffer(&bytes),
            "0000: 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n0010: 10 11"
        );
    }

    #[test]
    fn printer_text_view_decodes_cp866_and_normalizes_controls() {
        assert_eq!(
            format_printer_text(&[0x8F, 0xE0, b'!', b'\r', b'\n', b'\t', 0x01]),
            "Пр!\n    ·"
        );
    }

    #[test]
    fn printer_actions_are_available_for_empty_ready_spool() {
        assert_eq!(
            printer_actions_enabled(&printer_state(DeviceStatus::Ready)),
            (true, true)
        );
    }

    #[test]
    fn printer_clear_remains_available_while_printing() {
        assert_eq!(
            printer_actions_enabled(&printer_state(DeviceStatus::Busy)),
            (false, true)
        );
    }

    #[test]
    fn printer_footer_values_are_bounded_without_splitting_unicode() {
        assert_eq!(compact_footer_value("коротко", 10), "коротко");
        assert_eq!(
            compact_footer_value("Очень длинное имя принтера", 12),
            "Очень дли..."
        );
    }

    fn printer_state(status: DeviceStatus) -> PrinterState {
        PrinterState {
            spool: Vec::new(),
            target_path: None::<PathBuf>,
            status,
            bytes_buffered: 0,
            last_error: None,
        }
    }
}
