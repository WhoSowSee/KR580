use crate::i18n::Lang;

#[derive(Clone, Copy)]
pub(super) enum PropertyLabel {
    Title,
    Favorites,
    General,
    Paper,
    Graphics,
    Advanced,
    Profiles,
    PresetName,
    Save,
    Delete,
    Preview,
    NoPreview,
    Cancel,
    Ok,
    Apply,
    Loading,
    NoFeatures,
    Parameters,
    Size,
    Source,
    Orientation,
    Portrait,
    Landscape,
    ProviderError,
}

pub(super) fn label(lang: Lang, key: PropertyLabel) -> &'static str {
    match lang {
        Lang::Ru => ru(key),
        Lang::En => en(key),
    }
}

fn ru(key: PropertyLabel) -> &'static str {
    match key {
        PropertyLabel::Title => "Свойства принтера",
        PropertyLabel::Favorites => "Избранное",
        PropertyLabel::General => "Основные",
        PropertyLabel::Paper => "Бумага",
        PropertyLabel::Graphics => "Графика",
        PropertyLabel::Advanced => "Дополнительно",
        PropertyLabel::Profiles => "Профили",
        PropertyLabel::PresetName => "Название профиля",
        PropertyLabel::Save => "Сохранить",
        PropertyLabel::Delete => "Удалить",
        PropertyLabel::Preview => "Просмотр",
        PropertyLabel::NoPreview => "Буфер принтера пуст",
        PropertyLabel::Cancel => "Отмена",
        PropertyLabel::Ok => "OK",
        PropertyLabel::Apply => "Применить",
        PropertyLabel::Loading => "Загрузка свойств...",
        PropertyLabel::NoFeatures => "Для этого раздела нет доступных параметров",
        PropertyLabel::Parameters => "Параметры драйвера",
        PropertyLabel::Size => "Размер",
        PropertyLabel::Source => "Подача",
        PropertyLabel::Orientation => "Ориентация",
        PropertyLabel::Portrait => "Книжная",
        PropertyLabel::Landscape => "Альбомная",
        PropertyLabel::ProviderError => "Расширенные свойства драйвера недоступны",
    }
}

fn en(key: PropertyLabel) -> &'static str {
    match key {
        PropertyLabel::Title => "Printer properties",
        PropertyLabel::Favorites => "Favorites",
        PropertyLabel::General => "General",
        PropertyLabel::Paper => "Paper",
        PropertyLabel::Graphics => "Graphics",
        PropertyLabel::Advanced => "Advanced",
        PropertyLabel::Profiles => "Profiles",
        PropertyLabel::PresetName => "Profile name",
        PropertyLabel::Save => "Save",
        PropertyLabel::Delete => "Delete",
        PropertyLabel::Preview => "Preview",
        PropertyLabel::NoPreview => "Printer buffer is empty",
        PropertyLabel::Cancel => "Cancel",
        PropertyLabel::Ok => "OK",
        PropertyLabel::Apply => "Apply",
        PropertyLabel::Loading => "Loading properties...",
        PropertyLabel::NoFeatures => "No settings are available in this section",
        PropertyLabel::Parameters => "Driver parameters",
        PropertyLabel::Size => "Size",
        PropertyLabel::Source => "Source",
        PropertyLabel::Orientation => "Orientation",
        PropertyLabel::Portrait => "Portrait",
        PropertyLabel::Landscape => "Landscape",
        PropertyLabel::ProviderError => "Advanced driver properties are unavailable",
    }
}
