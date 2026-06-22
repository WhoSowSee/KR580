use super::locale::{Locale, Text as T};
use super::{operations, platform, style, window_events};
use iced::widget::{Space, button, column, container, progress_bar, text};
use iced::{Element, Length, Settings, Size, Subscription, Task, Theme, alignment, theme};
use std::path::PathBuf;
use std::time::Duration;

#[path = "uninstaller_chrome.rs"]
mod uninstaller_chrome;

const ACTION_HEIGHT: f32 = 44.0;

#[derive(Debug, Clone)]
enum Message {
    UninstallProgressTick,
    UninstallFinished(Result<(), String>),
    ClosePressed,
    RemovalScheduled(Result<(), String>),
    WindowOpened(iced::window::Id),
    WindowDragStart,
    WindowMinimize,
    WindowClose,
}

struct Uninstaller {
    install_dir: PathBuf,
    uninstalling: bool,
    closing: bool,
    pending_uninstall: Option<PathBuf>,
    progress: f32,
    result: Option<Result<(), String>>,
    close_error: Option<String>,
    window_id: Option<iced::window::Id>,
    locale: Locale,
}

pub fn run(install_dir: PathBuf) -> iced::Result {
    iced::application(
        move || Uninstaller::new(install_dir.clone()),
        Uninstaller::update,
        Uninstaller::view,
    )
    .title(title)
    .theme(theme)
    .subscription(Uninstaller::subscription)
    .style(app_style)
    .settings(Settings {
        antialiasing: true,
        ..Settings::default()
    })
    .window(iced::window::Settings {
        size: Size::new(620.0, 420.0),
        min_size: Some(Size::new(560.0, 360.0)),
        position: iced::window::Position::Centered,
        decorations: false,
        exit_on_close_request: false,
        ..iced::window::Settings::default()
    })
    .run()
}

fn theme(_state: &Uninstaller) -> Theme {
    Theme::Dark
}

fn title(state: &Uninstaller) -> String {
    state.locale.t(T::WindowTitleUninstaller).to_owned()
}

fn app_style(_state: &Uninstaller, _theme: &Theme) -> theme::Style {
    theme::Style {
        background_color: style::BLACK,
        text_color: style::TEXT,
    }
}

impl Uninstaller {
    fn new(install_dir: PathBuf) -> (Self, Task<Message>) {
        (
            Self {
                install_dir: install_dir.clone(),
                uninstalling: true,
                closing: false,
                pending_uninstall: Some(install_dir),
                progress: 0.08,
                result: None,
                close_error: None,
                window_id: None,
                locale: Locale::system(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UninstallProgressTick => {
                if let Some(install_dir) = self.pending_uninstall.take() {
                    self.progress = 0.18;
                    return Task::perform(
                        async move { operations::prepare_uninstall(&install_dir) },
                        Message::UninstallFinished,
                    );
                }
                if self.uninstalling {
                    self.progress = if self.progress >= 0.92 {
                        0.18
                    } else {
                        self.progress + 0.08
                    };
                }
            }
            Message::UninstallFinished(result) => {
                self.uninstalling = false;
                self.pending_uninstall = None;
                self.progress = 1.0;
                self.result = Some(result);
            }
            Message::ClosePressed | Message::WindowClose => {
                return self.close_after_uninstall();
            }
            Message::RemovalScheduled(result) => {
                self.closing = false;
                match result {
                    Ok(()) => return self.close_window(),
                    Err(error) => {
                        self.close_error = Some(error);
                    }
                }
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                return iced::window::run(id, |window| platform::set_rounded_corners(window))
                    .discard();
            }
            Message::WindowDragStart => {
                return self.window_id.map_or_else(Task::none, iced::window::drag);
            }
            Message::WindowMinimize => {
                return self
                    .window_id
                    .map_or_else(Task::none, |id| iced::window::minimize(id, true));
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let window_events = Subscription::batch([
            iced::window::open_events().map(Message::WindowOpened),
            iced::event::listen_with(|event, _status, _window| {
                window_events::close_request(event).then_some(Message::WindowClose)
            }),
        ]);
        if self.uninstalling {
            Subscription::batch([
                window_events,
                iced::time::every(Duration::from_millis(120))
                    .map(|_| Message::UninstallProgressTick),
            ])
        } else {
            window_events
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            uninstaller_chrome::title_bar(self.uninstalling, self.closing),
            self.content()
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn content(&self) -> Element<'_, Message> {
        let mut content = column![
            self.header(),
            self.path_panel(),
            self.status_panel(),
            Space::new().height(Length::Fill),
        ]
        .spacing(14)
        .padding(22);

        if let Some(error) = &self.close_error {
            content = content.push(
                text(error)
                    .font(style::FONT)
                    .size(12)
                    .color(style::WHITE)
                    .width(Length::Fill),
            );
        }

        if !self.uninstalling {
            content = content.push(self.close_button());
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn header(&self) -> Element<'_, Message> {
        column![
            text(self.locale.t(T::UninstallerTitle))
                .font(style::FONT_BOLD)
                .size(30)
                .color(style::TEXT),
            text(self.locale.t(T::UninstallerSubtitle))
                .font(style::FONT)
                .size(15)
                .color(style::MUTED),
        ]
        .spacing(4)
        .into()
    }

    fn path_panel(&self) -> Element<'_, Message> {
        container(
            column![
                text(self.locale.t(T::InstallFolder))
                    .font(style::FONT_BOLD)
                    .size(13)
                    .color(style::WHITE),
                text(self.install_dir.display().to_string())
                    .font(style::FONT)
                    .size(13)
                    .color(style::MUTED)
                    .width(Length::Fill),
            ]
            .spacing(6),
        )
        .padding(14)
        .style(style::soft_panel)
        .width(Length::Fill)
        .into()
    }

    fn status_panel(&self) -> Element<'_, Message> {
        let (title, body) = match self.result.as_ref() {
            Some(Ok(())) => (self.locale.t(T::Removed), self.locale.t(T::RemovedBody)),
            Some(Err(error)) => (self.locale.t(T::UninstallFailed), error.as_str()),
            None => (self.locale.t(T::Removing), self.locale.t(T::RemovingBody)),
        };

        container(
            column![
                text(title)
                    .font(style::FONT_BOLD)
                    .size(17)
                    .color(style::WHITE),
                text(body)
                    .font(style::FONT)
                    .size(13)
                    .color(style::MUTED)
                    .width(Length::Fill),
                progress_bar(0.0..=1.0, self.progress)
                    .girth(Length::Fixed(8.0))
                    .style(style::progress),
            ]
            .spacing(10),
        )
        .padding(14)
        .style(style::panel)
        .width(Length::Fill)
        .into()
    }

    fn close_button(&self) -> Element<'_, Message> {
        let label = if self.closing {
            self.locale.t(T::ClosingEllipsis)
        } else {
            self.locale.t(T::Close)
        };
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
        .height(Length::Fixed(ACTION_HEIGHT))
        .style(style::primary_button)
        .on_press_maybe((!self.closing).then_some(Message::ClosePressed))
        .into()
    }

    fn close_after_uninstall(&mut self) -> Task<Message> {
        if self.uninstalling || self.closing {
            return Task::none();
        }
        if matches!(self.result, Some(Ok(()))) {
            self.closing = true;
            self.close_error = None;
            let install_dir = self.install_dir.clone();
            return Task::perform(
                async move { operations::schedule_install_dir_removal(&install_dir) },
                Message::RemovalScheduled,
            );
        }
        self.close_window()
    }

    fn close_window(&self) -> Task<Message> {
        self.window_id.map_or_else(iced::exit, iced::window::close)
    }
}
