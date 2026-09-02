//! Settings dialog overlay – entry point.
//!
//! Composes the four-zone modal (header / sidebar / content / footer)
//! plus the reset-confirm sub-modal. Layout primitives, styles, and
//! per-zone widgets live in submodules; this file only orchestrates
//! their composition.

mod consts;
mod content;
mod footer;
mod header;
mod language;
mod network;
mod reset_confirm;
mod setting_row;
mod shortcuts_row;
mod sidebar;
mod speed;
mod styles;
mod theme_row;

use iced::widget::{Space, column, container, keyed_column, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use consts::{DIALOG_HEIGHT, DIALOG_WIDTH};
use content::settings_content;
use footer::settings_footer;
use header::settings_header;
use reset_confirm::reset_confirm_overlay;
use setting_row::{separator_horizontal, separator_vertical};
use sidebar::settings_sidebar;
use styles::{modal_backdrop_style, modal_dialog_style};

use crate::app::{Message, SettingsDialog};
use crate::i18n::Lang;

pub(super) fn settings_modal_overlay<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
    file_association_toggle_revision: u64,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseSettings);

    let body = container(
        column![
            settings_header(lang),
            separator_horizontal(),
            row![
                settings_sidebar(dialog, lang),
                separator_vertical(),
                settings_content(dialog, lang),
            ]
            .height(Length::Fill),
            separator_horizontal(),
            settings_footer(dialog, lang),
        ]
        .width(Length::Fixed(DIALOG_WIDTH))
        .height(Length::Fixed(DIALOG_HEIGHT)),
    )
    .style(modal_dialog_style);

    let body: Element<'a, Message> =
        keyed_column(vec![(file_association_toggle_revision, body.into())]).into();

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let modal: Element<'a, Message> = stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    if dialog.reset_confirm_open {
        stack![
            modal,
            reset_confirm_overlay(
                dialog.reset_confirm_focus,
                dialog.reset_confirm_keyboard_focus_visible,
                lang,
            )
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        modal
    }
}

#[cfg(test)]
mod tests {
    use iced::advanced::renderer::Headless;
    use iced::advanced::{Layout, Shell, clipboard, layout, widget};
    use iced::mouse;
    use iced::widget::Space;
    use iced::{Element, Event, Point, Size};

    use super::super::theme::{DARK_COLOR_SCHEMES, LIGHT_COLOR_SCHEMES};
    use super::consts::DIALOG_HEIGHT;
    use super::content::{category_matches_query, matches_query, settings_content};
    use super::language::language_label_key;
    use super::shortcuts_row::shortcut_action_matches_query;
    use super::theme_row::theme_option_matches_query;
    use crate::app::messages::SpeedTier;
    use crate::app::{Message, SettingsCategory, SettingsDialog};
    use crate::i18n::{Key, Lang};
    use crate::persistence::{ColorScheme, NetworkSettings, ShortcutAction};

    #[test]
    fn empty_query_matches_every_row() {
        assert!(matches_query(&[Key::SettingsLanguageLabel], Lang::Ru, ""));
    }

    #[test]
    fn russian_query_matches_russian_label() {
        assert!(matches_query(
            &[Key::SettingsLanguageLabel, Key::SettingsLanguageHint],
            Lang::Ru,
            "язык"
        ));
    }

    #[test]
    fn english_query_misses_when_label_is_russian_only() {
        assert!(!matches_query(
            &[Key::SettingsLanguageLabel],
            Lang::Ru,
            "language"
        ));
    }

    #[test]
    fn localized_category_name_matches_search() {
        for (category, query) in [
            (SettingsCategory::General, "общие"),
            (SettingsCategory::ExternalDevices, "внешние устройства"),
            (SettingsCategory::Appearance, "внешний вид"),
            (SettingsCategory::Shortcuts, "горячие клавиши"),
        ] {
            assert!(category_matches_query(category, Lang::Ru, query));
        }
    }

    #[test]
    fn plural_theme_name_matches_search() {
        for (lang, query) in [(Lang::Ru, "темы"), (Lang::En, "themes")] {
            assert!(matches_query(
                &[Key::SettingsThemeLabel, Key::SettingsThemeHint],
                lang,
                query
            ));
        }
    }

