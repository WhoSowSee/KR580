//! "О программе" / "About" modal overlay.
//!
//! Centred dialog with the title at the top-left, an app-icon plate
//! next to a name + version block, a description paragraph spanning
//! the dialog width, and a pill-shaped GitHub button at the bottom.
//! Click outside or Esc dismisses it.

use iced::widget::{Space, button, column, container, image, mouse_area, opaque, row, stack, svg};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::icons;
use super::theme::{
    TOKYO_BOARD, TOKYO_BORDER, TOKYO_MUTED, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_SURFACE_3,
    TOKYO_TEXT, ui_text,
};
use crate::app::Message;
use crate::i18n::{Key, Lang};

const DIALOG_WIDTH: f32 = 388.0;
const DIALOG_PADDING: u16 = 24;
const APP_ICON_PLATE_SIZE: f32 = 64.0;
const APP_ICON_GLYPH_SIZE: f32 = 64.0;
const GITHUB_ICON_SIZE: f32 = 17.0;
/// Polar opposite of "rectangular" — large radius collapses container
/// corners to a perfect pill shape regardless of inner content size.
const PILL_RADIUS: f32 = 999.0;

pub(super) fn about_modal_overlay<'a>(lang: Lang) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseAbout);

    let title = ui_text(lang.t(Key::AboutTitle), 24, TOKYO_TEXT);

    let app_icon_glyph = image(icons::app_icon())
        .width(Length::Fixed(APP_ICON_GLYPH_SIZE))
        .height(Length::Fixed(APP_ICON_GLYPH_SIZE));
    let app_icon_plate = container(app_icon_glyph)
        .width(Length::Fixed(APP_ICON_PLATE_SIZE))
        .height(Length::Fixed(APP_ICON_PLATE_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    let app_name = ui_text(lang.t(Key::AppName), 18, TOKYO_TEXT);
    let version = ui_text(lang.t(Key::AboutVersion), 13, TOKYO_MUTED);

    let identity_row = row![
        app_icon_plate,
        column![app_name, Space::new().height(Length::Fixed(4.0)), version,]
            .align_x(alignment::Horizontal::Left),
    ]
    .spacing(16)
    .align_y(alignment::Vertical::Center);

    let description = ui_text(lang.t(Key::AboutDescription), 14, TOKYO_TEXT);

    let github_glyph = svg(icons::github())
        .width(Length::Fixed(GITHUB_ICON_SIZE))
        .height(Length::Fixed(GITHUB_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });
    let github_button = button(
        container(
            row![
                github_glyph,
                ui_text(lang.t(Key::AboutGithubLabel), 13, TOKYO_TEXT),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
        )
        .padding(iced::Padding::ZERO.left(18).right(22).top(9).bottom(9)),
    )
    .on_press(Message::OpenUrl("https://github.com/WhoSowSee/KR580"))
    .padding(0)
    .style(|_theme, status| pill_button_style(status));

    let github_row = row![github_button, Space::new().width(Length::Fill),];

    let body = container(
        column![
            title,
            Space::new().height(Length::Fixed(20.0)),
            identity_row,
            Space::new().height(Length::Fixed(20.0)),
            description,
            Space::new().height(Length::Fixed(20.0)),
            github_row,
        ]
        .width(Length::Fixed(DIALOG_WIDTH)),
    )
    .padding(DIALOG_PADDING)
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

fn modal_backdrop_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color {
            r: 0.07,
            g: 0.07,
            b: 0.13,
            a: 0.70,
        })),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..iced::widget::container::Style::default()
    }
}

fn modal_dialog_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

/// Soundly-style pill: dark fill, no border, fully rounded corners,
/// brightens on hover/press without changing shape.
fn pill_button_style(status: iced::widget::button::Status) -> iced::widget::button::Style {
    use iced::widget::button;
    let background = match status {
        button::Status::Pressed => TOKYO_SURFACE_3,
        button::Status::Hovered => TOKYO_SURFACE_2,
        _ => TOKYO_SURFACE,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: PILL_RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}
