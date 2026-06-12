use iced::widget::container;
use iced::{Background, Border, Color};

use super::super::styles::input_shell_style;
pub(super) use super::super::styles::{
    inset_style as dropdown_panel_style, inset_style as group_panel_style,
    legend_label_style as group_label_style, modal_backdrop_style,
    modal_dropdown_option_style as dropdown_option_style,
    modal_field_button_style as field_button_style,
    modal_field_button_style as footer_button_style, panel_style as modal_dialog_style,
};
use super::super::theme::{TOKYO_BORDER, TOKYO_RED, TOKYO_SURFACE};

pub(super) fn badge_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.35,
            ..TOKYO_SURFACE
        })),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..container::Style::default()
    }
}

pub(super) fn error_shell_style(theme: &iced::Theme) -> container::Style {
    let mut style = input_shell_style(theme, false);
    style.border.color = TOKYO_RED;
    style
}
