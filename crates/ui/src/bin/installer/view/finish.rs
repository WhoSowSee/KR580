use super::super::locale::{Locale, Text as T};
use super::super::operations::InstallReport;
use super::super::{Installer, Message, style};
use iced::widget::{button, checkbox, column, container, progress_bar, text};
use iced::{Element, Length, alignment};
use k580_ui::install_mode::InstallMode;

const INSTALL_HEIGHT: f32 = 48.0;

pub fn result_panel(app: &Installer) -> Element<'_, Message> {
    let body = match app.result() {
        Some(Ok(report)) => success_panel(report, app.locale()),
        Some(Err(error)) => message_panel(app.locale().t(T::InstallFailed), error, style::WHITE),
        None if app.installing() => installing_panel(app),
        None => message_panel(
            app.locale().t(T::Ready),
            app.locale().t(T::ReadyBody),
            style::MUTED,
        ),
    };

    container(body).width(Length::Fill).into()
}

pub fn bottom_action(app: &Installer) -> Element<'_, Message> {
    match app.result() {
        Some(Ok(_)) => done_button(app.locale()),
        _ => install_button(app),
    }
}

pub fn post_install_action(app: &Installer) -> Option<Element<'_, Message>> {
    let Some(Ok(report)) = app.result() else {
        return None;
    };

    let action_label = match report.mode {
        InstallMode::Portable => app.locale().t(T::OpenInstallationFolder),
        InstallMode::System => app.locale().t(T::LaunchKr580),
    };
    let mut content = column![
        checkbox(app.post_install_action())
            .label(action_label)
            .on_toggle(Message::PostInstallActionToggled)
            .font(style::FONT)
            .text_size(15)
            .size(18)
            .spacing(10)
            .style(style::check),
    ]
    .spacing(8);

    if let Some(error) = app.post_install_error() {
        content = content.push(
            text(error)
                .font(style::FONT)
                .size(12)
                .color(style::WHITE)
                .width(Length::Fill),
        );
    }

    Some(content.into())
}

fn installing_panel(app: &Installer) -> Element<'_, Message> {
    container(
        column![
            text(app.locale().t(T::Installing))
                .font(style::FONT_BOLD)
                .size(15)
                .color(style::WHITE),
            text(app.locale().t(T::InstallingBody))
                .font(style::FONT)
                .size(12)
                .color(style::MUTED),
            progress_bar(0.0..=1.0, app.install_progress())
                .girth(Length::Fixed(8.0))
                .style(style::progress),
        ]
        .spacing(8),
    )
    .padding(12)
    .style(style::soft_panel)
    .width(Length::Fill)
    .into()
}

fn success_panel(report: &InstallReport, locale: Locale) -> Element<'_, Message> {
    let path_state = if report.path_changed {
        locale.t(T::TerminalLaunchEnabled)
    } else {
        locale.t(T::TerminalLaunchUnchanged)
    };

    container(
        column![
            text(locale.t(T::Installed))
                .font(style::FONT_BOLD)
                .size(15)
                .color(style::WHITE),
            text(format!(
                "{}: {}",
                locale.t(T::Location),
                report.install_dir.display()
            ))
            .font(style::FONT)
            .size(12)
            .color(style::TEXT),
            text(system_state(report, locale))
                .font(style::FONT)
                .size(12)
                .color(style::TEXT),
            text(file_association_state(report, locale))
                .font(style::FONT)
                .size(12)
                .color(style::MUTED),
            text(path_state)
                .font(style::FONT)
                .size(12)
                .color(style::MUTED),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(style::soft_panel)
    .width(Length::Fill)
    .into()
}

fn file_association_state(report: &InstallReport, locale: Locale) -> &'static str {
    if report.file_association_created {
        locale.t(T::FileAssociationCreated)
    } else {
        locale.t(T::FileAssociationUnchanged)
    }
}

fn system_state(report: &InstallReport, locale: Locale) -> &'static str {
    if !report.system_integrated {
        return locale.t(T::PortableReady);
    }
    if report.desktop_shortcut_created {
        locale.t(T::SearchDesktopUninstallReady)
    } else {
        locale.t(T::SearchUninstallReady)
    }
}

fn message_panel<'a>(title: &'a str, body: &'a str, color: iced::Color) -> Element<'a, Message> {
    container(
        column![
            text(title).font(style::FONT_BOLD).size(16).color(color),
            text(body)
                .font(style::FONT)
                .size(13)
                .color(style::MUTED)
                .width(Length::Fill),
        ]
        .spacing(6),
    )
    .padding(12)
    .style(style::soft_panel)
    .width(Length::Fill)
    .into()
}

fn install_button(app: &Installer) -> Element<'_, Message> {
    let label = if app.installing() {
        app.locale().t(T::InstallingEllipsis)
    } else {
        app.locale().t(T::InstallKr580)
    };

    button_text(
        label,
        (!app.installing()).then_some(Message::InstallPressed),
    )
}

fn done_button(locale: Locale) -> Element<'static, Message> {
    button_text(locale.t(T::Done), Some(Message::DonePressed))
}

fn button_text<'a>(label: &'a str, message: Option<Message>) -> Element<'a, Message> {
    button(
        container(
            text(label)
                .font(style::FONT_BOLD)
                .size(16)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fixed(INSTALL_HEIGHT))
    .style(style::primary_button)
    .on_press_maybe(message)
    .into()
}
