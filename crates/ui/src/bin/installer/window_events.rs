pub(super) fn close_request(event: iced::Event) -> bool {
    matches!(
        event,
        iced::Event::Window(iced::window::Event::CloseRequested)
    )
}

#[cfg(test)]
mod tests {
    use super::close_request;
    use iced::{Event, Size, window};

    #[test]
    fn close_request_accepts_os_window_close_event() {
        assert!(close_request(Event::Window(window::Event::CloseRequested)));
    }

    #[test]
    fn close_request_ignores_other_window_events() {
        assert!(!close_request(Event::Window(window::Event::Resized(
            Size::new(640.0, 480.0)
        ))));
    }
}
