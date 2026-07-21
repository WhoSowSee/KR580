mod devices;
mod settings;

use super::{help_ru, keys::Key};
pub(super) fn translate(key: Key) -> &'static str {
    if let Some(value) = settings::translate(key) {
        return value;
    }
    if let Some(value) = devices::translate(key) {
        return value;
    }

    match key {
        Key::MenuFile => "Файл",
        Key::MenuMp => "МП-Система",
        Key::MenuView => "Вид",
        Key::MenuSettings => "Настройки",
        Key::MenuHelp => "Помощь",
        Key::HelpShowDocs => "Вызвать справку",
        Key::HelpAbout => "О программе",
        Key::HelpDialogTitle => "Справка",
        Key::HelpSearchPlaceholder => "Поиск по справке…",
        Key::HnAbout => "О программе",
        Key::HnAppearance => "Внешний вид",
        Key::HnArithmeticCommands => "Арифметические команды",
        Key::HnCommandSummary => "Сводка команд",
        Key::HnControlTransferCommands => "Команды передачи управления",
        Key::HnCpuArchitecture => "Микропроцессор КР580ВМ80",
        Key::HnDataTransferCommands => "Команды пересылок",
        Key::HnExport => "Экспорт",
        Key::HnExternalDevices => "Внешние устройства",
        Key::HnFeatures => "Возможности эмулятора",
        Key::HnFilesExport => "Файлы, импорт и экспорт",
        Key::HnFlagsRegister => "Регистр флагов",
        Key::HnFloppy => "Дисковод КР580",
        Key::HnGeneralPrinciples => "Быстрый старт",
        Key::HnGeneralSettings => "Все настройки",
        Key::HnHdd => "Жёсткий диск КР580",
        Key::HnImport => "Импорт",
        Key::HnInstructionSet => "Система команд",
        Key::HnIntroduction => "Введение",
        Key::HnIoCommands => "Команды ввода-вывода",
        Key::HnLogicalCommands => "Логические команды",
        Key::HnMainMenu => "Главное меню",
        Key::HnMainWindow => "Главное окно программы",
        Key::HnMemoryIoSpaces => "Пространства памяти и ввода-вывода",
        Key::HnMemorySearch => "Навигация по памяти",
        Key::HnMenuFile => "Меню «Файл»",
        Key::HnMenuHelp => "Меню «Справка»",
        Key::HnMenuMpSystem => "Меню «МП-Система»",
        Key::HnMenuView => "Меню «Вид»",
        Key::HnMonitor => "Монитор КР580",
        Key::HnNetwork => "Сетевой адаптер КР580",
        Key::HnPrinter => "Принтер КР580",
        Key::HnProcessorControlCommands => "Команды управления процессором",
        Key::HnProgramInterface => "Интерфейс и выполнение",
        Key::HnRamEditing => "ОЗУ: просмотр и редактирование",
        Key::HnRegisterEditing => "Редактирование регистров",
        Key::HnRegisters => "Регистры МП",
        Key::HnRunButtons => "Выполнение и сброс",
        Key::HnSaveLoad => "Файлы .580",
        Key::HnSettings => "Настройки программы",
        Key::HnStackCommands => "Стековые команды",
        Key::HnTopicShortcuts => "Горячие клавиши",
        Key::HcAbout
        | Key::HcFeatures
        | Key::HcGeneralPrinciples
        | Key::HcRegisters
        | Key::HcFlagsRegister
        | Key::HcMemoryIoSpaces
        | Key::HcDataTransferCommands
        | Key::HcLogicalCommands
        | Key::HcArithmeticCommands
        | Key::HcControlTransferCommands
        | Key::HcProcessorControlCommands
        | Key::HcIoCommands
        | Key::HcStackCommands
        | Key::HcMainWindow
        | Key::HcRamEditing
        | Key::HcRegisterEditing
        | Key::HcRunButtons
        | Key::HcMemorySearch
        | Key::HcMenuFile
        | Key::HcMenuMpSystem
        | Key::HcMenuView
        | Key::HcMenuHelp
        | Key::HcSaveLoad
        | Key::HcImport
        | Key::HcExport
        | Key::HcMonitor
        | Key::HcFloppy
        | Key::HcHdd
        | Key::HcNetwork
        | Key::HcPrinter
        | Key::HcGeneralSettings
        | Key::HcAppearance
        | Key::HcCommandSummary
        | Key::HcShortcuts => help_ru::translate(key),
        Key::AboutTitle => "О программе",
        Key::AppName => "KR580",
        Key::AboutDescription => {
            "Программа-эмулятор микропроцессорной системы на базе микропроцессора КР580ВМ80"
        }
        Key::AboutVersion => "Версия",
        Key::AboutGithubLabel => "GitHub",
        Key::FileNew => "Новый файл",
        Key::FileOpen => "Открыть",
        Key::FileSave => "Сохранить",
        Key::FileSaveAs => "Сохранить как",
        Key::FileImport => "Импорт",
        Key::FileExport => "Экспорт",
        Key::ExportFormatXlsx => "MS Excel",
        Key::ExportFormatText => "Текстовый файл",
        Key::ExportPageLabel => "На странице",
        Key::ExportPageDefault => "Подпрограмма 1",
        Key::ExportPageNameBase => "Подпрограмма",
        Key::ExportSectionLabel => "В разделе",
        Key::ExportSectionDefault => "Раздел 1",
        Key::ExportSectionNameBase => "Раздел",
        Key::ExportAddPageTooltip => "Добавить страницу",
        Key::ExportAddSectionTooltip => "Добавить раздел",
        Key::ExportDeletePageTooltip => "Удалить страницу",
        Key::ExportDeleteSectionTooltip => "Удалить раздел",
        Key::ExportMemoryGroup => "Содержимое ОЗУ",
        Key::ExportRegistersGroup => "Значения регистров",
        Key::ExportFlagsGroup => "Значения флагов",
        Key::ExportRangeFrom => "Ячейки, начиная с",
        Key::ExportRangeTo => "по",
        Key::ExportColumnAddress => "Включая столбец \"№ ячейки ОЗУ\"",
        Key::ExportColumnValue => "Включая столбец \"Значение ячейки ОЗУ\"",
        Key::ExportColumnCommand => "Включая столбец \"Команда\"",
        Key::ExportColumnComment => "Добавить пустой столбец для комментариев",
        Key::ExportRegisterAccumulator => "Аккумулятор",
        Key::ExportRegisterStackPointer => "Указатель стека",
        Key::ExportRegisterProgramCounter => "Счётчик команд",
        Key::ExportRegisterCycles => "Счётчик тактов",
        Key::ImportSourceGroup => "Источник импорта",
        Key::ImportFileLabel => "Файл",
        Key::ImportNoFile => "Файл не выбран",
        Key::ImportNoTargets => "В файле нет отдельных листов или разделов",
        Key::ImportSheetLabel => "На листе",
        Key::ImportSectionLabel => "В разделе",
        Key::ImportBrowseTooltip => "Выбрать файл",
        Key::ImportChooseFileRequired => "Выберите файл для импорта",
        Key::MpRunProgram => "Выполнить программу",
        Key::MpRunInstruction => "Выполнить команду",
        Key::MpRunTact => "Выполнить такт",
        Key::MpResetRam => "Очистить ОЗУ",
        Key::MpResetCpu => "Очистить регистры",
        Key::MpClearHalt => "Сбросить флаг HLT",
        Key::DiscardCancel => "Отменить",
        Key::DiscardBody => "Несохранённые изменения будут потеряны.",
        Key::DiscardBodyDeleteHdd => "Все данные будут потеряны.",
        Key::DiscardTitleOpen => "Открыть файл",
        Key::DiscardTitleNew => "Новый файл",
        Key::DiscardTitleImport => "Импорт",
        Key::DiscardTitleClose => "Закрыть приложение",
        Key::DiscardTitleDeleteHdd => "Удалить файл HDD?",
        Key::DiscardConfirmOpen => "Открыть",
        Key::DiscardConfirmNew => "Создать",
        Key::DiscardConfirmImport => "Импортировать",
        Key::DiscardConfirmClose => "Закрыть",
        Key::DiscardConfirmDeleteHdd => "Удалить",
        Key::StatusReady => "Готов",
        Key::StatusNewFile => "Новый файл",
        Key::StatusCpuHalted => "ЦП остановлен",
        Key::StatusStopped => "Остановлен",
        Key::StatusTact => "Такт",
        Key::StatusCycle => "цикл",
        Key::StatusOpened => "Открыто",
        Key::StatusSavedTo => "Сохранено в",
        Key::StatusExportTo => "Экспорт в",
        Key::StatusImportFrom => "Импорт из",
        Key::ErrorPrefix => "Ошибка",
        Key::HaltNotice => "Процессор остановлен командой HLT\nСбросьте регистры или флаг HLT",
        Key::SpeedTitle => "Скорость",
        Key::SpeedUnit => "инстр/сек",
        Key::HeaderStatus => "Статус",
        Key::HltOn => "HLT ВКЛ",
        Key::HltOff => "HLT ВЫКЛ",
        Key::RegistersAndOperands => "Регистры и операнды",
        Key::Accumulator => "Аккумулятор",
        Key::BufferRegister1 => "Буферный регистр 1",
        Key::BufferRegister2 => "Буферный регистр 2",
        Key::AddressBuffer => "Буфер адреса",
        Key::AddressBufferTooltip => {
            "Последний 16-разрядный адрес, выставленный процессором на адресную шину."
        }
        Key::InstructionRegister => "Регистр команд",
        Key::InstructionRegisterTooltip => {
            "Байт кода операции команды, выполняемой в данный момент."
        }
        Key::InstructionDecoder => "Д/Ш команд",
        Key::InstructionDecoderTooltip => {
            "Человекочитаемая мнемоника, декодированная из регистра команд."
        }
        Key::ControlSignals => "Сигналы управления",
        Key::CurrentCommand => "Текущая команда",
        Key::DataBuffer => "Буфер данных",
        Key::DataBufferTooltip => {
            "Последний байт, появившийся на 8-разрядной шине данных процессора. Обновляется при каждом чтении памяти или порта."
        }
        Key::FlagsRegister => "Регистр признаков",
        Key::FlagsRegisterTooltip => {
            "Компактное представление флагов PSW: S (знак), Z (ноль), AC (вспомогательный перенос), P (чётность), C (перенос). Биты 1 и 3 на 8080 всегда равны 0."
        }
        Key::StatusRegister => "Регистр состояния",
        Key::PswTooltip => {
            "Слово состояния программы: аккумулятор A, сложенный с байтом флагов PSW. Старший байт – A, младший – флаги."
        }
        Key::StackPointerTooltip => {
            "Указатель стека. Указывает на вершину программного стека в памяти; PUSH уменьшает, POP увеличивает."
        }
        Key::ProgramCounterTooltip => {
            "Счётчик команд. Содержит адрес следующего байта команды, который будет прочитан."
        }
        Key::IncDecTooltip => {
            "Количество байт, которое процессор добавит к счётчику команд после завершения текущей команды."
        }
        Key::Multiplexer => "Мультиплексор",
        Key::TempStorageRegisters => "Регистры временного хранения",
        Key::GeneralPurposeRegisters => "Регистры общего назначения (РОН)",
        Key::StackPointer => "Указатель стека (УС)",
        Key::ProgramCounter => "Счётчик команд (СК)",
        Key::IncDec => "Инкремент-декремент",

        Key::LampF2 => {
            "Вторая фаза такта\nВторая половина внутреннего такта; многие управляющие импульсы привязаны к этому фронту."
        }
        Key::LampF1 => {
            "Первая фаза такта\nПервая половина внутреннего такта; процессор начинает новое состояние шины."
        }
        Key::LampSync => {
            "Синхронизация\nАктивен в начале машинного цикла и определяет тип операции на шине."
        }
        Key::LampReady => {
            "Процессор готов\nПри активном уровне процессор завершает цикл шины; при низком – вставляет такты ожидания."
        }
        Key::LampWait => {
            "Состояние ожидания\nЗагорается, когда процессор растягивает цикл шины из-за неактивного READY."
        }
        Key::LampHold => {
            "Запрос захвата шины\nВнешнее устройство просит процессор отключиться от системной шины для прямого доступа к памяти."
        }
        Key::LampInt => {
            "Запрос прерывания\nВнешнее устройство требует обработки; учитывается только при разрешённых прерываниях."
        }
        Key::LampInte => {
            "Прерывания разрешены\nПри установленном флаге процессор принимает маскируемый запрос прерывания."
        }
        Key::LampDbin => {
            "Приём данных с шины\nПроцессор читает данные из памяти или порта ввода-вывода на шину данных."
        }
        Key::LampWr => {
            "Строб записи\nПроцессор записывает данные из аккумулятора или шины данных в память/порт."
        }
        Key::LampHlda => {
            "Подтверждение захвата\nПроцессор отключился от шины и предоставил её контроллеру прямого доступа к памяти."
        }
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
        Key::MemoryListTitle => "Содержимое ячеек ОЗУ",
        Key::ColumnAddress => "Адрес",
        Key::ColumnValue => "Значение",
        Key::ColumnCommand => "Команда",
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
        Key::OpcodeSearchPlaceholder => "Поиск: hex или мнемоника",

        Key::StatusByteHeader => "Статусный байт T1: что процессор делает на текущем такте.",
        Key::StatusPrefix => "Статус:",
        Key::StatusNoProgramAt => "Нет программы по адресу",
        Key::StatusNothingToUndo => "Нечего отменять",
        Key::StatusNothingToRedo => "Нечего вернуть",
        Key::StatusEnterHexPattern => "Введите hex-шаблон для поиска",
        Key::StatusInvalidMemoryBytes => "Некорректные байты: используйте HEX-пары через пробел",
        Key::StatusMemoryBytesOutOfRange => "Последовательность не помещается в ОЗУ",
        Key::StatusPatternFound => "Найден шаблон",
        Key::StatusAtAddress => "по адресу",
        Key::StatusNoMatchesFor => "Нет адресов, соответствующих",
        Key::ErrNotA580File => "Не .580 файл – поддерживается только расширение .580",
        Key::ErrFileEmpty => "Файл пуст",
        Key::ErrWrong580Size => "Не похоже на .580 файл (должно быть ровно 65549 байт)",
        Key::ErrLegacyTrailerCorrupt => "Конец файла повреждён – это не .580 файл",
        Key::ErrSettingsNewerVersion => {
            "Настройки сохранены в более новой версии – обновите программу"
        }
        Key::ErrSettingsCorrupt => "Файл настроек повреждён",
        Key::ErrCannotReadFileFormat => "Не удалось прочитать файл – проверьте формат",
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
        Key::ErrFloppyImageNotAttached => "Файл образа дисковода не подключён",
        Key::ErrInternal => "Внутренняя ошибка приложения",
        Key::ErrGenericFailed => "Не удалось выполнить операцию",
        _ => unreachable!("missing russian translation for {key:?}"),
    }
}