    #[test]
    fn language_label_key_round_trips_per_lang() {
        assert_eq!(language_label_key(Lang::Ru), Key::LangRussian);
        assert_eq!(language_label_key(Lang::En), Key::LangEnglish);
    }

    #[test]
    fn shortcut_query_matches_only_the_requested_action() {
        let matches: Vec<_> = ShortcutAction::ALL
            .into_iter()
            .filter(|action| shortcut_action_matches_query(*action, Lang::Ru, "перейти к ffff"))
            .collect();

        assert_eq!(matches, vec![ShortcutAction::JumpMemoryEnd]);
    }

    #[test]
    fn theme_query_matches_only_the_requested_option() {
        let matches: Vec<_> = DARK_COLOR_SCHEMES
            .iter()
            .chain(LIGHT_COLOR_SCHEMES.iter())
            .copied()
            .filter(|scheme| theme_option_matches_query(*scheme, Lang::En, "material ocean"))
            .collect();

        assert_eq!(matches, vec![ColorScheme::MaterialOcean]);
    }

    #[test]
    fn hints_carry_no_trailing_period() {
        for key in [
            Key::SettingsLanguageHint,
            Key::SettingsSpeedHint,
            Key::SettingsThemeHint,
            Key::SettingsShortcutsHint,
        ] {
            for lang in [Lang::Ru, Lang::En] {
                let hint = lang.t(key);
                assert!(
                    !hint.ends_with('.'),
                    "{lang:?} {key:?} hint ends with a period: {hint:?}",
                );
            }
        }
    }

    #[test]
    fn settings_dialog_balances_vertical_content_margins() {
        let height = std::hint::black_box(DIALOG_HEIGHT);

        assert_eq!(height, 496.0);
    }

    #[test]
    fn settings_content_captures_wheel_without_native_double_scroll() {
        let dialog = test_dialog();
        let mut root = settings_content(&dialog, Lang::Ru);
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("renderer runtime")
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .expect("software renderer");
        let mut tree = widget::Tree::new(&root);
        let node = root.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(540.0, 388.0)),
        );
        let layout = Layout::new(&node);

        for (delta, expected) in [
            (mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 }, 40.0),
            (mouse::ScrollDelta::Pixels { x: 0.0, y: -12.5 }, 12.5),
        ] {
            let mut messages = Vec::new();
            let mut shell = Shell::new(&mut messages);
            root.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled { delta }),
                layout,
                mouse::Cursor::Available(Point::new(270.0, 194.0)),
                &renderer,
                &mut clipboard::Null,
                &mut shell,
                &layout.bounds(),
            );

            assert!(matches!(
                messages.as_slice(),
                [Message::SettingsContentWheelScrolled(actual)] if *actual == expected
            ));
        }
    }

    #[test]
    fn scroll_hint_visibility_preserves_scrollable_tree_path() {
        let mut dialog = test_dialog();
        dialog.content_can_scroll_down = true;
        let visible = scrollable_tag_path(&dialog);
        dialog.content_can_scroll_down = false;
        let hidden = scrollable_tag_path(&dialog);

        assert_eq!(visible, hidden);
    }

    fn scrollable_tag_path(dialog: &SettingsDialog) -> Vec<widget::tree::Tag> {
        let root = settings_content(dialog, Lang::Ru);
        let tree = widget::Tree::new(&root);
        let scrollable: Element<'_, Message> = iced::widget::scrollable(Space::new()).into();
        tag_path(&tree, scrollable.as_widget().tag()).expect("settings scrollable")
    }

    fn test_dialog() -> SettingsDialog {
        SettingsDialog::new(
            Lang::Ru,
            SpeedTier::High,
            false,
            true,
            None,
            None,
            NetworkSettings::default(),
        )
    }

    fn tag_path(tree: &widget::Tree, target: widget::tree::Tag) -> Option<Vec<widget::tree::Tag>> {
        if tree.tag == target {
            return Some(vec![tree.tag]);
        }
        tree.children.iter().find_map(|child| {
            let mut path = tag_path(child, target)?;
            path.insert(0, tree.tag);
            Some(path)
        })
    }
}
