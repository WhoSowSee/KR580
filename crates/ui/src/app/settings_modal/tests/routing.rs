use iced::widget::Id;

use crate::app::constants::MEMORY_ADDRESS_INPUT_ID;
use crate::app::{DesktopApp, Message};

#[test]
fn settings_modal_blocks_background_focus_reconciliation() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    let _ = app.update(Message::OpenSettings);
    let generation = app.mouse_press_generation;

    for message in [Message::MousePressed, Message::MousePressedIgnored] {
        let _ = app.update(message);
        assert_eq!(app.mouse_press_generation, generation);
    }

    let _ = app.update(Message::FocusReconciled {
        generation,
        hit: Some(Id::new(MEMORY_ADDRESS_INPUT_ID)),
    });

    assert_eq!(app.focused_input, None);
}
