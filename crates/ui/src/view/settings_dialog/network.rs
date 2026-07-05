use iced::widget::{Space, column, container, row};
use iced::{Element, Length, alignment};

use super::super::theme::{tokyo_muted, tokyo_red, ui_text};
use super::super::widgets::compact_text_input_shell;
use super::setting_row::setting_row;
use crate::app::{Message, SettingsDialog};
use crate::i18n::{Key, Lang, NetworkKey};

const NETWORK_LABEL_WIDTH: f32 = 42.0;
const NETWORK_HOST_WIDTH: f32 = 126.0;
const NETWORK_PORT_WIDTH: f32 = 68.0;

pub(super) fn network_defaults_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let client = endpoint_row(
        lang.t(Key::Network(NetworkKey::ModeClient)),
        &dialog.draft_network_client_host,
        &dialog.draft_network_client_port,
        Message::SettingsNetworkClientHostChanged,
        Message::SettingsNetworkClientPortChanged,
    );
    let server = endpoint_row(
        lang.t(Key::Network(NetworkKey::ModeServer)),
        &dialog.draft_network_server_host,
        &dialog.draft_network_server_port,
        Message::SettingsNetworkServerHostChanged,
        Message::SettingsNetworkServerPortChanged,
    );
    let error: Element<'a, Message> = dialog.network_error.as_deref().map_or_else(
        || Space::new().height(0).into(),
        |error| ui_text(error, 10, tokyo_red()).into(),
    );

    setting_row(
        lang.t(Key::Network(NetworkKey::GeneralSettingsLabel)),
        lang.t(Key::Network(NetworkKey::GeneralSettingsHint)),
        column![client, server, error].spacing(4).into(),
    )
}

fn endpoint_row<'a>(
    label: &'static str,
    host: &'a str,
    port: &'a str,
    host_message: fn(String) -> Message,
    port_message: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        container(ui_text(label, 11, tokyo_muted()))
            .width(Length::Fixed(NETWORK_LABEL_WIDTH))
            .align_x(alignment::Horizontal::Right),
        compact_text_input_shell(
            "127.0.0.1",
            host,
            host_message,
            Length::Fixed(NETWORK_HOST_WIDTH),
        ),
        compact_text_input_shell(
            "5800",
            port,
            port_message,
            Length::Fixed(NETWORK_PORT_WIDTH),
        ),
    ]
    .spacing(6)
    .align_y(alignment::Vertical::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{NETWORK_HOST_WIDTH, NETWORK_LABEL_WIDTH, NETWORK_PORT_WIDTH};

    #[test]
    fn network_endpoint_controls_use_compact_button_scale() {
        let label_width = std::hint::black_box(NETWORK_LABEL_WIDTH);
        let host_width = std::hint::black_box(NETWORK_HOST_WIDTH);
        let port_width = std::hint::black_box(NETWORK_PORT_WIDTH);

        assert!(host_width <= 126.0);
        assert!(port_width <= 68.0);
        assert!(label_width + host_width + port_width <= 242.0);
    }
}
