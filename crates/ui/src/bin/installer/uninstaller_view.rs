use super::super::locale::Text as T;
use super::super::{style, widgets, window_chrome};
use super::{Message, StageState, UninstallStage, Uninstaller};
use iced::widget::{Space, column, container, progress_bar, row, svg, text};
use iced::{Alignment, Element, Length, alignment};

const STAGE_HEIGHT: f32 = 60.0;

pub(super) fn view(app: &Uninstaller) -> Element<'_, Message> {
    column![
        window_chrome::title_bar(
            app.locale.t(T::WindowTitleUninstaller),
            app.window_maximized,
            window_chrome::CaptionMessages {
                drag: Message::WindowDragStart,
                minimize: Message::WindowMinimize,
                toggle_maximize: Message::WindowToggleMaximize,
                close: app.can_close().then_some(Message::WindowClose),
            },
        ),
        content(app),
        command_bar(app),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn content(app: &Uninstaller) -> Element<'_, Message> {
    container(
        column![
            header(),
            widgets::horizontal_rule(),
            path(app),
            progress_panel(app)
        ]
        .spacing(16)
        .width(Length::Fill),
    )
    .padding([18, 24])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn header() -> Element<'static, Message> {
    container(
        column![
            svg(window_chrome::CHIP.clone())
                .width(Length::Fixed(84.0))
                .height(Length::Fixed(66.0))
                .style(|_theme, _status| svg::Style {
                    color: Some(style::TEXT),
                }),
            text("КР580")
                .font(style::FONT_BOLD)
                .size(27)
                .color(style::TEXT),
            text(format!(
                "{} · {}",
                env!("CARGO_PKG_VERSION"),
                window_chrome::platform_label()
            ))
            .font(style::MONO_FONT)
            .size(12)
            .color(style::MUTED),
        ]
        .spacing(6)
        .align_x(Alignment::Center),
    )
    .height(Length::Fixed(120.0))
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn path(app: &Uninstaller) -> Element<'_, Message> {
    column![
        text(app.locale.t(T::Folder))
            .font(style::FONT_BOLD)
            .size(13)
            .color(style::MUTED),
        container(
            text(app.install_dir.display().to_string())
                .font(style::MONO_FONT)
                .size(13)
                .color(style::BLUE)
                .width(Length::Fill),
        )
        .padding([14, 16])
        .height(Length::Fixed(48.0))
        .style(style::group_frame)
        .width(Length::Fill),
    ]
    .spacing(7)
    .into()
}

fn progress_panel(app: &Uninstaller) -> Element<'_, Message> {
    let (status, status_color) = app.status();
    let stages = row![
        stage_cell(app, UninstallStage::System, "01", T::UninstallStageSystem),
        widgets::vertical_rule(),
        stage_cell(app, UninstallStage::Links, "02", T::UninstallStageLinks),
        widgets::vertical_rule(),
        stage_cell(app, UninstallStage::Files, "03", T::UninstallStageFiles),
    ]
    .spacing(0)
    .height(Length::Fixed(STAGE_HEIGHT));

    let progress = column![
        row![
            text(status).font(style::FONT).size(13).color(status_color),
            Space::new().width(Length::Fill),
            text(format!(
                "{:02}%",
                (app.display_progress * 100.0).round() as u8
            ))
            .font(style::MONO_FONT)
            .size(13)
            .color(style::BLUE),
        ]
        .align_y(Alignment::Center),
        progress_bar(0.0..=1.0, app.display_progress)
            .girth(Length::Fixed(7.0))
            .style(style::progress),
    ]
    .spacing(10)
    .padding(14);

    container(column![stages, widgets::horizontal_rule(), progress].spacing(0))
        .style(style::group_frame)
        .width(Length::Fill)
        .into()
}

fn stage_cell(
    app: &Uninstaller,
    stage: UninstallStage,
    number: &'static str,
    label: T,
) -> Element<'static, Message> {
    let state = app.stage_state(stage);
    let (accent, label_color) = match state {
        StageState::Complete => (style::GREEN, style::TEXT),
        StageState::Active => (style::BLUE, style::TEXT),
        StageState::Failed => (style::RED, style::TEXT),
        StageState::Pending => (style::MUTED, style::MUTED),
    };
    let rail: Element<'static, Message> =
        if matches!(state, StageState::Active | StageState::Failed) {
            container(Space::new())
                .width(Length::Fixed(3.0))
                .height(Length::Fill)
                .style(move |_| style::indicator_rail(accent, stage == UninstallStage::System))
                .into()
        } else {
            Space::new().width(Length::Fixed(3.0)).into()
        };

    container(
        row![
            rail,
            text(number).font(style::MONO_FONT).size(13).color(accent),
            text("●").font(style::FONT).size(9).color(accent),
            text(app.locale.t(label))
                .font(style::FONT_BOLD)
                .size(13)
                .color(label_color),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .height(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::FillPortion(1))
    .align_y(alignment::Vertical::Center)
    .into()
}

fn command_bar(app: &Uninstaller) -> Element<'_, Message> {
    let (label, message) = if app.can_close() {
        (app.locale.t(T::Close), Some(Message::ClosePressed))
    } else {
        (app.locale.t(T::RemovingEllipsis), None)
    };
    widgets::command_bar(label, message)
}
