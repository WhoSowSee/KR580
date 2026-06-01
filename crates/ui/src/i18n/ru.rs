use super::keys::Key;

pub(super) fn translate(key: Key) -> &'static str {
    match key {
        // Top menu
        Key::MenuFile => "Файл",
        Key::MenuMp => "МП-Система",
        Key::MenuView => "Вид",
        Key::MenuSettings => "Настройки",
        Key::MenuHelp => "Помощь",

        // Help dropdown
        Key::HelpShowDocs => "Вызвать справку",
        Key::HelpAbout => "О программе",
        Key::HelpComingSoon => "Справка появится в будущей версии",
        // About dialog
        Key::AboutTitle => "О программе",
        Key::AppName => "KR580",
        Key::AboutDescription => {
            "Программа-эмулятор микропроцессорной системы на базе микропроцессора КР580ВМ80"
        }
        Key::AboutVersion => "Версия 1.0.0",
        Key::AboutGithubLabel => "GitHub",

        // File menu
        Key::FileNew => "Новый файл",
        Key::FileOpen => "Открыть",
        Key::FileSave => "Сохранить",
        Key::FileSaveAs => "Сохранить как",
        Key::FileImport => "Импорт",
        Key::FileExport => "Экспорт",
        Key::LegacyFormatNote => "старый формат",

        // MP-System menu
        Key::MpRunProgram => "Выполнить программу",
        Key::MpRunInstruction => "Выполнить команду",
        Key::MpRunTact => "Выполнить такт",
        Key::MpResetRam => "Очистить ОЗУ",
        Key::MpResetCpu => "Очистить регистры",
        Key::MpClearHalt => "Сбросить флаг HLT",

        // Discard modal
        Key::DiscardCancel => "Отменить",
        Key::DiscardBody => "Несохранённые изменения будут потеряны.",
        Key::DiscardTitleOpen => "Открыть файл",
        Key::DiscardTitleNew => "Новый файл",
        Key::DiscardTitleImport => "Импорт",
        Key::DiscardTitleClose => "Закрыть приложение",
        Key::DiscardConfirmOpen => "Открыть",
        Key::DiscardConfirmNew => "Создать",
        Key::DiscardConfirmImport => "Импортировать",
        Key::DiscardConfirmClose => "Закрыть",

        // Status & notices
        Key::StatusReady => "Готов",
        Key::StatusNewFile => "Новый файл",
        Key::StatusCpuHalted => "ЦП остановлен",
        Key::StatusStopped => "Остановлен",
        Key::StatusTact => "Такт",
        Key::StatusCycle => "цикл",
        Key::StatusOpened => "Открыто",
        Key::StatusSavedTo => "Сохранено в",
        Key::StatusExportTo => "Экспорт в",
        Key::ErrorPrefix => "Ошибка",
        Key::LegacyOpenedNotice => "Открыт старый формат файла",
        Key::HaltNotice => "Процессор остановлен командой HLT\nСбросьте регистры или флаг HLT",

        // Speed panel
        Key::SpeedTitle => "Скорость",
        Key::SpeedUnit => "инстр/сек",

        // Settings dialog
        Key::SettingsTitle => "Настройки",
        Key::SettingsSearchPlaceholder => "Поиск настроек",
        Key::SettingsCategoryGeneral => "Общие",
        Key::SettingsCategoryAppearance => "Внешний вид",
        Key::SettingsCategoryShortcuts => "Горячие клавиши",
        Key::SettingsLanguageLabel => "Язык",
        Key::SettingsLanguageHint => "Язык интерфейса приложения",
        Key::SettingsSpeedLabel => "Скорость",
        Key::SettingsSpeedHint => "Скорость по умолчанию для всех файлов",
        Key::SettingsThemeLabel => "Тема",
        Key::SettingsThemeHint => "Тема оформления интерфейса",
        Key::SettingsThemePlaceholder => "Скоро",
        Key::SettingsShortcutsLabel => "Горячие клавиши",
        Key::SettingsShortcutsHint => "Настройка сочетаний клавиш",
        Key::SettingsNoMatches => "Нет совпадений",
        Key::SettingsReset => "Сброс",
        Key::SettingsResetConfirmTitle => "Сбросить настройки?",
        Key::SettingsResetConfirmBody => "Все настройки вернутся к значениям по умолчанию",
        Key::SettingsResetConfirmAction => "Сбросить",
        Key::LangRussian => "Русский",
        Key::LangEnglish => "Английский",
        Key::SpeedSlow => "Низко",
        Key::SpeedMedium => "Средне",
        Key::SpeedHigh => "Быстро",
        Key::SpeedMax => "Макс",

        // Schematic header
        Key::HeaderStatus => "Статус",
        Key::HltOn => "HLT ВКЛ",
        Key::HltOff => "HLT ВЫКЛ",

        // Schematic registers grid
        Key::RegistersAndOperands => "Регистры и операнды",
        Key::Accumulator => "Аккумулятор",
        Key::BufferRegister1 => "Буферный регистр 1",
        Key::BufferRegister2 => "Буферный регистр 2",
        Key::AddressBuffer => "Буфер адреса",
        Key::InstructionRegister => "Регистр команд",
        Key::InstructionDecoder => "Д/Ш команд",
        Key::ControlSignals => "Сигналы управления",
        Key::CurrentCommand => "Текущая команда",
        Key::DataBuffer => "Буфер данных",
        Key::FlagsRegister => "Регистр признаков",
        Key::StatusRegister => "Регистр состояния",

        // Mux panel
        Key::Multiplexer => "Мультиплексор",
        Key::TempStorageRegisters => "Регистры временного хранения",
        Key::GeneralPurposeRegisters => "Регистры общего назначения (РОН)",
        Key::StackPointer => "Указатель стека (УС)",
        Key::ProgramCounter => "Счётчик команд (СК)",
        Key::IncDec => "Инкремент-декремент",

        // Cycles / timings
        Key::CyclesAndTacts => "Цикл и такт",
        Key::CycleLabel => "Цикл",
        Key::TactLabel => "Такт",
        Key::CycleTooltip => "Какой по счёту шаг сейчас выполняет команда. Числится с единицы.",
        Key::TactTooltip => "Номер такта внутри текущего шага команды. Числится с единицы.",
        Key::InternalTimings => "Внутренние тайминги",
        Key::TotalTacts => "Тактов",
        Key::InstructionTact => "Такт инструкции",
        Key::PhaseLabel => "Фаза",
        Key::TotalTactsTooltip => {
            "Сколько тактов всего прошло с начала программы. Числится с нуля."
        }
        Key::InstructionTactTooltip => {
            "Номер такта внутри текущей команды по полной шкале (T1, T2, ...). Числится с единицы."
        }
        Key::PhaseTooltip => "То же, что «Такт инструкции», но считается с нуля.",

        // Memory list
        Key::MemoryListTitle => "Содержимое ячеек ОЗУ",
        Key::ColumnAddress => "Адрес",
        Key::ColumnValue => "Значение",
        Key::ColumnCommand => "Команда",

        // Editors panels
        Key::MemoryEditorTitle => "Ячейка ОЗУ и ее значение",
        Key::RegisterEditorTitle => "Регистр и его значение",
        Key::ActionPause => "Пауза",
        Key::ActionRunProgram => "Выполнить программу",
        Key::ActionRestartProgram => "Перезапустить программу",
        Key::ActionStepInstruction => "Выполнить команду",
        Key::ActionStepTact => "Выполнить такт",
        Key::ActionResetRam => "Сброс ОЗУ",
        Key::ActionResetCpu => "Сброс регистров",
        Key::ExecutionPanel => "Выполнение",
        Key::ResetPanel => "Сброс",

        // Quick access devices
        Key::QuickAccess => "Быстрый доступ",
        Key::DeviceMonitor => "Отобразить монитор",
        Key::DeviceFloppy => "Отобразить буфер дисковода",
        Key::DeviceHdd => "Отобразить буфер жёсткого диска",
        Key::DeviceNetwork => "Отобразить буфер сетевого адаптера",
        Key::DevicePrinter => "Отобразить буфер принтера",

        // Monitor window
        Key::MonitorUnifiedScreen => "Экран КР580",
        Key::MonitorTextLayer => "Текстовый слой",
        Key::MonitorPixelLayer => "Графический слой",
        Key::MonitorHexBuffer => "Поток байт",
        Key::MonitorClose => "Закрыть",
        Key::MonitorViewSplit => "Разделить",
        Key::MonitorViewUnified => "Объединить",
        Key::MonitorClearBuffer => "Очистить буфер",
        Key::MonitorSaveImage => "Сохранить изображение",
        Key::MonitorImageSaved => "Изображение монитора сохранено",
        Key::MonitorImageSaveFailed => "Не удалось сохранить изображение",
        Key::MonitorHexFilterAll => "Фильтр: всё",
        Key::MonitorHexFilterGraphics => "Фильтр: графика",
        Key::MonitorHexFilterText => "Фильтр: текст",

        // Current command columns
        Key::ColCmdCode => "Код",
        Key::ColCmdMnemonic => "Команда",
        Key::ColCmdOperand => "Операнд",
        Key::ColCmdLength => "Длина",
        Key::ColCmdKind => "Тип",
        Key::ColCmdAddressing => "Адресация",
        Key::CmdLengthByte => "1 байт",
        Key::CmdLengthBytes2 => "2 байта",
        Key::CmdLengthBytes3 => "3 байта",
        Key::CmdKindUnknown => "неизвестно",
        Key::CmdKindControl => "управление",
        Key::CmdKindBranch => "переход",
        Key::CmdKindStack => "стек",
        Key::CmdKindIo => "ввод/вывод",
        Key::CmdKindMove => "пересылка",
        Key::CmdKindLogic => "логика",
        Key::CmdKindArithmetic => "арифметика",
        Key::CmdAddrImplicit => "неявная",
        Key::CmdAddrImmediate => "непосредств",
        Key::CmdAddrDirect => "прямая",
        Key::CmdAddrIndirect => "косвенная",
        Key::CmdAddrRegister => "регистровая",

        // Opcode dropdown
        Key::OpcodeSearchPlaceholder => "Поиск: hex или мнемоника",

        // Status register tooltip
        Key::StatusByteHeader => "Статусный байт T1: что процессор делает на текущем такте.",
        Key::StatusPrefix => "Статус:",

        // Runtime status messages
        Key::StatusNoProgramAt => "Нет программы по адресу",
        Key::StatusNothingToUndo => "Нечего отменять",
        Key::StatusNothingToRedo => "Нечего вернуть",
        Key::StatusEnterHexPattern => "Введите hex-шаблон для поиска",
        Key::StatusPatternFound => "Найден шаблон",
        Key::StatusAtAddress => "по адресу",
        Key::StatusNoMatchesFor => "Нет адресов, соответствующих",

        // Humanize error
        Key::ErrFileCorruptedOrUnsupported => "Файл повреждён или имеет неподдерживаемый формат",
        Key::ErrFileNewerVersion => "Файл сохранён в более новой версии — обновите программу",
        Key::ErrNotLegacyFormat => "Файл не похож на сохранение в старом формате",
        Key::ErrLegacyTrailerCorrupt => {
            "Конец файла повреждён — это не сохранение в старом формате"
        }
        Key::ErrSettingsNewerVersion => {
            "Настройки сохранены в более новой версии — обновите программу"
        }
        Key::ErrSettingsCorrupt => "Файл настроек повреждён",
        Key::ErrCannotReadFileFormat => "Не удалось прочитать файл — проверьте формат",
        Key::ErrCannotReadFile => "Не удалось прочитать файл",
        Key::ErrCannotWriteTable => "Не удалось записать таблицу",
        Key::ErrCannotWriteFile => "Не удалось записать файл",
        Key::ErrFileNotFound => "Файл не найден",
        Key::ErrPermissionDenied => "Нет доступа к файлу",
        Key::ErrFileAlreadyExists => "Файл уже существует",
        Key::ErrDiskFull => "На диске недостаточно места",
        Key::ErrIoGeneric => "Ошибка чтения или записи файла",
        Key::ErrAddressOutOfRange => "Адрес вне допустимого диапазона памяти",
        Key::ErrUnknownRegister => "Неизвестное имя регистра",
        Key::ErrUndocumentedOpcode => "Недокументированная команда",
        Key::ErrInternal => "Внутренняя ошибка приложения",
        Key::ErrGenericFailed => "Не удалось выполнить операцию",
    }
}
