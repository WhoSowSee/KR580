use std::path::{Path, PathBuf};

use iced::{Event, Task, window};

use super::{DesktopApp, Message, PendingAction};
use crate::{
    i18n::Key,
    persistence::{ProgramSerializer, SubprogramSerializer},
    platform,
};

impl DesktopApp {
    pub(crate) fn handle_file_drag_event(
        &mut self,
        event: &Event,
        window: window::Id,
    ) -> Option<Task<Message>> {
        if self.main_window_id != Some(window) {
            return None;
        }
        if self.handle_import_file_drag_event(event) {
            return Some(Task::none());
        }
        match event {
            Event::Window(window::Event::FileHovered(_)) => {
                self.file_drag_hovered = true;
                Some(
                    window::run(window, platform::cursor_position_in_window)
                        .map(Message::FileDragCursorPosition),
                )
            }
            Event::Window(window::Event::FileDropped(path)) => {
                self.file_drag_hovered = false;
                self.file_drag_cursor_position = None;
                self.open_dropped_file(path.clone());
                Some(Task::none())
            }
            Event::Window(window::Event::FilesHoveredLeft) => {
                self.file_drag_hovered = false;
                self.file_drag_cursor_position = None;
                Some(Task::none())
            }
            _ => None,
        }
    }

    pub(crate) fn update_file_drag_cursor(&mut self, position: Option<iced::Point>) {
        if self.file_drag_hovered {
            self.file_drag_cursor_position = position;
        }
    }

    fn open_dropped_file(&mut self, path: PathBuf) {
        if !supports_dropped_program(&path) {
            self.show_error_notice(format!(
                "{}: {}",
                self.lang.t(Key::ErrorPrefix),
                self.lang.t(Key::ErrUnsupportedDroppedFile)
            ));
            return;
        }
        self.clear_error_notice();
        if self.dirty {
            self.open_discard_modal(PendingAction::OpenDroppedFile(path));
        } else {
            self.load_program_from_path(path);
        }
    }
}

fn supports_dropped_program(path: &Path) -> bool {
    ProgramSerializer::supports_path(path) || SubprogramSerializer::supports_path(path)
}

#[cfg(test)]
mod tests {
    use super::{DesktopApp, Event, Key, PendingAction, supports_dropped_program, window};
    use std::path::{Path, PathBuf};

    #[test]
    fn accepts_only_supported_program_extensions_case_insensitively() {
        assert!(supports_dropped_program(Path::new("program.580")));
        assert!(supports_dropped_program(Path::new("program.KRS")));
        assert!(!supports_dropped_program(Path::new("program.txt")));
        assert!(!supports_dropped_program(Path::new("program")));
    }

    #[test]
    fn hover_state_tracks_only_the_main_window() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        let main = window::Id::unique();
        let detached = window::Id::unique();
        app.main_window_id = Some(main);

        let hovered = Event::Window(window::Event::FileHovered(PathBuf::from("program.580")));
        let _task = app.handle_file_drag_event(&hovered, detached);
        assert!(!app.file_drag_hovered);

        let _task = app.handle_file_drag_event(&hovered, main);
        assert!(app.file_drag_hovered);
        assert!(app.file_drag_cursor_position.is_none());

        let left = Event::Window(window::Event::FilesHoveredLeft);
        let _task = app.handle_file_drag_event(&left, main);
        assert!(!app.file_drag_hovered);
        assert!(app.file_drag_cursor_position.is_none());
    }

    #[test]
    fn unsupported_drop_shows_notice_without_entering_dirty_gate() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        let main = window::Id::unique();
        app.main_window_id = Some(main);
        app.dirty = true;
        app.file_drag_hovered = true;

        let dropped = Event::Window(window::Event::FileDropped(PathBuf::from("program.txt")));
        let _task = app.handle_file_drag_event(&dropped, main);

        assert!(!app.file_drag_hovered);
        assert!(app.pending_action.is_none());
        assert_eq!(
            app.error_notice.as_deref(),
            Some(
                format!(
                    "{}: {}",
                    app.lang.t(Key::ErrorPrefix),
                    app.lang.t(Key::ErrUnsupportedDroppedFile)
                )
                .as_str()
            )
        );
        assert!(app.error_notice_dismiss_at.is_some());
    }

    #[test]
    fn dirty_supported_drop_preserves_path_until_confirmation() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        let main = window::Id::unique();
        let path = PathBuf::from("program.krs");
        app.main_window_id = Some(main);
        app.dirty = true;

        let dropped = Event::Window(window::Event::FileDropped(path.clone()));
        let _task = app.handle_file_drag_event(&dropped, main);

        assert!(matches!(
            app.pending_action.as_ref(),
            Some(PendingAction::OpenDroppedFile(pending)) if pending == &path
        ));

        let _task = app.confirm_discard();
        assert!(app.pending_action.is_none());
        assert!(app.dirty);
        assert_eq!(
            app.subprogram_dialog.as_ref().map(|dialog| &dialog.path),
            Some(&path)
        );
    }
}
