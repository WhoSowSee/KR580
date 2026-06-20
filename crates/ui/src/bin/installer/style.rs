use iced::widget::{button, checkbox, container, progress_bar, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, font, theme};

pub const FONT: Font = Font::with_name("Segoe UI Variable");
pub const FONT_BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..FONT
};

pub const BLACK: Color = Color::from_rgb8(0x05, 0x05, 0x05);
pub const PANEL: Color = Color::from_rgb8(0x10, 0x10, 0x10);
pub const SURFACE: Color = Color::from_rgb8(0x18, 0x18, 0x18);
pub const FIELD: Color = Color::from_rgb8(0x0B, 0x0B, 0x0B);
pub const LINE: Color = Color::from_rgb8(0x35, 0x35, 0x35);
pub const LINE_STRONG: Color = Color::from_rgb8(0x62, 0x62, 0x62);
pub const TEXT: Color = Color::from_rgb8(0xF4, 0xF4, 0xF4);
pub const MUTED: Color = Color::from_rgb8(0xA5, 0xA5, 0xA5);
pub const DIM: Color = Color::from_rgb8(0x70, 0x70, 0x70);
pub const WHITE: Color = Color::WHITE;

pub fn app_style(_state: &super::Installer, _theme: &Theme) -> theme::Style {
    theme::Style {
        background_color: BLACK,
        text_color: TEXT,
    }
}

pub fn titlebar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BLACK)),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn titlebar_divider(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(LINE)),
        ..container::Style::default()
    }
}

pub fn panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BLACK)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: LINE,
        },
        shadow: Shadow {
            color: Color {
                a: 0.45,
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 22.0,
        },
        ..container::Style::default()
    }
}

pub fn soft_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(FIELD)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: LINE,
        },
        ..container::Style::default()
    }
}

pub fn selected_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => SURFACE,
        button::Status::Hovered => PANEL,
        _ => FIELD,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.5,
            color: WHITE,
        },
        ..button::Style::default()
    }
}

pub fn neutral_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => SURFACE,
        button::Status::Hovered => PANEL,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: LINE_STRONG,
        },
        ..button::Style::default()
    }
}

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let disabled = matches!(status, button::Status::Disabled);
    let background = match status {
        button::Status::Pressed => Color::from_rgb8(0xD8, 0xD8, 0xD8),
        button::Status::Hovered => Color::from_rgb8(0xEA, 0xEA, 0xEA),
        button::Status::Disabled => LINE,
        _ => WHITE,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if disabled { MUTED } else { BLACK },
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: background,
        },
        ..button::Style::default()
    }
}

pub fn caption_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => Color::from_rgb8(0x26, 0x26, 0x26),
        button::Status::Hovered => SURFACE,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub fn close_caption_button(status: button::Status) -> button::Style {
    caption_button(status)
}

pub fn input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => WHITE,
        text_input::Status::Hovered => LINE_STRONG,
        _ => LINE,
    };
    text_input::Style {
        background: Background::Color(FIELD),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: MUTED,
        placeholder: DIM,
        value: TEXT,
        selection: Color { a: 0.28, ..WHITE },
    }
}

pub fn check(_theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let checked = matches!(
        status,
        checkbox::Status::Active { is_checked: true }
            | checkbox::Status::Hovered { is_checked: true }
            | checkbox::Status::Disabled { is_checked: true }
    );
    checkbox::Style {
        background: Background::Color(if checked { WHITE } else { FIELD }),
        icon_color: BLACK,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: if checked { WHITE } else { LINE_STRONG },
        },
        text_color: Some(TEXT),
    }
}

pub fn progress(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(FIELD),
        bar: Background::Color(WHITE),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: LINE,
        },
    }
}
