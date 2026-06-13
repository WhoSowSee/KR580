use iced::widget::{Space, button, column, container, mouse_area, opaque, row, scrollable, stack};
use iced::{Element, Length};

use super::super::theme::{TOKYO_BORDER, TOKYO_MUTED, TOKYO_SURFACE, TOKYO_TEXT, ui_text};
use super::consts::{CONTENT_PADDING, SETTING_ROW_HEIGHT};
use super::language::{language_dropdown_list, language_setting_row};
use super::network::network_defaults_row;
use super::setting_row::setting_row;
use super::shortcuts_row::shortcuts_setting_row;
use super::speed::speed_setting_row;
use super::theme_row::theme_setting_row;
use crate::app::{ContentFocus, Message, SettingsCategory, SettingsDialog, SettingsSection};
use crate::i18n::{Key, Lang, NetworkKey};

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

    let body: Element<'a, Message> = if rows.is_empty() {
        container(ui_text(lang.t(Key::SettingsNoMatches), 13, TOKYO_MUTED))
            .padding(CONTENT_PADDING)
            .into()
    } else {
        column(rows).spacing(20).padding(CONTENT_PADDING).into()
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
            ) {
                out.push(theme_setting_row(dialog, lang));
            }
        }
        SettingsCategory::Shortcuts => {
            if matches_query(
                &[Key::SettingsShortcutsLabel, Key::SettingsShortcutsHint],
                lang,
                lower_query,
            ) {
                out.push(shortcuts_setting_row(dialog, lang));
            }
        }
    }
}

fn group_header(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 11, TOKYO_MUTED).into()
}

fn follow_pc_toggle_row<'a>(dialog: &'a SettingsDialog, lang: Lang) -> Element<'a, Message> {
    use super::consts::TOGGLE_SEGMENT_WIDTH;
    use super::speed::segmented_button_width;

    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();

    let kb_focused = kb_focus == Some(ContentFocus::FollowPc);

    let segments = row![
        segmented_button_width(
            lang.t(Key::SettingsToggleOn),
            dialog.draft_follow_pc,
            kb_focused,
            Message::SettingsDraftFollowPcSet(true),
            TOGGLE_SEGMENT_WIDTH,
        ),
        segmented_button_width(
            lang.t(Key::SettingsToggleOff),
            !dialog.draft_follow_pc,
            false,
            Message::SettingsDraftFollowPcSet(false),
            TOGGLE_SEGMENT_WIDTH,
        ),
    ]
    .spacing(6);

    setting_row(
        lang.t(Key::SettingsFollowPcLabel),
        lang.t(Key::SettingsFollowPcHint),
        segments.into(),
    )
}

fn hdd_directory_row<'a>(dialog: &'a SettingsDialog, lang: Lang) -> Element<'a, Message> {
    use iced::{Background, Border, Color};

    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let kb_focused = kb_focus == Some(ContentFocus::HddDirectory);

    let raw_path = dialog
        .draft_hdd_directory
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_owned())
        });
    let display = truncate_path(&raw_path, 36);

    let browse_btn = button(
        container(ui_text(
            lang.t(Key::SettingsHddDirectoryBrowse),
            12,
            TOKYO_TEXT,
        ))
        .padding([2, 8]),
    )
    .on_press(Message::SettingsHddDirectoryBrowse)
    .style(move |_theme, status| {
        use iced::widget::button;
        let bg = match status {
            button::Status::Pressed => TOKYO_BORDER,
            button::Status::Hovered => TOKYO_SURFACE,
            _ if kb_focused => TOKYO_SURFACE,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TOKYO_TEXT,
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: TOKYO_BORDER,
            },
            ..button::Style::default()
        }
    });

    let control = row![
        Space::new().width(Length::Fill),
        ui_text(display, 13, TOKYO_MUTED),
        Space::new().width(Length::Fixed(8.0)),
        browse_btn,
    ]
    .align_y(iced::alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsHddDirectoryLabel),
        lang.t(Key::SettingsHddDirectoryHint),
        control.into(),
    )
}

fn truncate_path(path: &str, max: usize) -> String {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() <= max {
        return path.to_owned();
    }
    let sep = if path.contains('\\') { '\\' } else { '/' };
    let segments: Vec<&str> = path.split(sep).collect();
    if segments.len() < 3 {
        let head: String = chars.iter().take(max / 2).collect();
        let tail: String = chars.iter().skip(chars.len() - max / 2).collect();
        return format!("{head}…{tail}");
    }
    let first = segments[0];
    let last = segments[segments.len() - 1];
    let mut budget = max.saturating_sub(first.chars().count() + last.chars().count() + 3);
    let mut middle = String::new();
    for seg in segments[1..segments.len() - 1].iter().rev() {
        let cost = seg.chars().count() + 1;
        if cost > budget {
            break;
        }
        middle = format!("{sep}{seg}{middle}");
        budget -= cost;
    }
    if middle.is_empty() {
        format!("{first}{sep}…{sep}{last}")
    } else {
        format!("{first}{sep}…{middle}{sep}{last}")
    }
}

pub(super) fn matches_query(keys: &[Key], lang: Lang, lower_query: &str) -> bool {
    if lower_query.is_empty() {
        return true;
    }
    keys.iter()
        .any(|k| lang.t(*k).to_lowercase().contains(lower_query))
}
