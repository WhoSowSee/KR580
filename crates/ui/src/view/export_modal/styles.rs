use iced::widget::{button, container};
use iced::{Background, Border, Color};

pub(super) use super::super::styles::{
    inset_style as dropdown_panel_style, inset_style as group_panel_style,
    legend_label_style as group_label_style, modal_backdrop_style,
    modal_dropdown_option_style as dropdown_option_style, panel_style as modal_dialog_style,
};
use super::super::theme::{TOKYO_BORDER, TOKYO_GREEN, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT};

pub(super) fn tab_button_style(status: button::Status, active: bool) -> button::Style {
    let background = match (active, status) {
        (true, _) => TOKYO_SURFACE,
        (false, button::Status::Hovered) => Color {
            a: 0.45,
            ..TOKYO_SURFACE
        },
        (false, button::Status::Pressed) => TOKYO_SURFACE_2,
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}

pub(super) fn combo_arrow_style(_status: button::Status, _open: bool) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn checklist_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.32,
            ..TOKYO_SURFACE
        },
        button::Status::Pressed => Color {
            a: 0.45,
            ..TOKYO_SURFACE
        },
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn checkbox_style(checked: bool) -> container::Style {
    let border_color = if checked { TOKYO_GREEN } else { TOKYO_BORDER };
    let background = if checked {
        Some(Background::Color(Color {
            a: 0.18,
            ..TOKYO_GREEN
        }))
    } else {
        None
    };

    container::Style {
        text_color: Some(TOKYO_TEXT),
        background,
        border: Border {
            radius: 3.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..container::Style::default()
    }
}

pub(super) fn flag_checkbox_style(checked: bool) -> container::Style {
    let mut style = checkbox_style(checked);
    style.border.radius = 8.0.into();
    style
}

pub(super) fn footer_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.5,
            ..TOKYO_SURFACE
        },
        button::Status::Pressed => TOKYO_SURFACE_2,
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..button::Style::default()
    }
}
