use super::super::operations::InstallReport;
use super::super::{Installer, Message, widgets};
use super::{controls, finish, rail};
use iced::widget::{Space, column, container, row};
use iced::{Element, Length};
use k580_ui::install_mode::InstallMode;

const CONTENT_PADDING: f32 = 16.0;
const SECTION_SPACING: f32 = 16.0;

pub(super) fn content(app: &Installer) -> Element<'_, Message> {
    column![content_row(app), finish::command_bar(app)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn content_row(app: &Installer) -> Element<'_, Message> {
    let work = match app.result.as_ref() {
        Some(Ok(report)) => completed_work(app, report),
        _ => setup_work(app),
    };

    row![rail::rail(), widgets::vertical_rule(), work]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn setup_work(app: &Installer) -> Element<'_, Message> {
    let mut form = column![controls::mode_section(app)].spacing(SECTION_SPACING);
    if app.mode == InstallMode::System {
        form = form.push(controls::scope_section(app));
    }
    form = form
        .push(controls::folder_section(app))
        .push(controls::integration_section(app));

    if let Some(activity) = finish::activity_panel(app) {
        form = form.push(activity);
    }

    container(form)
        .padding(CONTENT_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn completed_work<'a>(app: &'a Installer, report: &InstallReport) -> Element<'a, Message> {
    container(
        column![
            finish::completion_panel(report, app.locale),
            finish::post_install_action(app, report.mode),
            Space::new().height(Length::Fill),
        ]
        .spacing(16),
    )
    .padding(CONTENT_PADDING)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
