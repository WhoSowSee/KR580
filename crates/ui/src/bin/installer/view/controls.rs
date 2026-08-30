use super::super::locale::Text as T;
use super::super::{Installer, Message, style, widgets};
use iced::widget::{Space, button, column, container, row, stack, svg, text, text_input};
use iced::{Alignment, Element, Length, alignment};
use k580_ui::install_mode::InstallMode;
use std::sync::LazyLock;

const CONTROL_HEIGHT: f32 = 44.0;
const BROWSE_WIDTH: f32 = 96.0;
const INDICATOR_WIDTH: f32 = 3.0;
const RADIO_SIZE: f32 = 12.0;
const RADIO_DOT_SIZE: f32 = 6.0;
const INTEGRATION_SPACING: f32 = 12.0;
const SEGMENTED_PADDING: iced::Padding = iced::Padding {
    top: 1.0,
    right: 1.0,
    bottom: 1.0,
    left: 0.0,
};

static FOLDER_OPEN: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("folder-open").as_slice()));

pub(super) fn mode_section(app: &Installer) -> Element<'_, Message> {
    let locale = app.locale;
    section(
        locale.t(T::Mode),
        segmented_group(
            option_button(
                locale.t(T::System),
                app.mode == InstallMode::System,
                Message::ModeSelected(InstallMode::System),
            ),
            option_button(
                locale.t(T::Portable),
                app.mode == InstallMode::Portable,
                Message::ModeSelected(InstallMode::Portable),
            ),
            app.mode == InstallMode::Portable,
        ),
    )
}

pub(super) fn scope_section(app: &Installer) -> Element<'_, Message> {
    #[cfg(windows)]
    {
        use k580_ui::install_mode::InstallScope;

        let locale = app.locale;
        section(
            locale.t(T::WindowsScope),
            segmented_group(
                option_button(
                    locale.t(T::CurrentUser),
                    app.scope == InstallScope::User,
                    Message::ScopeSelected(InstallScope::User),
                ),
                option_button(
                    locale.t(T::AllUsers),
                    app.scope == InstallScope::Machine,
                    Message::ScopeSelected(InstallScope::Machine),
                ),
                app.scope == InstallScope::Machine,
            ),
        )
    }
    #[cfg(not(windows))]
    {
        section(
            app.locale.t(T::Scope),
            container(
                text(app.locale.t(T::UserInstall))
                    .font(style::FONT)
                    .size(14)
                    .color(style::TEXT),
            )
            .height(Length::Fixed(CONTROL_HEIGHT))
            .padding(12)
            .style(|_| style::segmented_group(false))
            .width(Length::Fill)
            .into(),
        )
    }
}

pub(super) fn folder_section(app: &Installer) -> Element<'_, Message> {
    let locale = app.locale;
    let input = text_input(locale.t(T::InstallationFolder), &app.install_dir)
        .on_input(Message::InstallDirChanged)
        .padding(12)
        .size(14)
        .font(style::MONO_FONT)
        .style(style::joined_input);
    let browse = button(
        container(
            row![
                svg(FOLDER_OPEN.clone())
                    .width(Length::Fixed(15.0))
                    .height(Length::Fixed(15.0))
                    .style(|_theme, _status| svg::Style {
                        color: Some(style::TEXT),
                    }),
                text(locale.t(T::Browse)).font(style::FONT_BOLD).size(13),
            ]
            .spacing(7)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .height(Length::Fill)
    .width(Length::Fixed(BROWSE_WIDTH))
    .style(style::joined_button)
    .on_press(Message::BrowseInstallDir);

    section(
        locale.t(T::Folder),
        container(
            row![
                container(input)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_y(alignment::Vertical::Center),
                widgets::vertical_rule(),
                browse,
            ]
            .spacing(0)
            .align_y(Alignment::Center),
        )
        .height(Length::Fixed(CONTROL_HEIGHT))
        .padding(1)
        .style(style::group_frame)
        .width(Length::Fill)
        .into(),
    )
}

pub(super) fn integration_section(app: &Installer) -> Element<'_, Message> {
    let mut options = column![
        widgets::checkbox(
            app.add_to_path,
            app.locale.t(T::AddKrPath),
            Message::AddToPathToggled,
        ),
        widgets::checkbox(
            app.associate_program_files,
            app.locale.t(T::AssociateProgramFiles),
            Message::FileAssociationToggled,
        ),
    ]
    .spacing(INTEGRATION_SPACING);
    if app.mode == InstallMode::System {
        options = options.push(widgets::checkbox(
            app.create_desktop_shortcut,
            app.locale.t(T::DesktopShortcut),
            Message::DesktopShortcutToggled,
        ));
    }
    section(app.locale.t(T::Integration), options.into())
}

fn section<'a>(label: &'a str, body: Element<'a, Message>) -> Element<'a, Message> {
    column![
        text(label)
            .font(style::FONT_BOLD)
            .size(13)
            .color(style::MUTED),
        body,
    ]
    .spacing(7)
    .into()
}

fn segmented_group<'a>(
    left: Element<'a, Message>,
    right: Element<'a, Message>,
    right_selected: bool,
) -> Element<'a, Message> {
    let segments = container(
        row![left, segment_divider(right_selected), right]
            .spacing(0)
            .align_y(Alignment::Center),
    )
    .height(Length::Fixed(CONTROL_HEIGHT))
    .padding(SEGMENTED_PADDING)
    .style(move |_| style::segmented_group(right_selected))
    .width(Length::Fill);

    if right_selected {
        let frame = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::segmented_frame_overlay);
        stack![segments, frame]
            .width(Length::Fill)
            .height(Length::Fixed(CONTROL_HEIGHT))
            .into()
    } else {
        segments.into()
    }
}

fn segment_divider(right_selected: bool) -> Element<'static, Message> {
    if right_selected {
        Space::new()
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .into()
    } else {
        widgets::vertical_rule()
    }
}

fn option_button<'a>(label: &'a str, selected: bool, message: Message) -> Element<'a, Message> {
    let indicator = container(Space::new())
        .width(Length::Fixed(INDICATOR_WIDTH))
        .height(Length::Fill)
        .style(move |_| {
            if selected {
                style::indicator_rail(style::BLUE, true)
            } else {
                container::Style::default()
            }
        });
    let radio_dot: Element<'static, Message> = if selected {
        container(Space::new())
            .width(Length::Fixed(RADIO_DOT_SIZE))
            .height(Length::Fixed(RADIO_DOT_SIZE))
            .style(style::radio_dot)
            .into()
    } else {
        Space::new()
            .width(Length::Fixed(RADIO_DOT_SIZE))
            .height(Length::Fixed(RADIO_DOT_SIZE))
            .into()
    };
    let radio = container(radio_dot)
        .width(Length::Fixed(RADIO_SIZE))
        .height(Length::Fixed(RADIO_SIZE))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |_| style::radio_indicator(selected));
    let content = row![indicator, radio, text(label).font(style::FONT).size(14)]
        .spacing(12)
        .align_y(Alignment::Center);

    button(content)
        .padding(iced::Padding::ZERO.right(12))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme, status| style::segment_button(status))
        .on_press(message)
        .into()
}
