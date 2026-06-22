pub mod entry;

mod locale;
mod operations;
mod platform;
mod style;
mod uninstaller;
mod view;
mod window_events;

use iced::{Element, Settings, Size, Subscription, Task, Theme, time, window};
use k580_ui::install_mode::{InstallMode, InstallScope};
use locale::Locale;
use operations::{
    InstallReport, InstallRequest, default_install_dir, install, launch_installed_app,
    open_install_folder,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    ModeSelected(InstallMode),
    #[cfg(windows)]
    ScopeSelected(InstallScope),
    InstallDirChanged(String),
    BrowseInstallDir,
    InstallDirPicked(Option<PathBuf>),
    AddToPathToggled(bool),
    DesktopShortcutToggled(bool),
    FileAssociationToggled(bool),
    InstallPressed,
    InstallProgressTick,
    InstallFinished(Result<InstallReport, String>),
    PostInstallActionToggled(bool),
    DonePressed,
    PostInstallActionFinished(Result<(), String>),
    WindowOpened(window::Id),
    WindowDragStart,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowMaximizedChanged(bool),
}

pub struct Installer {
    mode: InstallMode,
    locale: Locale,
    scope: InstallScope,
    install_dir: String,
    add_to_path: bool,
    create_desktop_shortcut: bool,
    associate_580_files: bool,
    installing: bool,
    pending_install: Option<InstallRequest>,
    install_progress: f32,
    post_install_action: bool,
    post_install_error: Option<String>,
    result: Option<Result<InstallReport, String>>,
    window_id: Option<window::Id>,
    window_maximized: bool,
}

