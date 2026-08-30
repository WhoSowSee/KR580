use super::locale::{Locale, Text as T};
use super::{operations, platform, style, window_events};
use iced::{Settings, Size, Subscription, Task, time};
use std::path::PathBuf;
use std::time::Duration;

#[path = "uninstaller_view.rs"]
mod uninstaller_view;

const PROGRESS_FAST_STEP: f32 = 0.02;
const PROGRESS_DRIFT_STEP: f32 = 0.001;
const PROGRESS_STAGE_RESERVE: f32 = 0.01;
const PROGRESS_TICK_INTERVAL: Duration = Duration::from_millis(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UninstallStage {
    System,
    Links,
    Files,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StageState {
    Pending,
    Active,
    Complete,
    Failed,
}

impl UninstallStage {
    fn progress(self) -> f32 {
        match self {
            Self::System => 0.12,
            Self::Links => 0.40,
            Self::Files => 0.68,
        }
    }

    fn animation_limit(self) -> f32 {
        let next = match self {
            Self::System => Self::Links.progress(),
            Self::Links => Self::Files.progress(),
            Self::Files => 1.0,
        };
        next - PROGRESS_STAGE_RESERVE
    }
}

#[derive(Debug, Clone)]
enum Message {
    ProgressTick,
    SystemRemoved(Result<operations::UninstallPlan, String>),
    LinksRemoved(Result<(), String>),
    FilesScheduled(Result<(), String>),
    ClosePressed,
    WindowOpened(iced::window::Id),
    WindowDragStart,
    WindowMinimize,
    WindowToggleMaximize,
    WindowMaximizedChanged(bool),
    WindowClose,
}

struct Uninstaller {
    install_dir: PathBuf,
    started: bool,
    stage: UninstallStage,
    display_progress: f32,
    result: Option<Result<(), String>>,
    window_id: Option<iced::window::Id>,
    window_maximized: bool,
    locale: Locale,
}

pub(super) fn run(install_dir: PathBuf) -> iced::Result {
    iced::application(
        move || Uninstaller::new(install_dir.clone()),
        Uninstaller::update,
        uninstaller_view::view,
    )
    .title(|state: &Uninstaller| state.locale.t(T::WindowTitleUninstaller).to_owned())
    .theme(|_: &Uninstaller| iced::Theme::TokyoNight)
    .subscription(Uninstaller::subscription)
    .style(|_, _| style::application())
    .settings(Settings {
        antialiasing: true,
        ..Settings::default()
    })
    .window(iced::window::Settings {
        size: Size::new(760.0, 500.0),
        min_size: Some(Size::new(700.0, 460.0)),
        position: iced::window::Position::Centered,
        decorations: false,
        exit_on_close_request: false,
        ..iced::window::Settings::default()
    })
    .run()
}

impl Uninstaller {
    fn new(install_dir: PathBuf) -> (Self, Task<Message>) {
        (
            Self {
                install_dir,
                started: false,
                stage: UninstallStage::System,
                display_progress: 0.0,
                result: None,
                window_id: None,
                window_maximized: false,
                locale: Locale::system(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ProgressTick => {
                let confirmed = self.confirmed_progress();
                let (target, step) = if self.display_progress < confirmed {
                    (confirmed, PROGRESS_FAST_STEP)
                } else {
                    (self.target_progress(), PROGRESS_DRIFT_STEP)
                };
                self.display_progress = (self.display_progress + step).min(target);
            }
            Message::SystemRemoved(result) => match result {
                Ok(plan) => {
                    self.stage = UninstallStage::Links;
                    return Task::perform(
                        async move { operations::remove_links(plan) },
                        Message::LinksRemoved,
                    );
                }
                Err(error) => {
                    self.result = Some(Err(error));
                }
            },
            Message::LinksRemoved(result) => match result {
                Ok(()) => {
                    self.stage = UninstallStage::Files;
                    let install_dir = self.install_dir.clone();
                    return Task::perform(
                        async move { platform::schedule_remove_install_dir(&install_dir) },
                        Message::FilesScheduled,
                    );
                }
                Err(error) => {
                    self.result = Some(Err(error));
                }
            },
            Message::FilesScheduled(result) => {
                self.result = Some(result);
            }
            Message::ClosePressed | Message::WindowClose => {
                if self.can_close() {
                    return self.window_id.map_or_else(iced::exit, iced::window::close);
                }
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                let mut tasks = vec![
                    iced::window::run(id, |window| platform::set_rounded_corners(window)).discard(),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ];
                if !self.started {
                    self.started = true;
                    let install_dir = self.install_dir.clone();
                    tasks.push(Task::perform(
                        async move { operations::remove_system_entries(install_dir) },
                        Message::SystemRemoved,
                    ));
                }
                return Task::batch(tasks);
            }
            Message::WindowDragStart => {
                return self.window_id.map_or_else(Task::none, iced::window::drag);
            }
            Message::WindowMinimize => {
                return self
                    .window_id
                    .map_or_else(Task::none, |id| iced::window::minimize(id, true));
            }
            Message::WindowToggleMaximize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                self.window_maximized = !self.window_maximized;
                return Task::batch([
                    iced::window::toggle_maximize(id),
                    iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowMaximizedChanged(maximized) => {
                self.window_maximized = maximized;
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
        if self.progress_animating() {
            Subscription::batch([
                window_events,
                time::every(PROGRESS_TICK_INTERVAL).map(|_| Message::ProgressTick),
            ])
        } else {
            window_events
        }
    }

    fn stage_state(&self, stage: UninstallStage) -> StageState {
        match stage.cmp(&self.stage) {
            std::cmp::Ordering::Less => StageState::Complete,
            std::cmp::Ordering::Greater => StageState::Pending,
            std::cmp::Ordering::Equal if matches!(self.result, Some(Err(_))) => StageState::Failed,
            std::cmp::Ordering::Equal
                if matches!(self.result, Some(Ok(()))) && !self.progress_animating() =>
            {
                StageState::Complete
            }
            std::cmp::Ordering::Equal => StageState::Active,
        }
    }

    fn status(&self) -> (&str, iced::Color) {
        if let Some(Err(error)) = self.result.as_ref() {
            return (error, style::RED);
        }
        if matches!(self.result, Some(Ok(()))) && self.can_close() {
            return (self.locale.t(T::RemovalComplete), style::TEXT);
        }
        let text = match self.stage {
            UninstallStage::System => self.locale.t(T::RemovingSystem),
            UninstallStage::Links => self.locale.t(T::RemovingLinks),
            UninstallStage::Files => self.locale.t(T::RemovingFiles),
        };
        (text, style::MUTED)
    }

    fn target_progress(&self) -> f32 {
        if self.started && self.result.is_none() {
            self.stage.animation_limit()
        } else {
            self.confirmed_progress()
        }
    }

    fn confirmed_progress(&self) -> f32 {
        if !self.started && self.result.is_none() {
            return 0.0;
        }
        match self.result {
            Some(Ok(())) => 1.0,
            Some(Err(_)) => self.display_progress,
            None => self.stage.progress(),
        }
    }

    fn progress_animating(&self) -> bool {
        self.display_progress < self.target_progress()
    }

    fn can_close(&self) -> bool {
        self.result.is_some() && !self.progress_animating()
    }
}

#[cfg(test)]
mod tests;
