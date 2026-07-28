use iced::widget::{
    Space, button, column, container, mouse_area, opaque, row, scrollable, stack, svg, text_editor,
};
use iced::{Element, Font, Length, alignment};

use super::help::styles::{
    help_text_editor_style, hidden_scrollbar_style, modal_backdrop_style, modal_dialog_style,
    separator_horizontal, separator_vertical, sidebar_chip_style,
};
use super::icons;
use super::theme::{UI_BOLD_FONT, UI_FONT, tokyo_muted, tokyo_text, ui_text};
use crate::app::{ChangelogDialog, HelpMarkdownHighlight, HelpMarkdownHighlighter, Message};
use crate::i18n::{Key, Lang};

const DIALOG_WIDTH: f32 = 860.0;
const DIALOG_HEIGHT: f32 = 560.0;
const SIDEBAR_WIDTH: f32 = 250.0;
const HEADER_HEIGHT: f32 = 52.0;
const CONTENT_PADDING: f32 = 24.0;
const HEADER_ICON_SIZE: f32 = 20.0;
const HEADER_ICON_TOP_PADDING: f32 = 3.0;
const ARTICLE_EDITOR_HEIGHT: Length = Length::Shrink;

pub(super) fn changelog_modal_overlay<'a>(
    dialog: &'a ChangelogDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseChangelog);

    let header_icon_glyph = svg(icons::changelog())
        .width(Length::Fixed(HEADER_ICON_SIZE))
        .height(Length::Fixed(HEADER_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });
    let header_icon =
        container(header_icon_glyph).padding(iced::Padding::ZERO.top(HEADER_ICON_TOP_PADDING));
    let header = container(
        row![
            header_icon,
            ui_text(lang.t(Key::ChangelogTitle), 21, tokyo_text())
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
    )
    .padding([0, 20])
    .width(Length::Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .align_y(alignment::Vertical::Center);

    let body = container(
        column![
            header,
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_theme| separator_horizontal()),
            row![
                container(changelog_sidebar(dialog, lang))
                    .width(Length::Fixed(SIDEBAR_WIDTH))
                    .height(Length::Fill),
                container(Space::new())
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(|_theme| separator_vertical()),
                changelog_content(dialog),
            ]
            .height(Length::Fill),
        ]
        .width(Length::Fixed(DIALOG_WIDTH))
        .height(Length::Fixed(DIALOG_HEIGHT)),
    )
    .style(modal_dialog_style);

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn changelog_sidebar<'a>(dialog: &'a ChangelogDialog, lang: Lang) -> Element<'a, Message> {
    let all_selected = dialog.selected == 0;
    let all_releases = button(
        container(ui_text(lang.t(Key::ChangelogAllVersions), 14, tokyo_text()))
            .padding([9, 10])
            .width(Length::Fill),
    )
    .on_press(Message::ChangelogReleaseSelected(0))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| sidebar_chip_style(status, all_selected, false));

    let mut items = column![all_releases].spacing(2).width(Length::Fill);
    for (index, release) in dialog.releases.iter().enumerate() {
        let release_index = index + 1;
        let selected = dialog.selected == release_index;
        let release_button = button(
            container(
                column![
                    ui_text(&release.version, 14, tokyo_text()),
                    ui_text(&release.date, 12, tokyo_muted()),
                ]
                .spacing(2),
            )
            .padding([7, 10])
            .width(Length::Fill),
        )
        .on_press(Message::ChangelogReleaseSelected(release_index))
        .padding(0)
        .width(Length::Fill)
        .style(move |_theme, status| sidebar_chip_style(status, selected, false));
        items = items.push(release_button);
    }

    scrollable(container(items).padding([8, 8]))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(hidden_scrollbar_style)
        .into()
}

fn changelog_content<'a>(dialog: &'a ChangelogDialog) -> Element<'a, Message> {
    let body = text_editor(&dialog.article_content)
        .highlight_with::<HelpMarkdownHighlighter>(
            dialog.article_highlights.clone(),
            format_changelog_highlight,
        )
        .on_action(Message::ChangelogTextAction)
        .font(UI_FONT)
        .padding(CONTENT_PADDING)
        .size(14.0)
        .height(ARTICLE_EDITOR_HEIGHT)
        .style(help_text_editor_style);

    container(
        scrollable(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(hidden_scrollbar_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn format_changelog_highlight(
    highlight: &HelpMarkdownHighlight,
    _theme: &iced::Theme,
) -> iced::advanced::text::highlighter::Format<Font> {
    match highlight {
        HelpMarkdownHighlight::Bold => iced::advanced::text::highlighter::Format {
            color: None,
            font: Some(UI_BOLD_FONT),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_editor_yields_wheel_scrolling_to_parent() {
        assert_eq!(ARTICLE_EDITOR_HEIGHT, Length::Shrink);
    }
}
