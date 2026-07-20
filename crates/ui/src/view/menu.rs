//! Top menu strip + floating dropdowns. The bar doubles as the custom
//! title bar (drag handle + caption buttons) since the window runs with
//! `decorations: false`.

use iced::widget::{Space, button, column, container, mouse_area, row, svg};
use iced::{Border, Color, Element, Length, alignment};

use super::icons;
use super::menu_dropdowns::{
    FILE_DROPDOWN_WIDTH, HELP_DROPDOWN_WIDTH, MENU_ICON_SIZE, MP_DROPDOWN_WIDTH,
    VIEW_DROPDOWN_WIDTH, file_dropdown, help_dropdown, mp_dropdown, view_dropdown,
};
use super::menu_labels::{inactive_category_keys, settings_category_key};
use super::styles::{
    caption_button_style, close_caption_button_style, menu_bar_divider_style, menu_bar_style,
};
use super::theme::{tokyo_blue, tokyo_magenta, tokyo_text, ui_text};
use crate::app::{DesktopApp, MenuId, Message, TopMenuFocus, TopMenuIndicator};
use crate::i18n::Key;

const CAPTION_ICON_SIZE: f32 = 14.0;
/// Two diagonal strokes carry less optical weight than the minimise
/// stroke or maximise square at the same nominal size.
const CAPTION_CLOSE_ICON_SIZE: f32 = 16.0;
const CAPTION_BUTTON_WIDTH: f32 = 32.0;
const CAPTION_BUTTON_HEIGHT: f32 = 24.0;
const MENU_CATEGORY_PADDING: iced::Padding = iced::Padding {
    top: 3.0,
    right: 6.0,
    bottom: 3.0,
    left: 6.0,
};
const MENU_CATEGORY_SPACING: f32 = 6.0;

impl DesktopApp {
    pub(super) fn menu_bar(&self) -> Element<'_, Message> {
        // Empty space between menu and caption buttons is the
        // OS-native window drag handle.
        let drag_handle: Element<'_, Message> = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::WindowDragStart)
        .into();

        let caption_buttons = row![
            caption_button(
                icons::window_minimize(),
                Message::WindowMinimize,
                CaptionKind::Neutral,
            ),
            caption_button(
                if self.window_maximized {
                    icons::window_restore()
                } else {
                    icons::window_maximize()
                },
                Message::WindowToggleMaximize,
                CaptionKind::Neutral,
            ),
            caption_button(
                icons::window_close(),
                self.main_window_id
                    .map(Message::WindowCloseRequested)
                    .unwrap_or(Message::WindowClose),
                CaptionKind::Close
            ),
        ]
        .spacing(2)
        .align_y(alignment::Vertical::Center);

        let cpu_icon = svg(icons::cpu())
            .width(Length::Fixed(MENU_ICON_SIZE))
            .height(Length::Fixed(MENU_ICON_SIZE))
            .style(|_theme, _status| svg::Style {
                color: Some(tokyo_text()),
            });
        let cpu_toggle: Element<'_, Message> = mouse_area(
            container(cpu_icon).padding(iced::Padding::ZERO.right(MENU_CATEGORY_PADDING.right)),
        )
        .on_press(Message::MenuCategoriesToggled)
        .interaction(iced::mouse::Interaction::Pointer)
        .into();

        let mut bar_children: Vec<Element<'_, Message>> = Vec::with_capacity(8);
        bar_children.push(cpu_toggle);
        if self.menu_categories_visible {
            let category_focused = |menu| {
                self.top_menu_indicator == TopMenuIndicator::TabRing
                    && self.top_menu_focus == Some(TopMenuFocus::Category(menu))
            };
            bar_children.push(menu_trigger(
                self.lang.t(Key::MenuFile),
                MenuId::File,
                self.open_menu == Some(MenuId::File),
                category_focused(MenuId::File),
            ));
            bar_children.push(menu_trigger(
                self.lang.t(Key::MenuMp),
                MenuId::Mp,
                self.open_menu == Some(MenuId::Mp),
                category_focused(MenuId::Mp),
            ));
            bar_children.push(menu_trigger(
                self.lang.t(Key::MenuView),
                MenuId::View,
                self.open_menu == Some(MenuId::View),
                category_focused(MenuId::View),
            ));
            for key in inactive_category_keys() {
                bar_children.push(menu_label(self.lang.t(key)));
            }
            bar_children.push(settings_trigger(
                self.lang.t(settings_category_key()),
                category_focused(MenuId::Settings),
            ));
            bar_children.push(menu_trigger(
                self.lang.t(Key::MenuHelp),
                MenuId::Help,
                self.open_menu == Some(MenuId::Help),
                category_focused(MenuId::Help),
            ));
        }
        bar_children.push(drag_handle);
        bar_children.push(caption_buttons.into());

        let bar = container(
            iced::widget::Row::with_children(bar_children)
                .spacing(MENU_CATEGORY_SPACING)
                .align_y(alignment::Vertical::Center),
        )
        // Asymmetric padding equidistantly aligns the cpu glyph and
        // the close cross to the window edges. `.left(11)` is coupled
        // to FILE/MP_MENU_DROPDOWN_LEFT in `view/mod.rs`.
        .padding(iced::Padding::ZERO.left(11).right(2))
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(menu_bar_style);

        // While a dropdown is open the divider gets a hole punched
        // under it; the bleed pushes segment endpoints under the
        // frame so the dropdown's opaque fill paints over the seam.
        const DIVIDER_GAP_BLEED: f32 = -6.0;
        const ROOT_PADDING_LEFT: f32 = 8.0;
        let divider: Element<'_, Message> = match self.open_menu {
            None | Some(MenuId::Settings) => container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(menu_bar_divider_style)
                .into(),
            Some(menu) => {
                let (dropdown_left, gap_width) = match menu {
                    MenuId::File => (super::FILE_MENU_DROPDOWN_LEFT, FILE_DROPDOWN_WIDTH),
                    MenuId::Mp => (super::MP_MENU_DROPDOWN_LEFT, MP_DROPDOWN_WIDTH),
                    MenuId::View => (super::VIEW_MENU_DROPDOWN_LEFT, VIEW_DROPDOWN_WIDTH),
                    MenuId::Settings => unreachable!("settings has no dropdown"),
                    MenuId::Help => (super::HELP_MENU_DROPDOWN_LEFT, HELP_DROPDOWN_WIDTH),
                };
                let gap_local_left = (dropdown_left - ROOT_PADDING_LEFT).max(0.0);
                let left_segment_width = (gap_local_left - DIVIDER_GAP_BLEED).max(0.0);
                let gap_total_width = gap_width + DIVIDER_GAP_BLEED * 2.0;
                let line_segment = |w: Length| {
                    container(Space::new())
                        .width(w)
                        .height(Length::Fixed(1.0))
                        .style(menu_bar_divider_style)
                };
                row![
                    line_segment(Length::Fixed(left_segment_width)),
                    Space::new().width(Length::Fixed(gap_total_width)),
                    line_segment(Length::Fill),
                ]
                .height(Length::Fixed(1.0))
                .into()
            }
        };
        column![bar, divider].into()
    }

    pub(super) fn menu_dropdown(&self) -> Option<Element<'_, Message>> {
        let open_menu = self.open_menu?;
        let focused_item = match self.top_menu_focus {
            Some(TopMenuFocus::Item { menu, index })
                if self.top_menu_indicator != TopMenuIndicator::Hidden && menu == open_menu =>
            {
                Some((index, self.top_menu_indicator))
            }
            _ => None,
        };
        match open_menu {
            MenuId::File => Some(file_dropdown(
                self.lang,
                &self.shortcut_settings,
                focused_item,
            )),
            MenuId::Mp => Some(mp_dropdown(
                self.snapshot.cpu.halted,
                self.lang,
                &self.shortcut_settings,
                focused_item,
            )),
            MenuId::View => Some(view_dropdown(
                self.stack_view,
                self.lang,
                &self.shortcut_settings,
                focused_item,
            )),
            MenuId::Settings => None,
            MenuId::Help => Some(help_dropdown(
                self.lang,
                &self.shortcut_settings,
                focused_item,
            )),
        }
    }
}