pub fn run() -> iced::Result {
    iced::application(Installer::new, Installer::update, Installer::view)
        .title(title)
        .theme(theme)
        .subscription(Installer::subscription)
        .style(style::app_style)
        .settings(Settings {
            antialiasing: true,
            ..Settings::default()
        })
        .window(window::Settings {
            size: Size::new(680.0, 760.0),
            min_size: Some(Size::new(640.0, 720.0)),
            position: window::Position::Centered,
            decorations: false,
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .run()
}

pub fn run_uninstaller(install_dir: PathBuf) -> iced::Result {
    uninstaller::run(install_dir)
}

fn title(state: &Installer) -> String {
    state
        .locale
        .t(locale::Text::WindowTitleInstaller)
        .to_owned()
}

fn theme(_state: &Installer) -> Theme {
    Theme::Dark
}

impl Installer {
    fn new() -> (Self, Task<Message>) {
        let mode = InstallMode::System;
        let scope = InstallScope::User;
        (
            Self {
                mode,
                locale: Locale::system(),
                scope,
                install_dir: default_install_dir(mode, scope).display().to_string(),
                add_to_path: true,
                create_desktop_shortcut: true,
                associate_580_files: true,
                installing: false,
                pending_install: None,
                install_progress: 0.0,
                post_install_action: true,
                post_install_error: None,
                result: None,
                window_id: None,
                window_maximized: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ModeSelected(mode) => {
                self.mode = mode;
                if mode == InstallMode::Portable {
                    self.scope = InstallScope::User;
                }
                self.install_dir = default_install_dir(self.mode, self.scope)
                    .display()
                    .to_string();
                self.result = None;
                self.pending_install = None;
                self.post_install_action = true;
                self.post_install_error = None;
            }
            #[cfg(windows)]
            Message::ScopeSelected(scope) => {
                self.scope = scope;
                self.install_dir = default_install_dir(self.mode, self.scope)
                    .display()
                    .to_string();
                self.result = None;
                self.pending_install = None;
                self.post_install_action = true;
                self.post_install_error = None;
            }
            Message::InstallDirChanged(value) => {
                self.install_dir = value;
                self.result = None;
                self.post_install_error = None;
            }
            Message::BrowseInstallDir => {
                let current = PathBuf::from(self.install_dir.clone());
                return Task::perform(
                    async move { pick_folder(current) },
                    Message::InstallDirPicked,
                );
            }
            Message::InstallDirPicked(Some(path)) => {
                self.install_dir = path.display().to_string();
                self.result = None;
                self.post_install_error = None;
            }
            Message::InstallDirPicked(None) => {}
            Message::AddToPathToggled(value) => {
                self.add_to_path = value;
                self.result = None;
                self.post_install_error = None;
            }
            Message::DesktopShortcutToggled(value) => {
                self.create_desktop_shortcut = value;
                self.result = None;
                self.post_install_error = None;
            }
            Message::FileAssociationToggled(value) => {
                self.associate_580_files = value;
                self.result = None;
                self.post_install_error = None;
            }
            Message::InstallPressed => {
                if self.installing {
                    return Task::none();
                }
                let request = match self.request() {
                    Ok(request) => request,
                    Err(error) => {
                        self.result = Some(Err(error));
                        return Task::none();
                    }
                };
                self.installing = true;
                self.pending_install = Some(request);
                self.install_progress = 0.06;
                self.post_install_error = None;
                self.result = None;
            }
            Message::InstallProgressTick => {
                if self.installing {
                    if let Some(request) = self.pending_install.take() {
                        self.install_progress = 0.18;
                        return Task::perform(
                            async move { install(request) },
                            Message::InstallFinished,
                        );
                    }
                    self.install_progress = if self.install_progress >= 0.92 {
                        0.18
                    } else {
                        self.install_progress + 0.08
                    };
                }
            }
            Message::InstallFinished(result) => {
                self.installing = false;
                self.pending_install = None;
                self.install_progress = 1.0;
                self.post_install_action = result.is_ok();
                self.result = Some(result);
            }
            Message::PostInstallActionToggled(value) => {
                self.post_install_action = value;
                self.post_install_error = None;
            }
            Message::DonePressed => {
                let Some(Ok(report)) = self.result.clone() else {
                    return self.close_window();
                };
                if !self.post_install_action {
                    return self.close_window();
                }
                self.post_install_error = None;
                return Task::perform(
                    async move {
                        match report.mode {
                            InstallMode::Portable => open_install_folder(report.install_dir),
                            InstallMode::System => launch_installed_app(report.k580_path),
                        }
                    },
                    Message::PostInstallActionFinished,
                );
            }
            Message::PostInstallActionFinished(result) => match result {
                Ok(()) => return self.close_window(),
                Err(error) => {
                    self.post_install_error = Some(error);
                }
            },
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                return Task::batch([
                    window::run(id, |window| platform::set_rounded_corners(window)).discard(),
                    window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowDragStart => {
                return self.window_id.map_or_else(Task::none, window::drag);
            }
            Message::WindowMinimize => {
                return self
                    .window_id
                    .map_or_else(Task::none, |id| window::minimize(id, true));
            }
            Message::WindowToggleMaximize => {
                let Some(id) = self.window_id else {
                    return Task::none();
                };
                self.window_maximized = !self.window_maximized;
                return Task::batch([
                    window::toggle_maximize(id),
                    window::is_maximized(id).map(Message::WindowMaximizedChanged),
                ]);
            }
            Message::WindowClose => {
                return self.close_window();
            }
            Message::WindowMaximizedChanged(maximized) => {
                self.window_maximized = maximized;
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let window_events = Subscription::batch([
            window::open_events().map(Message::WindowOpened),
            iced::event::listen_with(|event, _status, _window| {
                window_events::close_request(event).then_some(Message::WindowClose)
            }),
        ]);
        if self.installing {
            Subscription::batch([
                window_events,
                time::every(Duration::from_millis(120)).map(|_| Message::InstallProgressTick),
            ])
        } else {
            window_events
        }
    }

    fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }

    fn request(&self) -> Result<InstallRequest, String> {
        let install_dir = self.install_dir.trim();
        if install_dir.is_empty() {
            return Err("installation folder is empty".to_owned());
        }
        Ok(InstallRequest {
            mode: self.mode,
            scope: self.scope,
            install_dir: PathBuf::from(install_dir),
            add_to_path: self.add_to_path,
            create_desktop_shortcut: self.create_desktop_shortcut,
            associate_580_files: self.associate_580_files,
        })
    }

    fn close_window(&self) -> Task<Message> {
        self.window_id.map_or_else(iced::exit, window::close)
    }
}

fn pick_folder(current: PathBuf) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if current.is_dir() {
        dialog = dialog.set_directory(current);
    }
    dialog.pick_folder()
}

impl Installer {
    pub fn mode(&self) -> InstallMode {
        self.mode
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    #[cfg(windows)]
    pub fn scope(&self) -> InstallScope {
        self.scope
    }

    pub fn install_dir(&self) -> &str {
        &self.install_dir
    }

    pub fn add_to_path(&self) -> bool {
        self.add_to_path
    }

    pub fn create_desktop_shortcut(&self) -> bool {
        self.create_desktop_shortcut
    }

    pub fn associate_580_files(&self) -> bool {
        self.associate_580_files
    }

    pub fn installing(&self) -> bool {
        self.installing
    }

    pub fn install_progress(&self) -> f32 {
        self.install_progress
    }

    pub fn post_install_action(&self) -> bool {
        self.post_install_action
    }

    pub fn post_install_error(&self) -> Option<&str> {
        self.post_install_error.as_deref()
    }

    pub fn result(&self) -> Option<&Result<InstallReport, String>> {
        self.result.as_ref()
    }

    pub fn window_maximized(&self) -> bool {
        self.window_maximized
    }
}
