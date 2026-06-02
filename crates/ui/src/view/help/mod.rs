mod consts;
mod content;
mod sidebar;
mod styles;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use consts::{DIALOG_HEIGHT, DIALOG_WIDTH, HEADER_HEIGHT, SIDEBAR_WIDTH};
use content::help_content;
use sidebar::help_sidebar;
use styles::{modal_backdrop_style, modal_dialog_style, separator_horizontal, separator_vertical};

use crate::app::{HelpDialog, Message};
use crate::i18n::Lang;
use crate::view::theme::{TOKYO_TEXT, ui_text};

pub(super) fn help_modal_overlay<'a>(dialog: &'a HelpDialog, lang: Lang) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseHelp);

    let header = container(
        container(ui_text(
            lang.t(crate::i18n::Key::HelpDialogTitle),
            21,
            TOKYO_TEXT,
        ))
        .padding([0, 20])
        .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .align_y(iced::alignment::Vertical::Center);

    let body = container(
        column![
            header,
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_theme| separator_horizontal()),
            row![
                container(help_sidebar(dialog, lang))
                    .width(Length::Fixed(SIDEBAR_WIDTH))
                    .height(Length::Fill),
                container(Space::new())
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(|_theme| separator_vertical()),
                help_content(dialog, lang),
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
