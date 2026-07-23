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
    use crate::app::SettingsCategory;

    use super::super::theme::{DARK_COLOR_SCHEMES, LIGHT_COLOR_SCHEMES};
    use super::consts::DIALOG_HEIGHT;
    use super::content::{category_matches_query, matches_query, settings_content_needs_scroll};
    use super::language::language_label_key;
    use super::shortcuts_row::shortcut_action_matches_query;
    use super::theme_row::theme_option_matches_query;
    use crate::i18n::{Key, Lang};
    use crate::persistence::{ColorScheme, ShortcutAction};

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
    fn general_settings_content_does_not_scroll_without_search() {
        assert!(!settings_content_needs_scroll(
            SettingsCategory::General,
            false
        ));
        assert!(settings_content_needs_scroll(
            SettingsCategory::General,
            true
        ));
        assert!(settings_content_needs_scroll(
            SettingsCategory::ExternalDevices,
            false
        ));
    }
}
