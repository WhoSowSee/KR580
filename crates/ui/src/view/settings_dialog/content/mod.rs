use iced::widget::{Space, column, container, mouse_area, opaque, row, scrollable, stack};
use iced::{Element, Length, Padding};

use super::super::theme::{tokyo_muted, ui_text};
use super::consts::{CONTENT_PADDING, SETTING_ROW_HEIGHT};
use super::language::{language_dropdown_list, language_setting_row};
use super::network::network_defaults_row;
use super::shortcuts_row::shortcuts_setting_row;
use super::speed::speed_setting_row;
use super::theme_row::{theme_search_matches, theme_setting_row};
use crate::app::{Message, SettingsCategory, SettingsDialog};
use crate::i18n::{Key, Lang, NetworkKey};

mod rows;
use rows::*;

pub(super) fn settings_content<'a>(dialog: &'a SettingsDialog, lang: Lang) -> Element<'a, Message> {
    let lower_query = dialog.search_query().to_lowercase();
    let searching = !lower_query.is_empty();

    let mut rows: Vec<Element<'a, Message>> = Vec::new();
    let mut language_row_index: Option<usize> = None;

    if searching {
        for (i, cat) in SettingsCategory::ALL.iter().enumerate() {
            let mut group: Vec<Element<'a, Message>> = Vec::new();
            collect_category_rows(*cat, dialog, lang, &lower_query, &mut group, &mut |idx| {
                language_row_index = Some(rows.len() + 1 + idx);
            });
            if group.is_empty() {
                continue;
            }
            if i > 0 && !rows.is_empty() {
                rows.push(Space::new().height(Length::Fixed(8.0)).into());
            }
            rows.push(group_header(lang.t(cat.label_key())));
            rows.extend(group);
        }
    } else {
        collect_category_rows(
            dialog.category,
            dialog,
            lang,
            &lower_query,
            &mut rows,
            &mut |idx| {
                language_row_index = Some(idx);
            },
        );
    }

    let content_padding = if !searching && dialog.category == SettingsCategory::Shortcuts {
        Padding {
            top: 14.0,
            right: CONTENT_PADDING,
            bottom: CONTENT_PADDING,
            left: CONTENT_PADDING,
        }
    } else {
        Padding::from(CONTENT_PADDING)
    };

    let body: Element<'a, Message> = if rows.is_empty() {
        container(ui_text(lang.t(Key::SettingsNoMatches), 13, tokyo_muted()))
            .padding(content_padding)
            .into()
    } else {
        column(rows).spacing(20).padding(content_padding).into()
    };

    let body: Element<'a, Message> = scrollable(body)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    let body: Element<'a, Message> = match (dialog.language_dropdown_open, language_row_index) {
        (true, Some(idx)) if !searching => {
            let row_top = CONTENT_PADDING + (idx as f32) * (SETTING_ROW_HEIGHT + 20.0);
            // Slight overlap (-4 px) so the dropdown panel reads as a
            // continuation of the anchor's chrome instead of a panel
            // floating below it.
            let overlay_top = row_top + SETTING_ROW_HEIGHT - 4.0;
            // When the user has moved the keyboard highlight, the
            // selected row stops painting filled so only one option
            // reads as "active under the cursor". When no highlight
            // exists yet (dropdown was just opened), selected stands
            // in for it.
            let (visible_selection, highlighted) = match dialog.dropdown_highlight {
                Some(h) => (None, h),
                None => (Some(dialog.draft_lang), dialog.draft_lang),
            };
            let dropdown = language_dropdown_list(visible_selection, highlighted, lang);
            let close_layer = mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message::SettingsLanguageDropdownToggled);

            let positioned_dropdown = column![
                Space::new().height(Length::Fixed(overlay_top)),
                row![
                    Space::new().width(Length::Fill),
                    opaque(dropdown),
                    Space::new().width(Length::Fixed(CONTENT_PADDING)),
                ]
                .width(Length::Fill),
                Space::new().height(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            stack![body, close_layer, positioned_dropdown]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        _ => body,
    };

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn collect_category_rows<'a>(
    category: SettingsCategory,
    dialog: &'a SettingsDialog,
    lang: Lang,
    lower_query: &str,
    out: &mut Vec<Element<'a, Message>>,
    mut on_language_row: impl FnMut(usize),
) {
    let start = out.len();
    match category {
        SettingsCategory::General => {
            if matches_query(
                &[Key::SettingsLanguageLabel, Key::SettingsLanguageHint],
                lang,
                lower_query,
            ) {
                on_language_row(out.len() - start);
                out.push(language_setting_row(dialog, lang));
            }
            if matches_query(
                &[Key::SettingsSpeedLabel, Key::SettingsSpeedHint],
                lang,
                lower_query,
            ) {
                out.push(speed_setting_row(dialog, lang));
            }
            if matches_query(
                &[Key::SettingsFollowPcLabel, Key::SettingsFollowPcHint],
                lang,
                lower_query,
            ) {
                out.push(follow_pc_toggle_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::SettingsMemoryOperandHighlightingLabel,
                    Key::SettingsMemoryOperandHighlightingHint,
                ],
                lang,
                lower_query,
            ) {
                out.push(memory_operand_highlighting_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::SettingsFileAssociationLabel,
                    Key::SettingsFileAssociationHint,
                    Key::SettingsFileAssociationAdd,
                    Key::SettingsFileAssociationRemove,
                ],
                lang,
                lower_query,
            ) {
                out.push(file_association_row(dialog, lang));
            }
        }
        SettingsCategory::ExternalDevices => {
            if matches_query(
                &[Key::SettingsFloppyImageLabel, Key::SettingsFloppyImageHint],
                lang,
                lower_query,
            ) {
                out.push(floppy_image_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::SettingsHddDirectoryLabel,
                    Key::SettingsHddDirectoryHint,
                ],
                lang,
                lower_query,
            ) {
                out.push(hdd_directory_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::SettingsPrinterLabel,
                    Key::SettingsPrinterHint,
                    Key::SettingsPrinterSetup,
                    Key::SettingsPrinterSystemDefault,
                    Key::SettingsPrinterClear,
                ],
                lang,
                lower_query,
            ) {
                out.push(printer_default_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::SettingsPrinterDialogModeLabel,
                    Key::SettingsPrinterDialogModeHint,
                    Key::SettingsPrinterDialogModeCustom,
                    Key::SettingsPrinterDialogModeSystem,
                ],
                lang,
                lower_query,
            ) {
                out.push(printer_dialog_mode_row(dialog, lang));
            }
            if matches_query(
                &[
                    Key::Network(NetworkKey::GeneralSettingsLabel),
                    Key::Network(NetworkKey::GeneralSettingsHint),
                    Key::Network(NetworkKey::ModeClient),
                    Key::Network(NetworkKey::ModeServer),
                ],
                lang,
                lower_query,
            ) {
                out.push(network_defaults_row(dialog, lang));
            }
        }
        SettingsCategory::Appearance => {
            if matches_query(
                &[Key::SettingsThemeLabel, Key::SettingsThemeHint],
                lang,
                lower_query,
            ) || theme_search_matches(lang, lower_query)
            {
                out.push(theme_setting_row(dialog, lang));
            }
        }
        SettingsCategory::Shortcuts => {
            if matches_query(
                &[Key::SettingsShortcutsLabel, Key::SettingsShortcutsHint],
                lang,
                lower_query,
            ) || crate::app::shortcuts::shortcut_search_matches(lang, lower_query)
            {
                out.push(shortcuts_setting_row(dialog, lang));
            }
        }
    }
}

fn group_header(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 11, tokyo_muted()).into()
}

pub(super) fn matches_query(keys: &[Key], lang: Lang, lower_query: &str) -> bool {
    if lower_query.is_empty() {
        return true;
    }
    keys.iter()
        .any(|k| lang.t(*k).to_lowercase().contains(lower_query))
}
