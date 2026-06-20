use iced::widget::{button, container, scrollable, text_editor, text_input};
use iced::{Background, Border, Color};

use crate::view::styles::input_borderless_style;
pub(super) use crate::view::styles::{
    large_dialog_style as modal_dialog_style, modal_backdrop_style,
};
use crate::view::theme::{
    TOKYO_BORDER, TOKYO_MUTED, TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT, TOKYO_TEXT_SELECTION,
};

pub(super) fn sidebar_chip_style(
    status: button::Status,
    active: bool,
    keyboard_focused: bool,
) -> button::Style {
    let background = if active || keyboard_focused {
        TOKYO_SURFACE
    } else {
        match status {
            button::Status::Pressed => TOKYO_SURFACE_2,
            button::Status::Hovered => TOKYO_SURFACE,
            _ => Color::TRANSPARENT,
        }
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn breadcrumb_button_style(status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => TOKYO_TEXT,
        _ => TOKYO_MUTED,
    };
    button::Style {
        background: None,
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..button::Style::default()
    }
}

pub(super) fn help_search_input_style(
    theme: &iced::Theme,
    status: text_input::Status,
) -> text_input::Style {
    text_input::Style {
        selection: TOKYO_TEXT_SELECTION,
        ..input_borderless_style(theme, status)
    }
}

pub(super) fn help_text_editor_style(
    _theme: &iced::Theme,
    _status: text_editor::Status,
) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        placeholder: TOKYO_MUTED,
        value: TOKYO_TEXT,
        selection: TOKYO_TEXT_SELECTION,
    }
}

pub(super) fn separator_horizontal() -> container::Style {
    container::Style {
        background: Some(Background::Color(iced::Color {
            a: 0.35,
            ..TOKYO_BORDER
        })),
        ..container::Style::default()
    }
}

pub(super) fn separator_vertical() -> container::Style {
    separator_horizontal()
}

pub(super) fn hidden_scrollbar_style(
    _theme: &iced::Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    let invisible_scroller = scrollable::Scroller {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    };
    let invisible_rail = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        scroller: invisible_scroller,
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: invisible_rail,
        horizontal_rail: invisible_rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: iced::Shadow::default(),
            icon: Color::TRANSPARENT,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Theme;

    #[test]
    fn help_search_input_selection_uses_readable_gray_overlay() {
        let style = help_search_input_style(
            &Theme::TokyoNight,
            text_input::Status::Focused { is_hovered: false },
        );

        assert_eq!(style.selection, TOKYO_TEXT_SELECTION);
    }

    #[test]
    fn breadcrumb_hover_highlights_text_without_own_background() {
        let style = breadcrumb_button_style(button::Status::Hovered);

        assert_eq!(style.background, None);
        assert_eq!(style.text_color, TOKYO_TEXT);
    }

    #[test]
    fn help_text_editor_keeps_article_surface_transparent() {
        let style = help_text_editor_style(&Theme::TokyoNight, text_editor::Status::Active);

        assert_eq!(style.background, Background::Color(Color::TRANSPARENT));
        assert_eq!(style.selection, TOKYO_TEXT_SELECTION);
    }
}
