use iced::widget::{button, container};
use iced::{Background, Border, Color};

pub(super) use super::super::styles::modal_backdrop_style;
use super::super::theme::{
    tokyo_board, tokyo_border, tokyo_surface, tokyo_surface_2, tokyo_surface_3, tokyo_text,
};

/// Dialog body uses `tokyo_board()` so it blends with the schematic plate
/// behind the backdrop instead of standing out as a light-grey panel.
pub(super) fn modal_dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(tokyo_text()),
        background: Some(Background::Color(tokyo_board())),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(super) fn header_close_button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => tokyo_surface_2(),
        button::Status::Pressed => tokyo_surface_3(),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Sidebar category – fill change only, optional 1-px border ring
/// when keyboard-focused so Ctrl+Tab → arrow keys is observable.
pub(super) fn sidebar_chip_style(
    status: button::Status,
    active: bool,
    keyboard_focused: bool,
) -> button::Style {
    let background = match (active, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.5,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => tokyo_surface(),
        _ => Color::TRANSPARENT,
    };
    let border_color = if keyboard_focused {
        tokyo_text()
    } else {
        Color::TRANSPARENT
    };
    let border_width = if keyboard_focused { 1.0 } else { 0.0 };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: border_width,
            color: border_color,
        },
        ..button::Style::default()
    }
}

/// Search-shell border that lights up only while the Search section
/// owns the keyboard focus.
pub(super) fn section_input_style(focused: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            radius: 6.0.into(),
            width: if focused { 1.0 } else { 0.0 },
            color: if focused {
                tokyo_border()
            } else {
                Color::TRANSPARENT
            },
        },
        ..container::Style::default()
    }
}

pub(super) fn segmented_button_style(
    status: button::Status,
    active: bool,
    keyboard_focused: bool,
) -> button::Style {
    let background = match (active, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.4,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => Color {
            a: 0.6,
            ..tokyo_surface()
        },
        _ => Color::TRANSPARENT,
    };
    let border_color = if keyboard_focused {
        tokyo_text()
    } else {
        tokyo_border()
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

pub(super) fn footer_button_style(status: button::Status, focused: bool) -> button::Style {
    let background = match (focused, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.4,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => Color {
            a: 0.6,
            ..tokyo_surface()
        },
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

pub(super) fn dropdown_anchor_style(
    status: button::Status,
    keyboard_focused: bool,
) -> button::Style {
    let background = match (keyboard_focused, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.4,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => Color {
            a: 0.6,
            ..tokyo_surface()
        },
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

pub(super) fn dropdown_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokyo_board())),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: tokyo_border(),
        },
        ..container::Style::default()
    }
}

pub(super) fn dropdown_option_style(
    status: button::Status,
    selected: bool,
    highlighted: bool,
) -> button::Style {
    // Selected fills with tokyo_surface(); the keyboard highlight uses
    // the same fill so arrow keys feel like a hover preview without
    // committing the value. Mouse hover on a non-selected, non-
    // highlighted row uses the muted half-alpha tint.
    let background = match (selected || highlighted, status) {
        (true, _) => tokyo_surface(),
        (false, button::Status::Hovered) => Color {
            a: 0.5,
            ..tokyo_surface()
        },
        (false, button::Status::Pressed) => tokyo_surface(),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: tokyo_text(),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}
