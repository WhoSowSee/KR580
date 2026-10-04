//! View layer for the desktop UI.

mod about;
mod changelog;
mod chips;
mod current_command;
mod cycles;
mod device_toolbar;
mod editors;
mod export_modal;
mod file_drop;
mod font_warmup;
mod help;
mod icons;
mod import_modal;
mod lamps;
mod memory_list;
pub(crate) use memory_list::{operand_jump_target, operand_port_number};
mod menu;
mod menu_dropdowns;
mod menu_labels;
mod modal;
mod monitor;
mod monitor_font;
pub(crate) mod monitor_image;
mod mux;
mod network;
mod network_settings;
mod notices;
mod opcode_dropdown;
mod printer;
mod printer_setup;
mod schematic;
mod settings_dialog;
mod speed;
mod status_register;
mod storage;
mod styles;
mod subprogram_modal;
pub(crate) mod theme;
mod tooltips;
mod utils;
mod widgets;
mod windows;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use modal::discard_modal_overlay;
use monitor::monitor_window_overlay;
use network::network_window_overlay;
use notices::{error_notice_overlay, halt_notice_overlay, with_settings_notice};
use printer::printer_window_overlay;
use printer_setup::with_printer_setup_overlay;
use settings_dialog::settings_modal_overlay;
use storage::{floppy_window_overlay, hdd_window_overlay};
use styles::app_style;

use about::about_modal_overlay;
use changelog::changelog_modal_overlay;
use export_modal::{ExportModalViewState, export_modal_overlay};
use file_drop::with_file_drop_hover;
use help::help_modal_overlay;
use import_modal::{ImportModalViewState, import_modal_overlay};
use subprogram_modal::{SubprogramModalViewState, subprogram_modal_overlay};

use crate::app::{DesktopApp, MenuId, Message, PendingAction};
use crate::i18n::Key;

/// Vertical offset of the dropdown so its top border sits on the
/// menu bar's bottom hairline.
const MENU_DROPDOWN_TOP: f32 = 34.0;

/// Per-trigger horizontal offset. Tied to `.left(11)` padding in
/// `menu/menu_bar()`. Exposed so the bar's hairline can punch a hole
/// under the open dropdown.
pub(super) const FILE_MENU_DROPDOWN_LEFT: f32 = 39.0;
pub(super) const MP_MENU_DROPDOWN_LEFT: f32 = 93.0;
/// View menu trigger offset between MP and Settings. Coupled to the
/// same `.left(11)` bar padding and the per-trigger label widths.
pub(super) const VIEW_MENU_DROPDOWN_LEFT: f32 = 130.0;
/// Right-most menu trigger ("Помощь" / "Help"). Coupled to the same
/// `.left(11)` bar padding and the per-trigger label widths between
/// `MP_MENU_DROPDOWN_LEFT` and this offset.
pub(super) const HELP_MENU_DROPDOWN_LEFT: f32 = 308.0;

impl DesktopApp {
    fn main_view(&self) -> Element<'_, Message> {
        let main = row![self.schematic_panel(), self.side_panel()]
            .spacing(8)
            .height(Length::Fill);

        let content = column![self.menu_bar(), main]
            .padding(iced::Padding {
                top: 0.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            })
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        let app_root: Element<'_, Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(app_style)
            .into();
        let app_root = font_warmup::wrap_startup(self.shell.startup_frames_seen, app_root);

