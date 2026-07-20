use crate::app::settings_modal::{FooterFocus, SettingsDialog, SettingsSection};

pub(super) fn cycle_section(dialog: &mut SettingsDialog, backward: bool) {
    dialog.language_dropdown_open = false;
    dialog.dropdown_highlight = None;
    dialog.keyboard_focus_visible = true;
    let next = if backward {
        dialog.section.previous()
    } else {
        dialog.section.next()
    };
    dialog.section = next;
    match next {
        SettingsSection::Search => {}
        SettingsSection::Sidebar => dialog.sidebar_focus = dialog.category,
        SettingsSection::Content => {
            dialog.content_focus = Some(if backward {
                dialog.last_content_focus()
            } else {
                dialog.first_content_focus()
            });
        }
        SettingsSection::Footer => {
            dialog.footer_focus = if backward {
                FooterFocus::Save
            } else {
                FooterFocus::Cancel
            };
        }
    }
}
