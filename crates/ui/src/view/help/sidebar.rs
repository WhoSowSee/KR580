use iced::widget::{button, column, container, row, scrollable, svg, text, text_input};
use iced::{Color, Element, Length, alignment};

use super::styles::{help_search_input_style, hidden_scrollbar_style, sidebar_chip_style};
use crate::app::{HelpDialog, HelpNode, Message};
use crate::i18n::Lang;
use crate::view::icons;
use crate::view::theme::{TOKYO_MUTED, TOKYO_SURFACE, TOKYO_TEXT, ui_text};

const CHEVRON_SIZE: f32 = 14.0;
const SEARCH_ICON_SIZE: f32 = 13.0;
const TOGGLE_ICON_SIZE: f32 = 15.0;

pub(super) fn help_sidebar<'a>(dialog: &'a HelpDialog, lang: Lang) -> Element<'a, Message> {
    let search_icon = svg(icons::search())
        .width(Length::Fixed(SEARCH_ICON_SIZE))
        .height(Length::Fixed(SEARCH_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_MUTED),
        });

    let search_field = text_input(
        lang.t(crate::i18n::Key::HelpSearchPlaceholder),
        &dialog.search,
    )
    .on_input(Message::HelpSearchChanged)
    .padding(0)
    .size(14)
    .style(help_search_input_style);

    let all_expanded = dialog.all_expanded();
    let toggle_icon = if all_expanded {
        icons::collapse_all()
    } else {
        icons::expand_all()
    };
    let toggle_btn = button(
        svg(toggle_icon)
            .width(Length::Fixed(TOGGLE_ICON_SIZE))
            .height(Length::Fixed(TOGGLE_ICON_SIZE))
            .style(|_theme, _status| svg::Style {
                color: Some(TOKYO_MUTED),
            }),
    )
    .on_press(Message::HelpToggleExpandAll)
    .padding(0)
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => TOKYO_SURFACE,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..button::Style::default()
        }
    });

    let search_row = row![search_icon, search_field.width(Length::Fill), toggle_btn]
        .spacing(6)
        .align_y(alignment::Vertical::Center);

    let mut items: Vec<Element<'a, Message>> = Vec::new();
    let searching = !dialog.results_query().is_empty();
    build_tree(&mut items, &HelpNode::ROOTS, dialog, lang, 0, searching);

    let list = column(items).spacing(2).width(Length::Fill);

    column![
        container(search_row).padding(iced::Padding::new(10.0).right(8.0).bottom(6.0).left(12.0)),
        scrollable(
            container(list).padding(iced::Padding::new(4.0).right(8.0).bottom(12.0).left(8.0)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_tree<'a>(
    items: &mut Vec<Element<'a, Message>>,
    nodes: &[HelpNode],
    dialog: &HelpDialog,
    lang: Lang,
    depth: u8,
    searching: bool,
) {
    for &node in nodes {
        let is_category = node.is_category();
        let expanded = dialog.expanded.contains(&node);
        let selected = !is_category && dialog.selected == node;
        let label_key = node.label_key();
        let label = lang.t(label_key);
        let indent = (depth as u16) * 14;

        if searching && !dialog.node_matches_search(node, lang) {
            continue;
        }

        let arrow: Element<'a, Message> = if is_category {
            let icon = if expanded {
                icons::chevron_down()
            } else {
                icons::chevron_right()
            };
            svg(icon)
                .width(Length::Fixed(CHEVRON_SIZE))
                .height(Length::Fixed(CHEVRON_SIZE))
                .style(|_theme, _status| svg::Style {
                    color: Some(TOKYO_MUTED),
                })
                .into()
        } else {
            text("")
                .size(10)
                .color_maybe(Some(Color::TRANSPARENT))
                .width(Length::Fixed(CHEVRON_SIZE))
                .into()
        };

        let row_content = row![
            text("").width(Length::Fixed(indent as f32)),
            arrow,
            ui_text(label, 14, TOKYO_TEXT),
        ]
        .spacing(2)
        .align_y(alignment::Vertical::Center);

        let message = if is_category {
            Message::HelpNodeToggled(node)
        } else {
            Message::HelpNodeSelected(node)
        };

        items.push(
            button(container(row_content).padding([4, 6]).width(Length::Fill))
                .on_press(message)
                .padding(0)
                .width(Length::Fill)
                .style(move |_theme, status| sidebar_chip_style(status, selected, false))
                .into(),
        );

        if is_category && expanded {
            build_tree(items, node.children(), dialog, lang, depth + 1, searching);
        }
    }
}