        let app_with_menu: Element<'_, Message> = if let Some(dropdown) = self.menu_dropdown() {
            let left = match self.shell.open_menu {
                Some(MenuId::File) => FILE_MENU_DROPDOWN_LEFT,
                Some(MenuId::Mp) => MP_MENU_DROPDOWN_LEFT,
                Some(MenuId::View) => VIEW_MENU_DROPDOWN_LEFT,
                Some(MenuId::Help) => HELP_MENU_DROPDOWN_LEFT,
                Some(MenuId::Settings) | None => FILE_MENU_DROPDOWN_LEFT,
            };
            stack![app_root, menu_dropdown_overlay(dropdown, left)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            app_root
        };
        let app_with_menu = with_file_drop_hover(
            app_with_menu,
            self.document.file_drag_hovered,
            self.document.file_drag_cursor_position,
            self.shell.main_window_size,
            self.preferences.lang.t(Key::FileDropOpenHint),
        );

        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.execution.halt_notice.as_deref() {
                stack![app_with_menu, halt_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_menu
            };

        let app_with_overlays: Element<'_, Message> =
            if let Some(notice) = self.shell.error_notice.as_deref() {
                stack![app_with_overlays, error_notice_overlay(notice)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                app_with_overlays
            };

        let scrimmed: Element<'_, Message> = if self.memory.opcode_dropdown_address.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::HideOpcodeDropdown)
                .into()
        } else if self.shell.open_menu.is_some() {
            mouse_area(app_with_overlays)
                .on_press(Message::MenuClosed)
                .into()
        } else {
            app_with_overlays
        };

        let layered: Element<'_, Message> =
            if let Some(action) = self.document.pending_action.as_ref() {
                let modal = discard_modal_overlay(
                    action,
                    self.document.discard_modal_focus,
                    self.document.discard_modal_keyboard_focus_visible,
                    self.preferences.lang,
                );
                if matches!(action, PendingAction::DeleteHdd)
                    && self.panels.hdd_open
                    && !self.panels.hdd_window.detached()
                {
                    stack![
                        scrimmed,
                        hdd_window_overlay(
                            &self.snapshot.devices.hdd,
                            self.panels.hdd_file_exists,
                            self.panels.hdd_show_image_contents,
                            &self.panels.hdd_image_contents,
                            self.panels.hdd_image_error.as_deref(),
                            self.preferences.lang,
                            self.device_toolbar(crate::app::ToolWindowKind::Hdd),
                        ),
                        modal,
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    stack![scrimmed, modal]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            } else if self.export.export_modal_open {
                stack![
                    scrimmed,
                    export_modal_overlay(ExportModalViewState {
                        tab: self.export.export_tab,
                        focus: self.export.export_modal_focus,
                        keyboard_focus_visible: self.export.export_modal_keyboard_focus_visible,
                        target_input: self.export_target_input(),
                        target_options: self.export_target_options(),
                        target_dropdown_open: self.export.export_target_dropdown.is_open(),
                        target_highlight: self.export.export_target_dropdown.highlight(),
                        memory_start: &self.export.export_memory_start_input,
                        memory_end: &self.export.export_memory_end_input,
                        columns: self.export.export_memory_columns,
                        registers: self.export.export_registers,
                        flags: self.export.export_flags,
                        lang: self.preferences.lang,
                    })
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if self.import.import_modal_open {
                stack![
                    scrimmed,
                    import_modal_overlay(ImportModalViewState {
                        focus: self.import.import_modal_focus,
                        keyboard_focus_visible: self.import.import_modal_keyboard_focus_visible,
                        file_drag_hovered: self.import.import_file_drag_hovered,
                        file_display: &self.import.import_file_display,
                        format: self.import.import_file_format,
                        target_input: &self.import.import_target_input,
                        target_options: &self.import.import_target_options,
                        target_dropdown_open: self.import.import_target_dropdown.is_open(),
                        target_highlight: self.import.import_target_dropdown.highlight(),
                        error: self.import.import_error.as_deref(),
                        lang: self.preferences.lang,
                    })
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if let Some(dialog) = self.document.subprogram_dialog.as_ref() {
                stack![
                    scrimmed,
                    subprogram_modal_overlay(SubprogramModalViewState {
                        mode: dialog.mode,
                        focus: dialog.focus,
                        keyboard_focus_visible: dialog.keyboard_focus_visible,
                        path: &dialog.path,
                        start: &dialog.start_input,
                        end: &dialog.end_input,
                        error: dialog.error.as_deref(),
                        lang: self.preferences.lang,
                    })
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if let Some(dialog) = self.preferences.settings_dialog.as_ref() {
                stack![
                    scrimmed,
                    settings_modal_overlay(
                        dialog,
                        self.preferences.lang,
                        self.preferences.file_association_pending
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if let Some(dialog) = self.shell.changelog_dialog.as_ref() {
                let about = stack![scrimmed, about_modal_overlay(self.preferences.lang)]
                    .width(Length::Fill)
                    .height(Length::Fill);
                stack![
                    about,
                    changelog_modal_overlay(dialog, self.preferences.lang)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if self.shell.about_dialog_open {
                stack![scrimmed, about_modal_overlay(self.preferences.lang)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else if let Some(dialog) = self.shell.help_dialog.as_ref() {
                stack![scrimmed, help_modal_overlay(dialog, self.preferences.lang)]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else if self.panels.monitor_open && !self.panels.monitor_window.detached() {
                stack![
                    scrimmed,
                    monitor_window_overlay(
                        &self.snapshot.devices.monitor,
                        self.panels.monitor_split,
                        self.hex_popup_view_state(),
                        self.preferences.lang
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if self.panels.hdd_open && !self.panels.hdd_window.detached() {
                stack![
                    scrimmed,
                    hdd_window_overlay(
                        &self.snapshot.devices.hdd,
                        self.panels.hdd_file_exists,
                        self.panels.hdd_show_image_contents,
                        &self.panels.hdd_image_contents,
                        self.panels.hdd_image_error.as_deref(),
                        self.preferences.lang,
                        self.device_toolbar(crate::app::ToolWindowKind::Hdd),
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if self.panels.floppy_open && !self.panels.floppy_window.detached() {
                stack![
                    scrimmed,
                    floppy_window_overlay(
                        &self.snapshot.devices.floppy,
                        self.panels.floppy_show_image_contents,
                        &self.panels.floppy_image_contents,
                        self.panels.floppy_image_error.as_deref(),
                        self.preferences.lang,
                        self.device_toolbar(crate::app::ToolWindowKind::Floppy),
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else if self.panels.network_open && !self.panels.network_window.detached() {
                stack![scrimmed, network_window_overlay(self.network_view_state())]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else if self.panels.printer_open && !self.panels.printer_window.detached() {
                stack![
                    scrimmed,
                    printer_window_overlay(
                        &self.snapshot.devices.printer,
                        self.panels.printer_text_view,
                        self.printer_target_label(),
                        self.preferences.lang,
                        self.device_toolbar(crate::app::ToolWindowKind::Printer),
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                stack![scrimmed]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            };

        with_settings_notice(
            with_printer_setup_overlay(
                layered,
                self.printer_setup
                    .printer_setup_dialog
                    .as_ref()
                    .filter(|_| !self.printer_setup_uses_detached_window()),
                self.preferences.lang,
            ),
            self.preferences.settings_notice,
            self.preferences.lang,
        )
    }
}

fn menu_dropdown_overlay(dropdown: Element<'_, Message>, left: f32) -> Element<'_, Message> {
    column![
        Space::new().height(Length::Fixed(MENU_DROPDOWN_TOP)),
        row![
            Space::new().width(Length::Fixed(left)),
            opaque(dropdown),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
