//! Top-of-window menu strip plus the floating dropdown panels.
//!
//! The bar itself is a flat `row` of clickable labels; "Файл" and
//! "МП-Система" each toggle a dropdown panel that the root `view`
//! overlays on top of the rest of the UI. iced 0.14 has no first-class
//! menu widget, so we lean on `mouse_area` + `button` and a tiny bit of
//! state in `DesktopApp` (`open_menu`) to drive visibility. Every
//! dropdown entry is a flat action — submenus would just add a second
//! click on top of the format selection that the OS file picker already
//! handles, and on the МП-Система side every entry is a single
//! emulator command anyway.

use iced::widget::{Space, button, column, container, mouse_area, row, svg};
use iced::{Element, Length, alignment};

use super::icons;
use super::styles::{menu_bar_style, menu_button_style, opcode_dropdown_style};
use super::theme::{TOKYO_BORDER, TOKYO_MAGENTA, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::{DesktopApp, MenuId, Message};

/// Width of the floating "Файл" dropdown. Picked wide enough that
/// "Сохранить как" + the longest hint ("Ctrl+Shift+S") sit on one
/// line at 13 px without wrapping, with room for the leading 16 px
/// glyph and its gap.
const FILE_DROPDOWN_WIDTH: f32 = 260.0;

/// Width of the "МП-Система" dropdown. Tuned for the longest label
/// here ("Очистить регистры") plus the longest shortcut hint
/// ("Ctrl+Shift+R") so the row fits on one line next to the 16 px
/// glyph without wrapping.
const MP_DROPDOWN_WIDTH: f32 = 270.0;

/// Edge length of the icon square that prefixes every dropdown row.
/// 16 px reads as "menu glyph" — small enough to not compete with the
/// label, large enough to remain legible at 100 % DPI.
const MENU_ICON_SIZE: f32 = 16.0;

impl DesktopApp {
    pub(super) fn menu_bar(&self) -> Element<'_, Message> {
        container(
            row![
                ui_text("Эмулятор KR580VM80A", 14, TOKYO_MAGENTA),
                menu_trigger("Файл", MenuId::File, self.open_menu == Some(MenuId::File)),
                menu_trigger("МП-Система", MenuId::Mp, self.open_menu == Some(MenuId::Mp),),
                menu_label("View"),
                menu_label("Settings"),
                menu_label("Help"),
            ]
            .spacing(18)
            .align_y(alignment::Vertical::Center),
        )
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(menu_bar_style)
        .into()
    }

    /// Builds the floating dropdown for whichever top-level menu is
    /// currently open, or `None` when the menu bar is at rest.
    /// Composed as a `column` so the root `view` can stack it on top
    /// of the rest of the UI at a fixed offset (just below the menu
    /// bar) without disturbing layout.
    pub(super) fn menu_dropdown(&self) -> Option<Element<'_, Message>> {
        match self.open_menu? {
            MenuId::File => Some(file_dropdown()),
            MenuId::Mp => Some(mp_dropdown()),
        }
    }
}

/// A non-clickable label for the menus we have not wired up yet (View,
/// Settings, Help). They keep their place in the bar so the visual
/// rhythm matches the reference emulator's chrome, even though clicking
/// them is currently a no-op.
fn menu_label(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 13, TOKYO_TEXT).into()
}

