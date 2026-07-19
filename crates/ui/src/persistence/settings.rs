use crate::devices::printer::PrinterSettings;
use crate::persistence::{SettingsError, ShortcutSettings};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};

const SETTINGS_VERSION: u32 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings_version: u32,
    pub network: NetworkSettings,
    pub storage: StorageSettings,
    pub export: ExportSettings,
    pub ui: UiSettings,
    pub general: GeneralSettings,
    #[serde(default)]
    pub shortcuts: ShortcutSettings,
    pub recent_files: Vec<PathBuf>,
}

/// General-purpose user preferences exposed via the in-app Settings dialog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub language: Language,
    pub default_speed: SpeedPreset,
    pub follow_pc: bool,
    pub memory_operand_highlighting: bool,
    pub floppy_image_path: Option<PathBuf>,
    pub hdd_directory: Option<PathBuf>,
    #[serde(default)]
    pub printer_name: Option<String>,
    #[serde(default)]
    pub printer_settings: Option<PrinterSettings>,
    #[serde(default)]
    pub printer_dialog_mode: PrinterDialogMode,
    #[serde(default)]
    pub printer_presets: Vec<PrinterPreset>,
}

impl GeneralSettings {
    pub fn set_printer_settings(&mut self, settings: Option<PrinterSettings>) {
        self.printer_name = settings
            .as_ref()
            .map(|settings| settings.printer_name.clone());
        self.printer_settings = settings;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterPreset {
    pub name: String,
    pub settings: PrinterSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Ru,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpeedPreset {
    Slow,
    Medium,
    High,
    Max,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrinterDialogMode {
    #[default]
    Custom,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub mode: NetworkMode,
    pub host: String,
    pub port: u16,
    pub bind_host: String,
    pub bind_port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkMode {
    Client,
    Server,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSettings {
    pub floppy_path: PathBuf,
    pub hdd_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    pub default_directory: PathBuf,
    pub line_endings: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub theme: ColorScheme,
    pub ram_view_base: u16,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    TokyoNight,
    TokyoNightLight,
    BlackWhiteDark,
    BlackWhiteLight,
    KanagawaWave,
    KanagawaLotus,
    CatppuccinMocha,
    CatppuccinLatte,
    Nord,
    GruvboxDark,
    GruvboxLight,
    MaterialOcean,
}

impl ColorScheme {
    pub const DEFAULT: Self = Self::TokyoNight;

    pub fn storage_name(self) -> &'static str {
        match self {
            Self::TokyoNight => "tokyoNight",
            Self::TokyoNightLight => "tokyoNightLight",
            Self::BlackWhiteDark => "blackWhiteDark",
            Self::BlackWhiteLight => "blackWhiteLight",
            Self::KanagawaWave => "kanagawaWave",
            Self::KanagawaLotus => "kanagawaLotus",
            Self::CatppuccinMocha => "catppuccinMocha",
            Self::CatppuccinLatte => "catppuccinLatte",
            Self::Nord => "nord",
            Self::GruvboxDark => "gruvboxDark",
            Self::GruvboxLight => "gruvboxLight",
            Self::MaterialOcean => "materialOcean",
        }
    }

    pub fn from_storage_name(raw: &str) -> Option<Self> {
        match raw {
            "dark" | "tokyoNight" => Some(Self::TokyoNight),
            "light" | "tokyoNightLight" => Some(Self::TokyoNightLight),
            "blackWhiteDark" => Some(Self::BlackWhiteDark),
            "blackWhiteLight" => Some(Self::BlackWhiteLight),
            "kanagawaWave" => Some(Self::KanagawaWave),
            "kanagawaLotus" => Some(Self::KanagawaLotus),
            "catppuccinMocha" => Some(Self::CatppuccinMocha),
            "catppuccinLatte" => Some(Self::CatppuccinLatte),
            "nord" => Some(Self::Nord),
            "gruvboxDark" => Some(Self::GruvboxDark),
            "gruvboxLight" => Some(Self::GruvboxLight),
            "materialOcean" => Some(Self::MaterialOcean),
            "monokai" => Some(Self::TokyoNight),
            _ => None,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::TokyoNight,
            1 => Self::TokyoNightLight,
            2 => Self::BlackWhiteDark,
            3 => Self::BlackWhiteLight,
            4 => Self::KanagawaWave,
            5 => Self::KanagawaLotus,
            6 => Self::CatppuccinMocha,
            7 => Self::CatppuccinLatte,
            8 => Self::Nord,
            9 => Self::GruvboxDark,
            10 => Self::GruvboxLight,
            11 => Self::MaterialOcean,
            _ => Self::DEFAULT,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Serialize for ColorScheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.storage_name())
    }
}

impl<'de> Deserialize<'de> for ColorScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::from_storage_name(&raw)
            .ok_or_else(|| serde::de::Error::custom(format!("unsupported color scheme: {raw}")))
    }
}

pub struct SettingsStore;

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,
            network: NetworkSettings::default(),
            storage: StorageSettings::default(),
            export: ExportSettings::default(),
            ui: UiSettings::default(),
            general: GeneralSettings::default(),
            shortcuts: ShortcutSettings::default(),
            recent_files: Vec::new(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            default_speed: SpeedPreset::High,
            follow_pc: false,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            printer_name: None,
            printer_settings: None,
            printer_dialog_mode: PrinterDialogMode::default(),
            printer_presets: Vec::new(),
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Client,
            host: "127.0.0.1".to_owned(),
            port: 5800,
            bind_host: "127.0.0.1".to_owned(),
            bind_port: 5800,
        }
    }
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            floppy_path: PathBuf::from("floppy.kpd"),
            hdd_path: PathBuf::from("hdd.kpd"),
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_directory: PathBuf::from("."),
            line_endings: "LF".to_owned(),
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: ColorScheme::DEFAULT,
            ram_view_base: 0,
        }
    }
}

impl SettingsStore {
    pub fn to_json(settings: &Settings) -> Result<String, SettingsError> {
        Ok(serde_json::to_string_pretty(settings)?)
    }

    pub fn from_json(json: &str) -> Result<Settings, SettingsError> {
        let mut settings: Settings = serde_json::from_str(json)?;
        let source_version = settings.settings_version;
        match source_version {
            SETTINGS_VERSION => {}
            1..=7 => {
                if source_version == 1 {
                    settings.network = NetworkSettings::default();
                }
                if source_version <= 4 {
                    settings.general.printer_name = None;
                }
                if source_version <= 5 {
                    settings.general.printer_dialog_mode = PrinterDialogMode::default();
                }
                settings.settings_version = SETTINGS_VERSION;
            }
            version => return Err(SettingsError::UnsupportedVersion(version)),
        }
        let printer_settings = settings.general.printer_settings.take().or_else(|| {
            settings
                .general
                .printer_name
                .take()
                .map(PrinterSettings::named)
        });
        settings.general.set_printer_settings(printer_settings);
        settings.shortcuts.normalize();
        Ok(settings)
    }

    pub fn save(path: impl AsRef<Path>, settings: &Settings) -> Result<(), SettingsError> {
        std::fs::write(path, Self::to_json(settings)?)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Settings, SettingsError> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }
}
