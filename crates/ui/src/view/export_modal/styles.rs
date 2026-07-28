use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme};

pub(super) use super::super::styles::{
    inset_style as dropdown_panel_style, inset_style as group_panel_style,
    legend_label_style as group_label_style, modal_backdrop_style,
    modal_dropdown_option_style as dropdown_option_style, modal_tab_button_style,
    panel_style as modal_dialog_style,
};
use super::super::theme::{tokyo_border, tokyo_green, tokyo_surface, tokyo_surface_2, tokyo_text};

pub(super) fn combo_arrow_style(_status: button::Status, _open: bool) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: tokyo_text(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn keyboard_input_shell_style(theme: &Theme, focused: bool) -> container::Style {
    let mut style = super::super::styles::input_shell_style(theme, focused);
    if focused {
        style.border.color = tokyo_text();
    }
    style
}

pub(super) fn checklist_button_style(status: button::Status, focused: bool) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.32,
            ..tokyo_surface()
        },
        button::Status::Pressed => Color {
            a: 0.45,
            ..tokyo_surface()
        },
        _ => Color::TRANSPARENT,
    };

    let mut style = button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    };
    if focused {
        style.border.color = tokyo_text();
        style.border.width = 1.0;
    }
    style
}

pub(super) fn tab_button_style(
    status: button::Status,
    active: bool,
    focused: bool,
) -> button::Style {
    let mut style = modal_tab_button_style(status, active);
    if focused {
        style.border.color = tokyo_text();
    }
    style
}

pub(super) fn checkbox_style(checked: bool) -> container::Style {
    let border_color = if checked {
        tokyo_green()
    } else {
        tokyo_border()
    };
    container::Style {
        text_color: Some(tokyo_text()),
        background: None,
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
            ..tokyo_surface()
        },
        button::Status::Pressed => tokyo_surface_2(),
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..button::Style::default()
    }
}
