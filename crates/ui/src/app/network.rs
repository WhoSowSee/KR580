use crate::backend::NetworkMode;

use super::DesktopApp;
use crate::i18n::{Key, NetworkKey};
use crate::settings_storage::load_settings;

impl DesktopApp {
    pub(crate) fn open_network_settings(&mut self) {
        let network = &self.snapshot.devices.network;
        self.network_mode_draft = network.mode;
        self.network_host_input = network.host.clone();
        self.network_port_input = network.port.to_string();
        self.network_settings_error = None;
        self.network_settings_open = true;
    }

    pub(crate) fn select_network_mode(&mut self, mode: NetworkMode) {
        let settings = load_settings();
        let (host, port) = match mode {
            NetworkMode::Client => (settings.network.host, settings.network.port),
            NetworkMode::Server => (settings.network.bind_host, settings.network.bind_port),
        };
        self.network_mode_draft = mode;
        self.network_host_input = host;
        self.network_port_input = port.to_string();
        self.network_settings_error = None;
    }

    pub(crate) fn apply_network_settings(&mut self) {
        let (host, port) =
            match parse_network_endpoint(&self.network_host_input, &self.network_port_input) {
                Ok(endpoint) => endpoint,
                Err(NetworkEndpointError::EmptyHost) => {
                    self.network_settings_error = Some(
                        self.lang
                            .t(Key::Network(NetworkKey::HostRequired))
                            .to_owned(),
                    );
                    return;
                }
                Err(NetworkEndpointError::InvalidPort) => {
                    self.network_settings_error = Some(
                        self.lang
                            .t(Key::Network(NetworkKey::PortInvalid))
                            .to_owned(),
                    );
                    return;
                }
            };

        self.dispatch_sync(crate::backend::AppCommand::ConfigureNetwork {
            mode: self.network_mode_draft,
            host,
            port,
        });
        self.network_settings_open = false;
        self.network_settings_error = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetworkEndpointError {
    EmptyHost,
    InvalidPort,
}

pub(crate) fn parse_network_endpoint(
    host: &str,
    port: &str,
) -> Result<(String, u16), NetworkEndpointError> {
    let host = host.trim();
    if host.is_empty() {
        return Err(NetworkEndpointError::EmptyHost);
    }
    let port = port
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or(NetworkEndpointError::InvalidPort)?;
    Ok((host.to_owned(), port))
}

#[cfg(test)]
mod tests {
    use super::{NetworkEndpointError, parse_network_endpoint};

    #[test]
    fn endpoint_validation_trims_host_and_parses_port() {
        assert_eq!(
            parse_network_endpoint(" 127.0.0.1 ", " 5800 "),
            Ok(("127.0.0.1".to_owned(), 5800))
        );
    }

    #[test]
    fn endpoint_validation_rejects_empty_host_and_invalid_port() {
        assert_eq!(
            parse_network_endpoint(" ", "5800"),
            Err(NetworkEndpointError::EmptyHost)
        );
        assert_eq!(
            parse_network_endpoint("127.0.0.1", "0"),
            Err(NetworkEndpointError::InvalidPort)
        );
        assert_eq!(
            parse_network_endpoint("127.0.0.1", "65536"),
            Err(NetworkEndpointError::InvalidPort)
        );
    }
}
