use iced::widget::{button, container};
use iced::{Background, Border, Color};

use super::super::theme::{
    TOKYO_BOARD, TOKYO_BORDER, TOKYO_GREEN, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT,
};

pub(super) fn modal_backdrop_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
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
        ..container::Style::default()
    }
}

pub(super) fn modal_dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn group_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn group_label_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

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

pub(super) fn icon_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.45,
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

pub(super) fn dropdown_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn dropdown_option_style(status: button::Status, highlighted: bool) -> button::Style {
    let background = match (highlighted, status) {
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
