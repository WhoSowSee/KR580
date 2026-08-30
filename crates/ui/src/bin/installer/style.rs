use iced::widget::{button, checkbox, container, progress_bar, text_input};
use iced::{Background, Border, Color, Font, Theme, font, theme};

pub(super) const FONT: Font = Font::with_name("Segoe UI Variable");
pub(super) const FONT_BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..FONT
};
pub(super) const MONO_FONT: Font = Font::MONOSPACE;
pub(super) const CHECKBOX_SIZE: f32 = 14.0;

pub(super) const BLACK: Color = Color::from_rgb8(0x12, 0x13, 0x20);
pub(super) const PANEL: Color = Color::from_rgb8(0x1D, 0x20, 0x30);
pub(super) const SURFACE: Color = Color::from_rgb8(0x2F, 0x33, 0x4D);
pub(super) const SURFACE_STRONG: Color = Color::from_rgb8(0x36, 0x3B, 0x59);
pub(super) const LINE: Color = Color::from_rgb8(0x41, 0x48, 0x68);
pub(super) const LINE_STRONG: Color = Color::from_rgb8(0x56, 0x5F, 0x89);
pub(super) const TEXT: Color = Color::from_rgb8(0xC0, 0xCA, 0xF5);
pub(super) const MUTED: Color = Color::from_rgb8(0x56, 0x5F, 0x89);
pub(super) const BLUE: Color = Color::from_rgb8(0x7A, 0xA2, 0xF7);
pub(super) const BLUE_HOVER: Color = Color::from_rgb8(0x89, 0xAD, 0xF8);
pub(super) const GREEN: Color = Color::from_rgb8(0x9E, 0xCE, 0x6A);
pub(super) const RED: Color = Color::from_rgb8(0xF7, 0x76, 0x8E);

pub(super) fn application() -> theme::Style {
    theme::Style {
        background_color: BLACK,
        text_color: TEXT,
    }
}

pub(super) fn divider() -> container::Style {
    container::Style {
        background: Some(Background::Color(LINE)),
        ..container::Style::default()
    }
}

pub(super) fn command_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BLACK)),
        border: Border {
            width: 1.0,
            color: LINE,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub(super) fn indicator_rail(color: Color, rounded: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: if rounded {
                iced::border::Radius {
                    top_left: 3.0,
                    bottom_left: 3.0,
                    ..iced::border::Radius::default()
                }
            } else {
                iced::border::Radius::default()
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub(super) fn radio_indicator(selected: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if selected { BLUE } else { LINE_STRONG },
        },
        ..container::Style::default()
    }
}

pub(super) fn radio_dot(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BLUE)),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub(super) fn segmented_group(frame_overlaid: bool) -> container::Style {
    let mut style = framed_group();
    if frame_overlaid {
        style.border = Border::default();
    }
    style
}

pub(super) fn segmented_frame_overlay(_theme: &Theme) -> container::Style {
    container::Style {
        border: framed_group().border,
        ..container::Style::default()
    }
}

pub(super) fn group_frame(_theme: &Theme) -> container::Style {
    framed_group()
}

pub(super) fn segment_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => SURFACE,
        button::Status::Hovered => PANEL,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        ..button::Style::default()
    }
}

pub(super) fn joined_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => SURFACE_STRONG,
        button::Status::Hovered => PANEL,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        ..button::Style::default()
    }
}

pub(super) fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = match status {
        button::Status::Hovered => (BLUE_HOVER, BLACK, BLUE_HOVER),
        button::Status::Pressed => (SURFACE_STRONG, TEXT, BLUE),
        button::Status::Disabled => (SURFACE, MUTED, LINE),
        _ => (BLUE, BLACK, BLUE),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            radius: 3.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}

pub(super) fn caption_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Pressed => SURFACE_STRONG,
        button::Status::Hovered => SURFACE,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        ..button::Style::default()
    }
}

pub(super) fn close_caption_button(status: button::Status) -> button::Style {
    if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        button::Style {
            background: Some(Background::Color(RED)),
            text_color: BLACK,
            ..button::Style::default()
        }
    } else {
        caption_button(status)
    }
}

pub(super) fn joined_input(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: MUTED,
        placeholder: MUTED,
        value: BLUE,
        selection: Color { a: 0.32, ..BLUE },
    }
}

pub(super) fn check(_theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let checked = matches!(
        status,
        checkbox::Status::Active { is_checked: true }
            | checkbox::Status::Hovered { is_checked: true }
            | checkbox::Status::Disabled { is_checked: true }
    );
    checkbox::Style {
        background: Background::Color(if checked { GREEN } else { BLACK }),
        icon_color: BLACK,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: if checked { GREEN } else { LINE_STRONG },
        },
        text_color: Some(TEXT),
    }
}

pub(super) fn progress(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(BLACK),
        bar: Background::Color(BLUE),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: LINE,
        },
    }
}

fn framed_group() -> container::Style {
    container::Style {
        background: Some(Background::Color(BLACK)),
        border: Border {
            radius: 3.0.into(),
            width: 1.0,
            color: LINE,
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framed_surfaces_and_input_fill_use_the_setup_canvas() {
        let theme = Theme::TokyoNight;
        assert_eq!(progress(&theme).background, Background::Color(BLACK));
        assert_eq!(
            group_frame(&theme).background,
            Some(Background::Color(BLACK))
        );
        let input = joined_input(&theme, text_input::Status::Active);
        assert_eq!(input.background, Background::Color(Color::TRANSPARENT));
        assert_eq!(input.value, BLUE);
    }

    #[test]
    fn primary_hover_stays_blue_and_only_lightens() {
        let hover = Color::from_rgb8(0x89, 0xAD, 0xF8);
        let active = primary_button(&Theme::TokyoNight, button::Status::Active);
        let hovered = primary_button(&Theme::TokyoNight, button::Status::Hovered);
        assert_eq!(active.background, Some(Background::Color(BLUE)));
        assert_eq!(hovered.background, Some(Background::Color(hover)));
        assert_eq!(hovered.border.color, hover);
    }

    #[test]
    fn segmented_choices_use_canvas_idle_and_panel_hover() {
        let transparent = Some(Background::Color(Color::TRANSPARENT));
        assert_eq!(
            segment_button(button::Status::Active).background,
            transparent
        );
        assert_eq!(
            segment_button(button::Status::Hovered).background,
            Some(Background::Color(PANEL))
        );
    }

    #[test]
    fn indicator_rail_rounds_only_the_outer_corners() {
        let radius = indicator_rail(BLUE, true).border.radius;
        assert_eq!(radius.top_left, 3.0);
        assert_eq!(radius.top_right, 0.0);
        assert_eq!(radius.bottom_right, 0.0);
        assert_eq!(radius.bottom_left, 3.0);
    }

    #[test]
    fn segmented_frame_overlay_reuses_one_transparent_rounded_frame() {
        let body = segmented_group(true);
        let overlay = segmented_frame_overlay(&Theme::TokyoNight);
        assert_eq!(body.background, Some(Background::Color(BLACK)));
        assert_eq!(body.border, Border::default());
        assert_eq!(overlay.background, None);
        assert_eq!(overlay.border, segmented_group(false).border);
    }
}