fn menu_label(label: &str) -> Element<'_, Message> {
    container(ui_text(label.to_owned(), 13, tokyo_text()))
        .padding(MENU_CATEGORY_PADDING)
        .into()
}

fn menu_trigger(
    label: &str,
    menu: MenuId,
    active: bool,
    keyboard_focused: bool,
) -> Element<'_, Message> {
    let color = if active {
        tokyo_magenta()
    } else {
        tokyo_text()
    };
    mouse_area(
        container(ui_text(label.to_owned(), 13, color))
            .padding(MENU_CATEGORY_PADDING)
            .style(move |_theme| menu_trigger_style(keyboard_focused)),
    )
    .on_press(Message::MenuToggled(menu))
    .interaction(iced::mouse::Interaction::Pointer)
    .into()
}

fn menu_trigger_style(keyboard_focused: bool) -> container::Style {
    container::Style {
        border: Border {
            radius: 4.0.into(),
            width: if keyboard_focused { 1.0 } else { 0.0 },
            color: if keyboard_focused {
                tokyo_blue()
            } else {
                Color::TRANSPARENT
            },
        },
        ..container::Style::default()
    }
}

fn settings_trigger(label: &str, keyboard_focused: bool) -> Element<'_, Message> {
    mouse_area(
        container(ui_text(label.to_owned(), 13, tokyo_text()))
            .padding(MENU_CATEGORY_PADDING)
            .style(move |_theme| menu_trigger_style(keyboard_focused)),
    )
    .on_press(Message::OpenSettings)
    .interaction(iced::mouse::Interaction::Pointer)
    .into()
}

#[derive(Clone, Copy)]
enum CaptionKind {
    Neutral,
    Close,
}

fn caption_button(
    icon: svg::Handle,
    action: Message,
    kind: CaptionKind,
) -> Element<'static, Message> {
    let glyph_size = match kind {
        CaptionKind::Neutral => CAPTION_ICON_SIZE,
        CaptionKind::Close => CAPTION_CLOSE_ICON_SIZE,
    };
    let glyph = svg(icon)
        .width(Length::Fixed(glyph_size))
        .height(Length::Fixed(glyph_size))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });

    let body = container(glyph)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

    button(body)
        .on_press(action)
        .padding(0)
        .width(Length::Fixed(CAPTION_BUTTON_WIDTH))
        .height(Length::Fixed(CAPTION_BUTTON_HEIGHT))
        .style(move |_theme, status| match kind {
            CaptionKind::Neutral => caption_button_style(status),
            CaptionKind::Close => close_caption_button_style(status),
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_focused_menu_category_uses_blue_border() {
        let style = menu_trigger_style(true);

        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.color, tokyo_blue());
    }
}
