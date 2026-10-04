use super::{SubprogramDialogFocus, SubprogramDialogMode};
use crate::app::{DesktopApp, Message};

use crate::app::test_support::settle_backend;

#[test]
fn loaded_range_and_next_save_follow_the_bytes_read() {
    use crate::backend::{AppCommand, Emulator};

    let dir = std::env::temp_dir().join(format!("kr580-load-range-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("changing.krs");
    let (mut app, _) = DesktopApp::with_initial_path(None);
    let mut emulator = Emulator::default();
    for loaded in [4, 2] {
        let expected = vec![0x42; loaded];
        std::fs::write(&path, &expected).unwrap();
        let events = emulator.handle_command(AppCommand::LoadSubprogram {
            path: path.clone(),
            start: 0x1000,
        });
        std::fs::write(&path, [0xFF]).unwrap();
        app.dispatch_request(AppCommand::ApplyCpuState(Box::new(emulator.snapshot().cpu)));
        settle_backend(&mut app);
        for event in events {
            app.consume_event(event);
        }
        assert_eq!(
            app.document.current_subprogram_range,
            Some((0x1000, 0x1000 + loaded as u16 - 1))
        );
        assert_eq!(
            &app.snapshot.cpu.memory.as_slice()[0x1000..0x1000 + loaded],
            &expected
        );
        app.memory.memory_inline_value_input.clear();
        let _ = app.save_program();
        settle_backend(&mut app);
        assert_eq!(std::fs::read(&path).unwrap(), expected);
    }
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn subprogram_saves_preserve_unsaved_memory_and_registers() {
    use crate::backend::AppCommand;
    use k580_core::RegisterName;

    let dir = std::env::temp_dir().join(format!("kr580-partial-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("partial.krs");
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.dispatch_request(AppCommand::ApplyCpuState(Box::default()));
    settle_backend(&mut app);
    app.mark_saved();
    app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0x76));
    app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0x42));
    app.dispatch_with_undo(AppCommand::SetRegister(RegisterName::A, 0x12));
    app.open_subprogram_dialog(path.clone(), SubprogramDialogMode::Save);
    let dialog = app.document.subprogram_dialog.as_mut().unwrap();
    dialog.start_input = "0100".into();
    dialog.end_input = "0100".into();
    app.confirm_subprogram();
    settle_backend(&mut app);
    assert!(app.shell.error_notice.is_none());
    assert!(app.document.subprogram_dialog.is_none());
    assert_eq!(std::fs::read(&path).unwrap(), [0x76]);
    assert!(app.document.dirty);
    assert_eq!(app.document.saved_cpu.memory.read(0x0100), 0x76);
    assert_eq!(app.document.saved_cpu.memory.read(0x2000), 0);
    assert_eq!(app.document.saved_cpu.registers.a, 0);

    app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0));
    assert!(app.document.dirty);
    app.dispatch_with_undo(AppCommand::SetRegister(RegisterName::A, 0));
    app.dispatch_with_undo(AppCommand::SetPc(0));
    settle_backend(&mut app);
    assert!(!app.document.dirty);
    app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0xC9));
    app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0x33));
    app.memory.memory_inline_value_input.clear();
    let _ = app.save_program();
    settle_backend(&mut app);
    assert_eq!(std::fs::read(&path).unwrap(), [0xC9]);
    assert!(app.document.dirty);
    app.dispatch_with_undo(AppCommand::SetMemory(0x2000, 0));
    settle_backend(&mut app);
    assert!(!app.document.dirty);

    app.dispatch_with_undo(AppCommand::SetMemory(0x0100, 0xFF));
    let saved = app.document.saved_cpu.clone();
    app.document.current_snapshot_path = Some(dir.join("missing/partial.krs"));
    let _ = app.save_program();
    settle_backend(&mut app);
    assert!(app.shell.error_notice.is_some());
    assert!(app.document.dirty);
    assert_eq!(app.document.saved_cpu, saved);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn address_edits_accept_only_up_to_four_hex_digits() {
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.open_subprogram_dialog("unused.krs".into(), SubprogramDialogMode::Save);
    for (input, expected) in [
        ("1a2f", "1A2F"),
        ("1A2F5", "1A2F"),
        ("12g4", "1A2F"),
        ("", ""),
        ("ffff", "FFFF"),
    ] {
        let _ = app.update(Message::SubprogramStartChanged(input.into()));
        let _ = app.update(Message::SubprogramEndChanged(input.into()));
        let dialog = app.document.subprogram_dialog.as_ref().unwrap();
        assert_eq!(dialog.start_input, expected, "start: {input}");
        assert_eq!(dialog.end_input, expected, "end: {input}");
    }
}

#[test]
fn dialogs_start_on_cancel_and_enter_closes_without_loading_or_saving() {
    for mode in [SubprogramDialogMode::Open, SubprogramDialogMode::Save] {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.open_subprogram_dialog("unused.krs".into(), mode);
        let dialog = app.document.subprogram_dialog.as_ref().unwrap();
        assert_eq!(dialog.focus, SubprogramDialogFocus::Cancel);
        assert!(!dialog.keyboard_focus_visible);
        let _ = app.update(Message::EnterPressed);
        assert!(app.document.subprogram_dialog.is_none());
        assert!(app.document.current_snapshot_path.is_none());
    }
}

#[test]
fn tab_traverses_visible_controls_in_both_directions_after_mouse_focus() {
    use SubprogramDialogFocus::{Cancel, Confirm, End, Start};

    for mode in [SubprogramDialogMode::Open, SubprogramDialogMode::Save] {
        let (mut app, _) = DesktopApp::with_initial_path(None);
        app.open_subprogram_dialog("unused.krs".into(), mode);
        let _ = app.update(Message::SubprogramFocusResolved(Some(
            iced::widget::Id::new(SubprogramDialogFocus::Start.input_id().unwrap()),
        )));
        let ring: &[_] = match mode {
            SubprogramDialogMode::Open => &[Start, Cancel, Confirm],
            SubprogramDialogMode::Save => &[Start, End, Cancel, Confirm],
        };
        for backward in [false, true] {
            for step in 1..=ring.len() {
                let _ = app.update(Message::FocusCycle { backward });
                let index = if backward {
                    ring.len() - step
                } else {
                    step % ring.len()
                };
                let dialog = app.document.subprogram_dialog.as_ref().unwrap();
                assert_eq!(dialog.focus, ring[index]);
                assert!(dialog.keyboard_focus_visible);
            }
        }
    }
}
