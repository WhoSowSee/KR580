use super::styles::{dropdown_anchor_style, dropdown_option_style, dropdown_panel_style};
use crate::app::Message;
use crate::view::icons;
use crate::view::theme::{tokyo_muted, tokyo_text, ui_text};
use crate::view::widgets::{anchored_overlay, dropdown_height, row_scrollable};
use iced::widget::{Space, button, column, container, row, scrollable, svg};
use iced::{Element, Length, alignment};

const OPTION_HEIGHT: f32 = 32.0;
const MENU_GAP: f32 = 6.0;

pub(super) struct DropdownItem {
    pub(super) label: String,
    pub(super) selected: bool,
    pub(super) message: Message,
}

pub(super) struct DropdownControl {
    pub(super) max_visible_options: usize,
    pub(super) opened: bool,
    pub(super) enabled: bool,
    pub(super) focused: bool,
    pub(super) toggle: Message,
    pub(super) dismiss: Message,
    pub(super) highlighted: Option<usize>,
}

pub(super) fn control(
    label: String,
    items: Vec<DropdownItem>,
    control: DropdownControl,
) -> Element<'static, Message> {
    let chevron = svg(icons::chevron_down())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_muted()),
        });
    let anchor = button(
        container(
            row![
                ui_text(label, 13, tokyo_text()),
                Space::new().width(Length::Fill),
                chevron,
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
        )
        .padding([8, 12])
        .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .on_press_maybe((control.enabled && !items.is_empty()).then_some(control.toggle))
    .style(move |_theme, status| dropdown_anchor_style(status, control.opened, control.focused));

    let item_count = items.len();
    let options = column(items.into_iter().enumerate().map(|(index, item)| {
        let emphasized = control
            .highlighted
            .map_or(item.selected, |value| value == index);
        dropdown_option(item, emphasized)
    }))
    .spacing(0)
    .width(Length::Fill);
    let options: Element<'static, Message> = if item_count > control.max_visible_options {
        row_scrollable(
            scrollable(options)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::hidden(),
                ))
                .height(Length::Fixed(dropdown_height(
                    item_count,
                    control.max_visible_options,
                    OPTION_HEIGHT,
                ))),
            OPTION_HEIGHT,
        )
    } else {
        options.into()
    };
    let panel = container(options)
        .padding(4)
        .width(Length::Fill)
        .style(dropdown_panel_style);

    anchored_overlay(
        anchor,
        panel,
        control.opened && item_count > 0,
        MENU_GAP,
        control.dismiss,
    )
}

fn dropdown_option(item: DropdownItem, emphasized: bool) -> Element<'static, Message> {
    button(
        container(ui_text(item.label, 13, tokyo_text()))
            .padding([6, 10])
            .width(Length::Fill)
            .height(Length::Fixed(OPTION_HEIGHT))
            .align_y(alignment::Vertical::Center),
    )
    .on_press(item.message)
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| dropdown_option_style(status, emphasized))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{Layout, Shell, clipboard, layout, renderer::Headless, widget};
    use iced::{Event, Point, Rectangle, Size, Vector, mouse};

    #[test]
    fn printer_wheel_step_reveals_the_next_full_option() {
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .unwrap();
        let mut root = control(
            "Printer 0".to_owned(),
            (0..10)
                .map(|index| DropdownItem {
                    label: format!("Printer {index}"),
                    selected: index == 0,
                    message: Message::PrinterSetupSelected(format!("Printer {index}")),
                })
                .collect(),
            DropdownControl {
                max_visible_options: 4,
                opened: true,
                enabled: true,
                focused: false,
                toggle: Message::PrinterSetupDropdownToggled(
                    crate::app::PrinterSetupDropdown::Printer,
                ),
                dismiss: Message::PrinterSetupDropdownDismissed(
                    crate::app::PrinterSetupDropdown::Printer,
                ),
                highlighted: None,
            },
        );
        let mut tree = widget::Tree::new(&root);
        let viewport = Rectangle::with_size(Size::new(420.0, 400.0));
        let anchor = root.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let mut popup = root
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&anchor),
                &renderer,
                &viewport,
                Vector::ZERO,
            )
            .unwrap();
        let node = popup.as_overlay_mut().layout(&renderer, viewport.size());
        let cursor = mouse::Cursor::Available(Point::new(
            20.0,
            anchor.size().height + MENU_GAP + 4.0 + OPTION_HEIGHT / 2.0,
        ));
        let mut messages = Vec::new();
        for event in [
            mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
            },
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            popup.as_overlay_mut().update(
                &Event::Mouse(event),
                Layout::new(&node),
                cursor,
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
            );
        }
        assert!(
            matches!(messages.as_slice(), [Message::PrinterSetupSelected(name)] if name == "Printer 1"),
            "unexpected selection: {messages:?}"
        );
    }

    #[test]
    fn popup_respects_the_row_limit_and_window_edges() {
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .unwrap();
        for max_visible_options in [4, 5, 6] {
            for count in [1, 4, 9] {
                for size in [Size::new(420.0, 400.0), Size::new(240.0, 190.0)] {
                    for y in [0.0, size.height / 2.0, size.height - 40.0] {
                        let mut root = control(
                            "Option 0".to_owned(),
                            (0..count)
                                .map(|index| DropdownItem {
                                    label: format!("Option {index}"),
                                    selected: index == 0,
                                    message: Message::PrinterSetupSelected(format!(
                                        "Option {index}"
                                    )),
                                })
                                .collect(),
                            DropdownControl {
                                max_visible_options,
                                opened: true,
                                enabled: true,
                                focused: false,
                                toggle: Message::PrinterSetupDropdownToggled(
                                    crate::app::PrinterSetupDropdown::Printer,
                                ),
                                dismiss: Message::PrinterSetupDropdownDismissed(
                                    crate::app::PrinterSetupDropdown::Printer,
                                ),
                                highlighted: None,
                            },
                        );
                        let mut tree = widget::Tree::new(&root);
                        let viewport = Rectangle::with_size(size);
                        let anchor = root
                            .as_widget_mut()
                            .layout(&mut tree, &renderer, &layout::Limits::new(Size::ZERO, size))
                            .move_to(Point::new(20.0, y));
                        let mut popup = root
                            .as_widget_mut()
                            .overlay(
                                &mut tree,
                                Layout::new(&anchor),
                                &renderer,
                                &viewport,
                                Vector::ZERO,
                            )
                            .unwrap();
                        let node = popup.as_overlay_mut().layout(&renderer, size);
                        let panel = Layout::new(&node).children().next().unwrap().bounds();
                        let below = (size.height - y - anchor.size().height - MENU_GAP).max(0.0);
                        let above = (y - MENU_GAP).max(0.0);
                        let requested = count.min(max_visible_options) as f32 * OPTION_HEIGHT
                            + if count > max_visible_options {
                                16.0
                            } else {
                                0.0
                            }
                            + 8.0;
                        let expected = requested.min(below.max(above));
                        assert!((panel.height - expected).abs() < 0.01, "{panel:?}");
                        assert!(panel.x >= 0.0 && panel.x + panel.width <= size.width);
                        assert!(panel.y >= 0.0 && panel.y + panel.height <= size.height);
                        if requested > below && above > below {
                            assert!(panel.y + panel.height <= y - MENU_GAP);
                        }
                    }
                }
            }
        }
    }
}
