//! "О программе" / "About" modal overlay.
//!
//! Centred dialog with the title at the top-left, an app-icon plate
//! next to a name + version block, a description paragraph spanning
//! the dialog width, and a pill-shaped GitHub button at the bottom.
//! Click outside or Esc dismisses it.

use iced::widget::{Space, button, column, container, image, mouse_area, opaque, row, stack, svg};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::icons;
use super::styles::{large_dialog_style as modal_dialog_style, modal_backdrop_style};
use super::theme::{
    tokyo_muted, tokyo_surface, tokyo_surface_2, tokyo_surface_3, tokyo_text, ui_text,
};
use crate::app::Message;
use crate::i18n::{Key, Lang};

const DIALOG_WIDTH: f32 = 388.0;
const DIALOG_PADDING: u16 = 24;
const APP_ICON_PLATE_SIZE: f32 = 64.0;
const APP_ICON_GLYPH_SIZE: f32 = 64.0;
const GITHUB_ICON_SIZE: f32 = 17.0;
/// Polar opposite of "rectangular" – large radius collapses container
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

    let title = ui_text(lang.t(Key::AboutTitle), 24, tokyo_text());

    let app_icon_glyph = image(icons::app_icon())
        .width(Length::Fixed(APP_ICON_GLYPH_SIZE))
        .height(Length::Fixed(APP_ICON_GLYPH_SIZE));
    let app_icon_plate = container(app_icon_glyph)
        .width(Length::Fixed(APP_ICON_PLATE_SIZE))
        .height(Length::Fixed(APP_ICON_PLATE_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    let app_name = ui_text(lang.t(Key::AppName), 18, tokyo_text());
    let version = ui_text(
        format!(
            "{} {}",
            lang.t(Key::AboutVersion),
            env!("CARGO_PKG_VERSION")
        ),
        13,
        tokyo_muted(),
    );

    let identity_row = row![
        app_icon_plate,
        column![app_name, Space::new().height(Length::Fixed(4.0)), version,]
            .align_x(alignment::Horizontal::Left),
    ]
    .spacing(16)
    .align_y(alignment::Vertical::Center);

    let description = ui_text(lang.t(Key::AboutDescription), 14, tokyo_text());

    let github_glyph = svg(icons::github())
        .width(Length::Fixed(GITHUB_ICON_SIZE))
        .height(Length::Fixed(GITHUB_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });
    let github_button = button(
        container(
            row![
                github_glyph,
                ui_text(lang.t(Key::AboutGithubLabel), 13, tokyo_text()),
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

/// Soundly-style pill: dark fill, no border, fully rounded corners,
/// brightens on hover/press without changing shape.
fn pill_button_style(status: iced::widget::button::Status) -> iced::widget::button::Style {
    use iced::widget::button;
    let background = match status {
        button::Status::Pressed => tokyo_surface_3(),
        button::Status::Hovered => tokyo_surface_2(),
        _ => tokyo_surface(),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: PILL_RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}
