use super::super::locale::{Locale, Text as T};
use super::super::operations::InstallReport;
use super::super::{Installer, Message, style, widgets};
use iced::widget::{Space, column, container, progress_bar, row, text};
use iced::{Alignment, Element, Length};
use k580_ui::install_mode::InstallMode;

pub(super) fn activity_panel(app: &Installer) -> Option<Element<'_, Message>> {
    match app.result.as_ref() {
        Some(Err(error)) => Some(error_panel(app.locale, error)),
        None if app.installing => Some(installing_panel(app)),
        _ => None,
    }
}

pub(super) fn completion_panel(
    report: &InstallReport,
    locale: Locale,
) -> Element<'static, Message> {
    let (path_state, path_color) = if report.path_changed {
        (locale.t(T::TerminalLaunchEnabled), style::TEXT)
    } else {
        (locale.t(T::TerminalLaunchUnchanged), style::MUTED)
    };
    let (association_state, association_color) = if report.file_association_created {
        (locale.t(T::FileAssociationCreated), style::TEXT)
    } else {
        (locale.t(T::FileAssociationUnchanged), style::MUTED)
    };

    column![
        text(locale.t(T::Installed))
            .font(style::FONT_BOLD)
            .size(24)
            .color(style::GREEN),
        text(format!(
            "{}: {}",
            locale.t(T::Location),
            report.install_dir.display()
        ))
        .font(style::MONO_FONT)
        .size(13)
        .color(style::BLUE)
        .width(Length::Fill),
        widgets::horizontal_rule(),
        report_line(system_state(report, locale), style::TEXT),
        report_line(association_state, association_color),
        report_line(path_state, path_color),
    ]
    .spacing(12)
    .into()
}

pub(super) fn post_install_action(app: &Installer, mode: InstallMode) -> Element<'_, Message> {
    let action_label = match mode {
        InstallMode::Portable => app.locale.t(T::OpenInstallationFolder),
        InstallMode::System => app.locale.t(T::LaunchKr580),
    };
    let mut content = column![widgets::checkbox(
        app.post_install_action,
        action_label,
        Message::PostInstallActionToggled,
    ),]
    .spacing(8);

    if let Some(error) = app.post_install_error.as_deref() {
        content = content.push(
            text(error)
                .font(style::FONT)
                .size(12)
                .color(style::RED)
                .width(Length::Fill),
        );
    }
    content.into()
}

pub(super) fn command_bar(app: &Installer) -> Element<'_, Message> {
    let (label, message) = match app.result.as_ref() {
        Some(Ok(_)) => (app.locale.t(T::Done), Some(Message::DonePressed)),
        _ if app.installing => (app.locale.t(T::InstallingEllipsis), None),
        _ => (app.locale.t(T::InstallKr580), Some(Message::InstallPressed)),
    };

    widgets::command_bar(label, message)
}

fn installing_panel(app: &Installer) -> Element<'_, Message> {
    container(
        column![
            row![
                text(app.locale.t(T::Installing))
                    .font(style::FONT_BOLD)
                    .size(14)
                    .color(style::BLUE),
                Space::new().width(Length::Fill),
                text(format!("{:02}%", (app.install_progress * 100.0) as u8))
                    .font(style::MONO_FONT)
                    .size(12)
                    .color(style::MUTED),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=1.0, app.install_progress)
                .girth(Length::Fixed(7.0))
                .style(style::progress),
        ]
        .spacing(8),
    )
    .padding(12)
    .style(style::group_frame)
    .width(Length::Fill)
    .into()
}

fn error_panel(locale: Locale, error: &str) -> Element<'_, Message> {
    container(
        column![
            text(locale.t(T::InstallFailed))
                .font(style::FONT_BOLD)
                .size(14)
                .color(style::RED),
            text(error)
                .font(style::FONT)
                .size(12)
                .color(style::TEXT)
                .width(Length::Fill),
        ]
        .spacing(5),
    )
    .padding(12)
    .style(style::group_frame)
    .width(Length::Fill)
    .into()
}

fn report_line(value: &'static str, color: iced::Color) -> Element<'static, Message> {
    row![
        text("●").font(style::FONT).size(10).color(style::GREEN),
        text(value).font(style::FONT).size(13).color(color),
    ]
    .spacing(9)
    .align_y(Alignment::Center)
    .into()
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
