use std::fmt::Write;

use iced::widget::{Space, column, container, mouse_area, opaque, row, scrollable, stack};
use iced::{Element, Length, Padding, alignment};
use k580_app::{ConnectionState, DeviceStatus, NetworkMode, NetworkState};

use super::icons;
use super::storage::chrome::{
    device_backdrop_style, device_buffer_style, icon_button, window_controls,
};
use super::styles::{panel_style, scrollable_style};
use super::theme::{MONO_FONT, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use super::tooltips::shortcut_hint;
use crate::app::{Message, ToolWindowKind};
use crate::i18n::{Key, Lang, NetworkKey};

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 340.0;

pub(in crate::view) struct NetworkViewState<'a> {
    pub(in crate::view) network: &'a NetworkState,
    pub(in crate::view) settings_open: bool,
    pub(in crate::view) mode: NetworkMode,
    pub(in crate::view) host: &'a str,
    pub(in crate::view) port: &'a str,
    pub(in crate::view) error: Option<&'a str>,
    pub(in crate::view) lang: Lang,
}

pub(in crate::view) fn network_window_overlay(view: NetworkViewState<'_>) -> Element<'_, Message> {
    let backdrop: Element<'_, Message> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(device_backdrop_style),
    )
    .on_press(Message::CloseNetwork)
    .into();
    let dialog = container(network_content(view, false, false))
        .padding(16)
        .style(panel_style)
        .width(Length::Fixed(WINDOW_WIDTH))
        .height(Length::Fixed(WINDOW_HEIGHT));
    let centered = center(opaque(dialog));

    stack![opaque(backdrop), centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(in crate::view) fn network_window(
    view: NetworkViewState<'_>,
    always_on_top: bool,
) -> Element<'_, Message> {
    container(network_content(view, true, always_on_top))
        .padding(16)
        .style(panel_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn network_content(
    view: NetworkViewState<'_>,
    detached: bool,
    always_on_top: bool,
) -> Element<'_, Message> {
    let settings_open = view.settings_open;
    let buffers = row![
        buffer_panel(
            view.lang.t(Key::Network(NetworkKey::RxBuffer)),
            format_network_buffer(&view.network.rx_buffer),
        ),
        buffer_panel(
            view.lang.t(Key::Network(NetworkKey::LastTransmittedValue)),
            format_last_transmitted_value(&view.network.tx_buffer),
        ),
    ]
    .spacing(12)
    .height(Length::Fill);
    let device_body = column![buffers, footer(view.network, view.lang)]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);
    let body: Element<'_, Message> = column![
        header(detached, always_on_top, settings_open, view.lang),
        Space::new().height(Length::Fixed(12.0)),
        device_body,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if settings_open {
        stack![body, super::network_settings::settings_overlay(view)].into()
    } else {
        body
    }
}

fn header(
    detached: bool,
    always_on_top: bool,
    settings_open: bool,
    lang: Lang,
) -> Element<'static, Message> {
    row![
        window_controls(ToolWindowKind::Network, detached, always_on_top, lang),
        icon_button(
            icons::globe(),
            Some(Message::OpenNetworkSettings),
            lang.t(Key::Network(NetworkKey::Settings)),
            settings_open,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Some(Message::ClearNetworkBuffers),
            lang.t(Key::Network(NetworkKey::ClearBuffers)),
            false,
            None,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Some(Message::CloseNetwork),
            lang.t(Key::MonitorClose),
            false,
            shortcut_hint(&Message::CloseNetwork),
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

fn buffer_panel(title: &'static str, text: String) -> Element<'static, Message> {
    let empty = text.is_empty();
    let content = scrollable(
        container(
            iced::widget::text(text)
                .font(MONO_FONT)
                .size(11)
                .color(TOKYO_TEXT)
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .padding(if empty { [34, 12] } else { [12, 12] })
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme, status| scrollable_style(true, theme, status));
    let frame = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(device_buffer_style)
        .clip(true);
    let label = container(ui_text(title, 13, TOKYO_MUTED))
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .width(Length::Fill);

    if empty {
        stack![frame, label]
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    } else {
        frame
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    }
}

fn footer<'a>(state: &'a NetworkState, lang: Lang) -> Element<'a, Message> {
    let mode = match state.mode {
        NetworkMode::Client => lang.t(Key::Network(NetworkKey::ModeClient)),
        NetworkMode::Server => lang.t(Key::Network(NetworkKey::ModeServer)),
    };
    let status = network_status(state, lang);
    let meta = format!(
        "{}: {status}   {}: {mode}   {}: {}:{}   {}: {}   {}: {}",
        lang.t(Key::Network(NetworkKey::Status)),
        lang.t(Key::Network(NetworkKey::Mode)),
        lang.t(Key::Network(NetworkKey::Endpoint)),
        state.host,
        state.port,
        lang.t(Key::Network(NetworkKey::RxTotal)),
        state.rx_total,
        lang.t(Key::Network(NetworkKey::TxTotal)),
        state.tx_total,
    );
    iced::widget::text(meta)
        .font(MONO_FONT)
        .size(12)
        .color(TOKYO_TEXT)
        .wrapping(iced::widget::text::Wrapping::None)
        .into()
}

pub(super) fn center<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
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

fn format_network_buffer(bytes: &[u8]) -> String {
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

fn format_last_transmitted_value(bytes: &[u8]) -> String {
    bytes
        .last()
        .map(|byte| format!("{byte:02X}"))
        .unwrap_or_default()
}

fn network_status(state: &NetworkState, lang: Lang) -> String {
    match &state.connection {
        ConnectionState::Refused => lang
            .t(Key::Network(NetworkKey::ConnectionRefused))
            .to_owned(),
        ConnectionState::TimedOut => lang
            .t(Key::Network(NetworkKey::ConnectionTimedOut))
            .to_owned(),
        ConnectionState::Error(_) => lang.t(Key::Network(NetworkKey::ConnectionError)).to_owned(),
        _ => match &state.status {
            DeviceStatus::Ready => lang.t(Key::DeviceStatusReady).to_owned(),
            DeviceStatus::NotReady => lang.t(Key::DeviceStatusNotReady).to_owned(),
            DeviceStatus::Busy => lang.t(Key::DeviceStatusBusy).to_owned(),
            DeviceStatus::NoData => lang.t(Key::DeviceStatusNoData).to_owned(),
            DeviceStatus::Connected => lang.t(Key::DeviceStatusConnected).to_owned(),
            DeviceStatus::Listening => lang.t(Key::DeviceStatusListening).to_owned(),
            DeviceStatus::Disconnected => lang.t(Key::DeviceStatusDisconnected).to_owned(),
            DeviceStatus::Error(_) => lang.t(Key::Network(NetworkKey::ConnectionError)).to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{format_last_transmitted_value, format_network_buffer, network_status};
    use crate::i18n::Lang;
    use k580_app::{ConnectionState, DeviceStatus, NetworkMode, NetworkState};

    #[test]
    fn network_buffer_uses_hex_offsets_and_sixteen_bytes_per_line() {
        let bytes = (0..18).collect::<Vec<_>>();
        assert_eq!(
            format_network_buffer(&bytes),
            "0000: 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n0010: 10 11"
        );
    }

    #[test]
    fn transmitted_value_has_no_offset_and_uses_the_last_byte() {
        assert_eq!(format_last_transmitted_value(&[0x40, 0x41]), "41");
    }

    #[test]
    fn network_status_never_exposes_socket_error_details() {
        let mut state = NetworkState {
            mode: NetworkMode::Client,
            host: "127.0.0.1".to_owned(),
            port: 5800,
            connection: ConnectionState::Error("os error 10061".to_owned()),
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
            rx_total: 0,
            tx_total: 0,
            last_error: Some("os error 10061".to_owned()),
            status: DeviceStatus::Error("os error 10061".to_owned()),
        };

        assert_eq!(network_status(&state, Lang::Ru), "Ошибка");

        state.connection = ConnectionState::Refused;
        assert_eq!(network_status(&state, Lang::Ru), "Отклонено");
    }
}
