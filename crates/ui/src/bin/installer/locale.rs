#[derive(Clone, Copy)]
pub(super) enum Locale {
    Ru,
    En,
}

#[derive(Clone, Copy)]
pub(super) enum Text {
    WindowTitleInstaller,
    WindowTitleUninstaller,
    Mode,
    System,
    Portable,
    #[cfg(windows)]
    WindowsScope,
    #[cfg(windows)]
    CurrentUser,
    #[cfg(windows)]
    AllUsers,
    #[cfg(not(windows))]
    Scope,
    #[cfg(not(windows))]
    UserInstall,
    Folder,
    InstallationFolder,
    Browse,
    Integration,
    AddKrPath,
    DesktopShortcut,
    AssociateProgramFiles,
    Installing,
    InstallFailed,
    Installed,
    Location,
    TerminalLaunchEnabled,
    TerminalLaunchUnchanged,
    FileAssociationCreated,
    FileAssociationUnchanged,
    PortableReady,
    SearchDesktopUninstallReady,
    SearchUninstallReady,
    OpenInstallationFolder,
    LaunchKr580,
    InstallKr580,
    InstallingEllipsis,
    Done,
    UninstallStageSystem,
    UninstallStageLinks,
    UninstallStageFiles,
    RemovingSystem,
    RemovingLinks,
    RemovingFiles,
    RemovalComplete,
    RemovingEllipsis,
    Close,
}

impl Locale {
    pub(super) fn system() -> Self {
        match k580_ui::system_locale::default_language() {
            k580_ui::persistence::Language::Ru => Self::Ru,
            k580_ui::persistence::Language::En => Self::En,
        }
    }

    pub(super) fn t(self, text: Text) -> &'static str {
        match self {
            Self::Ru => ru(text),
            Self::En => en(text),
        }
    }
}

fn en(text: Text) -> &'static str {
    match text {
        Text::WindowTitleInstaller => "KR580 Setup",
        Text::WindowTitleUninstaller => "KR580 Uninstaller",
        Text::Mode => "Mode",
        Text::System => "System",
        Text::Portable => "Portable",
        #[cfg(windows)]
        Text::WindowsScope => "Install for",
        #[cfg(windows)]
        Text::CurrentUser => "Current user",
        #[cfg(windows)]
        Text::AllUsers => "All users",
        #[cfg(not(windows))]
        Text::Scope => "Scope",
        #[cfg(not(windows))]
        Text::UserInstall => "User install",
        Text::Folder => "Path",
        Text::InstallationFolder => "Installation folder",
        Text::Browse => "Browse",
        Text::Integration => "Integration",
        Text::AddKrPath => "Add kr command to PATH",
        Text::DesktopShortcut => "Create desktop shortcut",
        Text::AssociateProgramFiles => "Associate .580 and .krs files with KR580",
        Text::Installing => "Installing",
        Text::InstallFailed => "Install failed",
        Text::Installed => "Installed",
        Text::Location => "Location",
        Text::TerminalLaunchEnabled => "Terminal launch enabled",
        Text::TerminalLaunchUnchanged => "Terminal launch unchanged",
        Text::FileAssociationCreated => "KR580 registered for .580 and .krs files",
        Text::FileAssociationUnchanged => ".580 and .krs associations unchanged",
        Text::PortableReady => "Portable layout ready",
        Text::SearchDesktopUninstallReady => "Search, desktop shortcut, and uninstall entry ready",
        Text::SearchUninstallReady => "Search and uninstall entry ready",
        Text::OpenInstallationFolder => "Open installation folder",
        Text::LaunchKr580 => "Launch KR580",
        Text::InstallKr580 => "Install KR580",
        Text::InstallingEllipsis => "Installing...",
        Text::Done => "Done",
        Text::UninstallStageSystem => "SYSTEM",
        Text::UninstallStageLinks => "LINKS",
        Text::UninstallStageFiles => "FILES",
        Text::RemovingSystem => "Removing system entries",
        Text::RemovingLinks => "Removing PATH and file associations",
        Text::RemovingFiles => "Removing application files",
        Text::RemovalComplete => "All removal steps complete",
        Text::RemovingEllipsis => "Removing...",
        Text::Close => "Close",
    }
}

fn ru(text: Text) -> &'static str {
    match text {
        Text::WindowTitleInstaller => "Установка KR580",
        Text::WindowTitleUninstaller => "Удаление KR580",
        Text::Mode => "Режим",
        Text::System => "Системный",
        Text::Portable => "Портативный",
        #[cfg(windows)]
        Text::WindowsScope => "Установить для",
        #[cfg(windows)]
        Text::CurrentUser => "Текущий пользователь",
        #[cfg(windows)]
        Text::AllUsers => "Все пользователи",
        #[cfg(not(windows))]
        Text::Scope => "Область",
        #[cfg(not(windows))]
        Text::UserInstall => "Установка для пользователя",
        Text::Folder => "Путь",
        Text::InstallationFolder => "Папка установки",
        Text::Browse => "Обзор",
        Text::Integration => "Интеграция",
        Text::AddKrPath => "Добавить команду kr в PATH",
        Text::DesktopShortcut => "Создать ярлык на рабочем столе",
        Text::AssociateProgramFiles => "Связать .580 и .krs с KR580",
        Text::Installing => "Установка",
        Text::InstallFailed => "Установка не выполнена",
        Text::Installed => "Установлено",
        Text::Location => "Папка",
        Text::TerminalLaunchEnabled => "Запуск из терминала включён",
        Text::TerminalLaunchUnchanged => "Запуск из терминала не изменён",
        Text::FileAssociationCreated => "KR580 зарегистрирован для файлов .580 и .krs",
        Text::FileAssociationUnchanged => "Связи с .580 и .krs не изменены",
        Text::PortableReady => "Портативная установка готова",
        Text::SearchDesktopUninstallReady => "Поиск, ярлык и удаление готовы",
        Text::SearchUninstallReady => "Поиск и удаление готовы",
        Text::OpenInstallationFolder => "Открыть папку установки",
        Text::LaunchKr580 => "Запустить KR580",
        Text::InstallKr580 => "Установить KR580",
        Text::InstallingEllipsis => "Установка...",
        Text::Done => "Готово",
        Text::UninstallStageSystem => "СИСТЕМА",
        Text::UninstallStageLinks => "СВЯЗИ",
        Text::UninstallStageFiles => "ФАЙЛЫ",
        Text::RemovingSystem => "Удаление системных записей",
        Text::RemovingLinks => "Удаление PATH и связей файлов",
        Text::RemovingFiles => "Удаление файлов приложения",
        Text::RemovalComplete => "Все этапы удаления завершены",
        Text::RemovingEllipsis => "Удаление...",
        Text::Close => "Закрыть",
    }
}
