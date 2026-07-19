use super::styles::{dropdown_anchor_style, dropdown_option_style, dropdown_panel_style};
use crate::app::Message;
use crate::view::icons;
use crate::view::theme::{tokyo_muted, tokyo_text, ui_text};
use crate::view::widgets::anchored_overlay;
use iced::widget::{Space, button, column, container, row, scrollable, svg};
use iced::{Element, Length, alignment};

const OPTION_HEIGHT: f32 = 32.0;
const MENU_MAX_HEIGHT: f32 = 192.0;
const MENU_GAP: f32 = 6.0;

pub(super) struct DropdownItem {
    pub(super) label: String,
    pub(super) selected: bool,
    pub(super) message: Message,
}

pub(super) struct DropdownControl {
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
    let options = column(
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let emphasized = control
                    .highlighted
                    .map_or(item.selected, |value| value == index);
                dropdown_option(item, emphasized)
            })
            .collect::<Vec<_>>(),
    )
    .spacing(0)
    .width(Length::Fill);
    let options = scrollable(options)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .height(Length::Fixed(
            (item_count as f32 * OPTION_HEIGHT).min(MENU_MAX_HEIGHT),
        ));
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
