use super::super::locale::Text as T;
use super::super::{Installer, Message, style};
use super::finish;
use iced::widget::{Space, button, checkbox, column, container, row, svg, text, text_input};
use iced::{Alignment, Element, Length, alignment};
use k580_ui::install_mode::InstallMode;
#[cfg(windows)]
use k580_ui::install_mode::InstallScope;
use std::sync::LazyLock;

const OPTION_HEIGHT: f32 = 64.0;
const FIELD_HEIGHT: f32 = 44.0;
const BROWSE_HEIGHT: f32 = 42.0;
const BROWSE_WIDTH: f32 = 96.0;
const PANEL_PADDING: f32 = 18.0;
const BUTTON_ICON_SIZE: f32 = 14.0;

macro_rules! action_icon_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/icons/actions/", $name, ".svg"))
    };
}

static FOLDER_OPEN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("folder-open").as_slice()));

pub fn content(app: &Installer) -> Element<'_, Message> {
    container(right_panel(app))
        .padding(14)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn right_panel(app: &Installer) -> Element<'_, Message> {
    let locale = app.locale();
    let mut controls = column![
        header(app),
        section_label(locale.t(T::Mode)),
        row![
            option_button(
                locale.t(T::System),
                locale.t(T::SystemCaption),
                app.mode() == InstallMode::System,
                Message::ModeSelected(InstallMode::System),
                false,
            ),
            option_button(
                locale.t(T::Portable),
                locale.t(T::PortableCaption),
                app.mode() == InstallMode::Portable,
                Message::ModeSelected(InstallMode::Portable),
                false,
            ),
        ]
        .spacing(10),
    ];

    if app.mode() == InstallMode::System {
        controls = controls.push(scope_section(app));
    }

    controls = controls
        .push(section_label(locale.t(T::Folder)))
        .push(folder_row(app))
        .push(
            checkbox(app.add_to_path())
                .label(locale.t(T::AddKrPath))
                .on_toggle(Message::AddToPathToggled)
                .font(style::FONT)
                .text_size(15)
                .size(18)
                .spacing(10)
                .style(style::check),
        );

    if app.mode() == InstallMode::System {
        controls = controls.push(desktop_shortcut_checkbox(app));
    }

    controls = controls
        .push(file_association_checkbox(app))
        .push(finish::result_panel(app));

    if let Some(action) = finish::post_install_action(app) {
        controls = controls
            .push(Space::new().height(Length::FillPortion(1)))
            .push(action)
            .push(Space::new().height(Length::FillPortion(1)));
    } else {
        controls = controls.push(Space::new().height(Length::Fill));
    }

    controls = controls
        .push(finish::bottom_action(app))
        .spacing(9)
        .padding(PANEL_PADDING);

    container(controls)
        .style(style::panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn desktop_shortcut_checkbox(app: &Installer) -> Element<'_, Message> {
    checkbox(app.create_desktop_shortcut())
        .label(app.locale().t(T::DesktopShortcut))
        .on_toggle(Message::DesktopShortcutToggled)
        .font(style::FONT)
        .text_size(15)
        .size(18)
        .spacing(10)
        .style(style::check)
        .into()
}

fn file_association_checkbox(app: &Installer) -> Element<'_, Message> {
    checkbox(app.associate_580_files())
        .label(app.locale().t(T::Associate580))
        .on_toggle(Message::FileAssociationToggled)
        .font(style::FONT)
        .text_size(15)
        .size(18)
        .spacing(10)
        .style(style::check)
        .into()
}

fn header(app: &Installer) -> Element<'_, Message> {
    let locale = app.locale();
    column![
        text(locale.t(T::InstallerTitle))
            .font(style::FONT_BOLD)
            .size(28)
            .color(style::TEXT),
        text(locale.t(T::InstallerSubtitle))
            .font(style::FONT)
            .size(15)
            .color(style::MUTED),
    ]
    .spacing(4)
    .into()
}

fn scope_section(app: &Installer) -> Element<'_, Message> {
    #[cfg(windows)]
    {
        column![
            section_label(app.locale().t(T::WindowsScope)),
            row![
                option_button(
                    app.locale().t(T::CurrentUser),
                    app.locale().t(T::NoElevation),
                    app.scope() == InstallScope::User,
                    Message::ScopeSelected(InstallScope::User),
                    app.mode() == InstallMode::Portable,
                ),
                option_button(
                    app.locale().t(T::AllUsers),
                    app.locale().t(T::MachinePath),
                    app.scope() == InstallScope::Machine,
                    Message::ScopeSelected(InstallScope::Machine),
                    app.mode() == InstallMode::Portable,
                ),
            ]
            .spacing(10),
        ]
        .spacing(8)
        .into()
    }
    #[cfg(not(windows))]
    {
        container(
            row![
                text(app.locale().t(T::Scope))
                    .font(style::FONT_BOLD)
                    .size(14)
                    .color(style::WHITE),
                Space::new().width(Length::Fixed(8.0)),
                text(app.locale().t(T::UserInstall))
                    .font(style::FONT)
                    .size(14)
                    .color(style::TEXT),
            ]
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(style::soft_panel)
        .width(Length::Fill)
        .into()
    }
}

fn folder_row(app: &Installer) -> Element<'_, Message> {
    row![
        container(
            text_input(app.locale().t(T::InstallationFolder), app.install_dir())
                .on_input(Message::InstallDirChanged)
                .padding(12)
                .size(15)
                .font(style::FONT)
                .style(style::input),
        )
        .width(Length::Fill)
        .height(Length::Fixed(FIELD_HEIGHT))
        .align_y(alignment::Vertical::Center),
        button(
            container(
                row![
                    svg(FOLDER_OPEN.clone())
                        .width(Length::Fixed(BUTTON_ICON_SIZE))
                        .height(Length::Fixed(BUTTON_ICON_SIZE))
                        .style(|_theme, _status| svg::Style {
                            color: Some(style::TEXT),
                        }),
                    text(app.locale().t(T::Browse))
                        .font(style::FONT_BOLD)
                        .size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        )
        .padding(0)
        .width(Length::Fixed(BROWSE_WIDTH))
        .height(Length::Fixed(BROWSE_HEIGHT))
        .style(style::neutral_button)
        .on_press(Message::BrowseInstallDir),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn option_button<'a>(
    title: &'a str,
    caption: &'a str,
    selected: bool,
    message: Message,
    disabled: bool,
) -> Element<'a, Message> {
    let content = container(
        column![
            text(title)
                .font(style::FONT_BOLD)
                .size(14)
                .color(if selected { style::WHITE } else { style::TEXT }),
            text(caption).font(style::FONT).size(12).color(style::MUTED),
        ]
        .spacing(3)
        .align_x(Alignment::Start),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(alignment::Vertical::Center);

    button(content)
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fixed(OPTION_HEIGHT))
        .style(if selected {
            style::selected_button
        } else {
            style::neutral_button
        })
        .on_press_maybe((!disabled).then_some(message))
        .into()
}

fn section_label<'a>(label: &'a str) -> Element<'a, Message> {
    text(label)
        .font(style::FONT_BOLD)
        .size(13)
        .color(style::WHITE)
        .into()
}
