use iced::widget::{Space, mouse_area, stack};
use iced::{Element, Length};

use super::monitor::monitor_window;
use super::network::network_window;
use super::printer::printer_window;
use super::printer_setup::{printer_properties_window_view, printer_setup_window_view};
use super::storage::{floppy_window, hdd_window};
use super::theme;
use crate::app::{DesktopApp, Message, ToolWindowKind};

const TOOL_WINDOW_DRAG_HEIGHT: f32 = 48.0;
const PRINTER_DIALOG_DRAG_HEIGHT: f32 = 52.0;

impl DesktopApp {
    pub(crate) fn view(&self, window: iced::window::Id) -> Element<'_, Message> {
        let content = self.window_view(window);
        super::widgets::device_keyboard_capture(
            content,
            self.device_keyboard_owner(window).is_some(),
        )
    }

    fn window_view(&self, window: iced::window::Id) -> Element<'_, Message> {
        theme::set_active_color_scheme(self.preferences.color_scheme);
        if self.printer_setup.printer_properties_window_id == Some(window) {
            return window_drag_surface(
                printer_properties_window_view(
                    self.printer_setup.printer_setup_dialog.as_ref(),
                    self.preferences.lang,
                ),
                window,
                PRINTER_DIALOG_DRAG_HEIGHT,
            );
        }
        if self.printer_setup.printer_setup_window_id == Some(window) {
            return window_drag_surface(
                printer_setup_window_view(
                    self.printer_setup.printer_setup_dialog.as_ref(),
                    self.preferences.lang,
                ),
                window,
                PRINTER_DIALOG_DRAG_HEIGHT,
            );
        }
        if self.panels.monitor_window.id() == Some(window) {
            if !self.panels.monitor_window.detached() {
                return Space::new().into();
            }
            return window_drag_surface(
                monitor_window(
                    &self.snapshot.devices.monitor,
                    self.panels.monitor_split,
                    self.hex_popup_view_state(),
                    self.preferences.lang,
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.panels.floppy_window.id() == Some(window) {
            if !self.panels.floppy_window.detached() {
                return Space::new().into();
            }
            return window_drag_surface(
                floppy_window(
                    &self.snapshot.devices.floppy,
                    self.panels.floppy_show_image_contents,
                    &self.panels.floppy_image_contents,
                    self.panels.floppy_image_error.as_deref(),
                    self.preferences.lang,
                    self.device_toolbar(ToolWindowKind::Floppy),
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.panels.hdd_window.id() == Some(window) {
            if !self.panels.hdd_window.detached() {
                return Space::new().into();
            }
            return window_drag_surface(
                hdd_window(
                    &self.snapshot.devices.hdd,
                    self.panels.hdd_file_exists,
                    self.panels.hdd_show_image_contents,
                    &self.panels.hdd_image_contents,
                    self.panels.hdd_image_error.as_deref(),
                    self.preferences.lang,
                    self.device_toolbar(ToolWindowKind::Hdd),
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.panels.network_window.id() == Some(window) {
            if !self.panels.network_window.detached() {
                return Space::new().into();
            }
            return window_drag_surface(
                network_window(self.network_view_state()),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.panels.printer_window.id() == Some(window) {
            if !self.panels.printer_window.detached() {
                return Space::new().into();
            }
            return window_drag_surface(
                printer_window(
                    &self.snapshot.devices.printer,
                    self.panels.printer_text_view,
                    self.printer_target_label(),
                    self.preferences.lang,
                    self.device_toolbar(ToolWindowKind::Printer),
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.shell.main_window_id != Some(window) {
            return Space::new().into();
        }
        self.main_view()
    }
}

fn window_drag_surface<'a>(
    content: Element<'a, Message>,
    window: iced::window::Id,
    height: f32,
) -> Element<'a, Message> {
    let drag_surface = mouse_area(
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(height)),
    )
    .on_press(Message::DetachedWindowDragStart(window));
    stack![drag_surface, content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use iced::advanced::{Layout, Shell, clipboard, layout, renderer::Headless, widget};
    use iced::{Event, Point, Size, mouse};

    use super::*;

    #[test]
    fn detached_monitor_top_inset_starts_window_drag() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        let window = iced::window::Id::unique();
        app.panels
            .monitor_window
            .set_native(crate::app::NativeWindow::Opening(window));
        app.panels
            .monitor_window
            .detach(app.panels.monitor_window.native().unwrap());
        let messages = press_messages(app.view(window), Point::new(530.0, 8.0));

        assert!(
            matches!(
                messages.as_slice(),
                [Message::DetachedWindowDragStart(actual)] if *actual == window
            ),
            "unexpected messages: {messages:?}"
        );
    }

    #[test]
    fn attached_monitor_preserves_base_scrollable_states() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        let window = iced::window::Id::unique();
        app.shell.main_window_id = Some(window);
        let mut tree = {
            let root = app.view(window);
            widget::Tree::new(&root)
        };
        let scrollable: Element<'_, Message> = iced::widget::scrollable(Space::new()).into();
        let tag = scrollable.as_widget().tag();
        let before = state_addresses(&tree, tag);
        assert!(!before.is_empty());

        app.panels.monitor_open = true;
        {
            let root = app.view(window);
            tree.diff(&root);
        }
        assert!(
            before
                .iter()
                .all(|address| state_addresses(&tree, tag).contains(address))
        );

        app.panels.monitor_open = false;
        {
            let root = app.view(window);
            tree.diff(&root);
        }
        assert!(
            before
                .iter()
                .all(|address| state_addresses(&tree, tag).contains(address))
        );
    }

    fn state_addresses(tree: &widget::Tree, tag: widget::tree::Tag) -> Vec<*const ()> {
        let mut addresses = Vec::new();
        collect_state_addresses(tree, tag, &mut addresses);
        addresses
    }

    fn collect_state_addresses(
        tree: &widget::Tree,
        tag: widget::tree::Tag,
        addresses: &mut Vec<*const ()>,
    ) {
        if tree.tag == tag
            && let widget::tree::State::Some(state) = &tree.state
        {
            addresses.push(state.as_ref() as *const dyn std::any::Any as *const ());
        }
        for child in &tree.children {
            collect_state_addresses(child, tag, addresses);
        }
    }

    fn press_messages(mut root: Element<'_, Message>, position: Point) -> Vec<Message> {
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("renderer runtime is available")
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .expect("software renderer is available");
        let mut tree = widget::Tree::new(&root);
        let node = root.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(1060.0, 600.0)),
        );
        let layout = Layout::new(&node);
        let viewport = layout.bounds();
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        root.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            mouse::Cursor::Available(position),
            &renderer,
            &mut clipboard::Null,
            &mut shell,
            &viewport,
        );
        messages
    }
}
