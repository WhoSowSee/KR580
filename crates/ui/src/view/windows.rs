use iced::widget::{Space, mouse_area, stack};
use iced::{Element, Length};

use super::monitor::monitor_window;
use super::network::network_window;
use super::printer::printer_window;
use super::printer_setup::{printer_properties_window_view, printer_setup_window_view};
use super::storage::{floppy_window, hdd_window};
use super::theme;
use crate::app::{DesktopApp, Message};

const TOOL_WINDOW_DRAG_HEIGHT: f32 = 48.0;
const PRINTER_DIALOG_DRAG_HEIGHT: f32 = 52.0;

impl DesktopApp {
    pub(crate) fn view(&self, window: iced::window::Id) -> Element<'_, Message> {
        theme::set_active_color_scheme(self.color_scheme);
        if self.printer_properties_window_id == Some(window) {
            return window_drag_surface(
                printer_properties_window_view(self.printer_setup_dialog.as_ref(), self.lang),
                window,
                PRINTER_DIALOG_DRAG_HEIGHT,
            );
        }
        if self.printer_setup_window_id == Some(window) {
            return window_drag_surface(
                printer_setup_window_view(self.printer_setup_dialog.as_ref(), self.lang),
                window,
                PRINTER_DIALOG_DRAG_HEIGHT,
            );
        }
        if self.monitor_window.id == Some(window) {
            if !self.monitor_window.detached {
                return Space::new().into();
            }
            return window_drag_surface(
                monitor_window(
                    &self.snapshot.devices.monitor,
                    self.monitor_split,
                    self.monitor_hex_popup,
                    self.monitor_hex_filter,
                    self.monitor_hex_scroll_visible_ticks > 0,
                    self.monitor_window.always_on_top,
                    self.lang,
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.floppy_window.id == Some(window) {
            if !self.floppy_window.detached {
                return Space::new().into();
            }
            return window_drag_surface(
                floppy_window(
                    &self.snapshot.devices.floppy,
                    self.floppy_show_image_contents,
                    &self.floppy_image_contents,
                    self.floppy_image_error.as_deref(),
                    self.floppy_window.always_on_top,
                    self.lang,
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.hdd_window.id == Some(window) {
            if !self.hdd_window.detached {
                return Space::new().into();
            }
            return window_drag_surface(
                hdd_window(
                    &self.snapshot.devices.hdd,
                    self.hdd_file_exists,
                    self.hdd_show_image_contents,
                    &self.hdd_image_contents,
                    self.hdd_image_error.as_deref(),
                    self.hdd_window.always_on_top,
                    self.lang,
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.network_window.id == Some(window) {
            if !self.network_window.detached {
                return Space::new().into();
            }
            return window_drag_surface(
                network_window(self.network_view_state(), self.network_window.always_on_top),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.printer_window.id == Some(window) {
            if !self.printer_window.detached {
                return Space::new().into();
            }
            return window_drag_surface(
                printer_window(
                    &self.snapshot.devices.printer,
                    self.printer_text_view,
                    self.printer_target_label(),
                    self.printer_window.always_on_top,
                    self.lang,
                ),
                window,
                TOOL_WINDOW_DRAG_HEIGHT,
            );
        }
        if self.main_window_id != Some(window) {
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
        app.monitor_window.id = Some(window);
        app.monitor_window.detached = true;
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
        app.main_window_id = Some(window);
        let mut tree = {
            let root = app.view(window);
            widget::Tree::new(&root)
        };
        let scrollable: Element<'_, Message> = iced::widget::scrollable(Space::new()).into();
        let tag = scrollable.as_widget().tag();
        let before = state_addresses(&tree, tag);
        assert!(!before.is_empty());

        app.monitor_open = true;
        {
            let root = app.view(window);
            tree.diff(&root);
        }
        assert!(
            before
                .iter()
                .all(|address| state_addresses(&tree, tag).contains(address))
        );

        app.monitor_open = false;
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