/// A clickable top-level menu label. The label itself is wrapped in a
/// `mouse_area` with `Pointer` interaction so the cursor signals
/// affordance, and the press dispatches `MenuToggled` for the menu we
/// own. When the menu is open the label tints to magenta so the user
/// can see at a glance which dropdown is currently visible — mirroring
/// the convention used by every native menu bar.
fn menu_trigger(label: &'static str, menu: MenuId, active: bool) -> Element<'static, Message> {
    let color = if active { TOKYO_MAGENTA } else { TOKYO_TEXT };
    mouse_area(ui_text(label, 13, color))
        .on_press(Message::MenuToggled(menu))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

/// Renders the actual "Файл" dropdown column. Both "Импорт" and
/// "Экспорт" are flat rows: each one opens a single OS file dialog
/// where the user picks the format via the file extension, so a
/// submenu inside the app would just duplicate that choice.
///
/// Each row carries a faint right-aligned shortcut hint so the user
/// can pick up the keyboard binding without having to consult a
/// help page. The hints mirror the actual handlers in the keyboard
/// subscription — see `DesktopApp::subscription` in `app/mod.rs`.
fn file_dropdown() -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = vec![
        menu_item("Новый файл", "Ctrl+N", icons::file(), Message::NewFile),
        menu_item(
            "Открыть",
            "Ctrl+O",
            icons::folder_open(),
            Message::OpenSnapshot,
        ),
        menu_item("Сохранить", "Ctrl+S", icons::save(), Message::SaveSnapshot),
        menu_item(
            "Сохранить как",
            "Ctrl+Shift+S",
            icons::save_as(),
            Message::SaveSnapshotAs,
        ),
        menu_item("Импорт", "Ctrl+I", icons::file_down(), Message::Import),
        menu_item("Экспорт", "Ctrl+E", icons::file_up(), Message::Export),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(FILE_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

/// Renders the "МП-Система" dropdown column. The three execution
/// gestures (run / step instruction / step tact) sit at the top, then a
/// thin separator, then the two reset gestures. Each row carries a
/// Ctrl-letter shortcut hint mirroring the actual handler in the
/// keyboard subscription — see `DesktopApp::subscription` in
/// `app/mod.rs`. Bindings: `Ctrl+R` runs the program (R = Run),
/// `Ctrl+T` steps one instruction, `Ctrl+Y` steps one tact (T and Y
/// sit next to each other on both QWERTY and ЙЦУКЕН so the pair
/// reads as "instruction → finer-grained tact"). The destructive
/// resets sit on `Ctrl+Shift+R` (RAM) and `Ctrl+Shift+G` (reGisters)
/// so an accidental modifier slip while typing in the address field
/// can't blow the program away.
fn mp_dropdown() -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = vec![
        menu_item(
            "Выполнить программу",
            "Ctrl+R",
            icons::play(),
            Message::ToggleRun,
        ),
        menu_item(
            "Выполнить команду",
            "Ctrl+T",
            icons::step_forward(),
            Message::StepInstruction,
        ),
        menu_item(
            "Выполнить такт",
            "Ctrl+Y",
            icons::redo_dot(),
            Message::StepTact,
        ),
        menu_separator(),
        menu_item(
            "Очистить ОЗУ",
            "Ctrl+Shift+R",
            icons::reset_ram(),
            Message::ResetRam,
        ),
        menu_item(
            "Очистить регистры",
            "Ctrl+Shift+G",
            icons::reset_registers(),
            Message::ResetCpu,
        ),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(MP_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

/// One actionable row inside a dropdown. Closing the menu *first* and
/// then dispatching the actual action via `Task::done(action)` keeps
/// the dropdown from sticking around behind a file dialog when the
/// dispatched action opens one — the user sees the menu close as soon
/// as they click, not after the dialog returns.
///
/// The row layout is `[icon] [label]  …  [shortcut]`: a 16 px tinted
/// SVG glyph on the left, the label spaced out with the same horizontal
/// gap used elsewhere in the editor chrome, a flexible spacer that
/// pushes the shortcut hint to the right edge, and the shortcut itself
/// rendered in `TOKYO_MUTED` so it reads as supplementary information
/// rather than a competing label.
fn menu_item(
    label: &'static str,
    shortcut: &'static str,
    icon: svg::Handle,
    action: Message,
) -> Element<'static, Message> {
    let glyph = svg(icon)
        .width(Length::Fixed(MENU_ICON_SIZE))
        .height(Length::Fixed(MENU_ICON_SIZE))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_TEXT),
        });

    let body = container(
        row![
            glyph,
            ui_text(label, 13, TOKYO_TEXT),
            Space::new().width(Length::Fill),
            ui_text(shortcut, 11, TOKYO_MUTED),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .align_y(alignment::Vertical::Center);

    let pair = vec![Message::MenuClosed, action];
    button(body)
        .on_press(Message::MenuBatch(pair))
        .padding(0)
        .width(Length::Fill)
        .style(move |_theme, status| menu_button_style(status))
        .into()
}

/// Visual divider between two groups of dropdown entries. iced 0.14 has
/// no native `<hr>`, so we render a 1-pixel-tall full-width container
/// painted in a low-alpha tint of the same border hue the dropdown
/// surface uses on its outline. The reduced alpha keeps the rule
/// readable as a hint without competing with the row labels — at full
/// `TOKYO_BORDER` strength the separator visually outranked the items
/// it was meant to group. A few pixels of vertical padding above and
/// below give the rule breathing room so it does not collide with the
/// glyphs of the adjacent rows.
fn menu_separator() -> Element<'static, Message> {
    container(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.35,
                    ..TOKYO_BORDER
                })),
                ..iced::widget::container::Style::default()
            }),
    )
    .padding([4, 8])
    .width(Length::Fill)
    .into()
}
