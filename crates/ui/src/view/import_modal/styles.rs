use iced::widget::{button, container};
use iced::{Background, Border, Color};

pub(super) use super::super::styles::{
    inset_style as dropdown_panel_style, modal_backdrop_style,
    modal_dropdown_option_style as dropdown_option_style,
    modal_field_button_style as field_button_style,
    modal_field_button_style as footer_button_style, panel_style as modal_dialog_style,
};
use super::super::theme::{tokyo_blue, tokyo_border, tokyo_surface, tokyo_text};

pub(super) fn field_button_style_focused(status: button::Status, focused: bool) -> button::Style {
    let mut style = field_button_style(status);
    if focused {
        style.border.color = tokyo_text();
    }
    style
}

pub(super) fn target_anchor_style_focused(status: button::Status, focused: bool) -> button::Style {
    let mut style = field_button_style_focused(status, focused);
    if status == button::Status::Pressed {
        style.background = field_button_style(button::Status::Hovered).background;
    }
    style
}

pub(super) fn drop_zone_style(
    _theme: &iced::Theme,
    hovered: bool,
    focused: bool,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: if hovered { 0.42 } else { 0.2 },
            ..tokyo_surface()
        })),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: if hovered {
                tokyo_blue()
            } else if focused {
                tokyo_text()
            } else {
                tokyo_border()
            },
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{drop_zone_style, field_button_style_focused, target_anchor_style_focused};
    use crate::view::theme::{tokyo_blue, tokyo_text};
    use iced::Theme;
    use iced::widget::button;

    #[test]
    fn keyboard_focused_field_uses_text_border() {
        let style = field_button_style_focused(button::Status::Active, true);

        assert_eq!(style.border.color, tokyo_text());
    }

    #[test]
    fn hovered_drop_zone_uses_accent_border() {
        let style = drop_zone_style(&Theme::TokyoNight, true, false);

        assert_eq!(style.border.color, tokyo_blue());
    }

    #[test]
    fn target_anchor_keeps_hover_fill_while_pressed() {
        let hovered = target_anchor_style_focused(button::Status::Hovered, false);
        let pressed = target_anchor_style_focused(button::Status::Pressed, false);

        assert_eq!(pressed.background, hovered.background);
    }
}
