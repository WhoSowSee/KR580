use super::super::driver_locale::has_cyrillic;
use crate::i18n::Lang;
use k580_ui::devices::printer::{PrinterFeature, PrinterFeatureOption, PrinterParameter};

mod en;
mod tables;

#[cfg(test)]
mod tests;

pub(super) fn feature_label(feature: &PrinterFeature, lang: Lang) -> String {
    let local = local_name(&feature.name);
    match lang {
        Lang::Ru => tables::feature(local, lang)
            .map(str::to_owned)
            .or_else(|| has_cyrillic(&feature.display_name).then(|| feature.display_name.clone()))
            .unwrap_or_else(|| humanize_ru(local)),
        Lang::En => en::feature(local, &feature.display_name),
    }
}

pub(super) fn localized_options(feature: &PrinterFeature, lang: Lang) -> Vec<PrinterFeatureOption> {
    feature
        .options
        .iter()
        .filter(|option| {
            !option.constrained || feature.selected_option.as_deref() == Some(&option.name)
        })
        .cloned()
        .map(|mut option| {
            option.display_name = option_label(feature, &option, lang);
            option
        })
        .collect()
}

pub(super) fn parameter_label(parameter: &PrinterParameter, lang: Lang) -> String {
    let local = local_name(&parameter.name);
    match lang {
        Lang::Ru => tables::parameter(local, lang)
            .map(str::to_owned)
            .or_else(|| {
                has_cyrillic(&parameter.display_name).then(|| parameter.display_name.clone())
            })
            .unwrap_or_else(|| humanize_ru(local)),
        Lang::En => en::parameter(local, &parameter.display_name),
    }
}

pub(super) fn parameter_visible(parameter: &PrinterParameter) -> bool {
    local_name(&parameter.name) != "PageDevmodeSnapshot"
}

fn option_label(feature: &PrinterFeature, option: &PrinterFeatureOption, lang: Lang) -> String {
    let feature = local_name(&feature.name);
    let option_name = local_name(&option.name);
    if lang == Lang::En {
        return en::option(feature, option_name, &option.display_name);
    }
    if let Some(translated) = tables::contextual(feature, option_name, lang) {
        return translated.to_owned();
    }
    if has_cyrillic(&option.display_name) {
        return option.display_name.clone();
    }
    if let Some(translated) = tables::option(option_name, lang) {
        return translated.to_owned();
    }
    if is_opaque_option(option_name) {
        return option.display_name.clone();
    }
    humanize_ru(option_name)
}

fn humanize_ru(value: &str) -> String {
    split_words(value)
        .into_iter()
        .map(|word| translate_word(&word).unwrap_or(word.as_str()).to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for character in value.replace(['_', '-'], " ").chars() {
        let boundary = character.is_uppercase()
            && current
                .chars()
                .last()
                .is_some_and(|previous| previous.is_lowercase() || previous.is_ascii_digit());
        if character.is_whitespace() || boundary {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            if character.is_whitespace() {
                continue;
            }
        }
        current.push(character);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn translate_word(word: &str) -> Option<&'static str> {
    Some(match word.to_ascii_lowercase().as_str() {
        "document" => "Документ",
        "job" => "Задание",
        "page" => "Страница",
        "color" => "Цвет",
        "adjust" => "Настройка",
        "preferred" => "Предпочтительный",
        "black" => "Чёрный",
        "first" => "Первая",
        "only" => "Только",
        "printer" => "Принтер",
        "default" => "По умолчанию",
        "duplex" => "Двусторонняя печать",
        "reverse" => "Обратный",
        "orientation" => "Ориентация",
        "input" => "Подача",
        "bin" => "Лоток",
        "binding" => "Переплёт",
        "border" => "Рамка",
        "scaling" => "Масштабирование",
        "offset" => "Смещение",
        "alignment" => "Выравнивание",
        "overlay" => "Наложение",
        "media" => "Бумага",
        "size" => "Размер",
        "source" => "Источник",
        "type" => "Тип",
        "preview" => "Предпросмотр",
        "layout" => "Разметка",
        "text" => "Текст",
        "order" => "Порядок",
        "resolution" => "Разрешение",
        "watermark" => "Водяной знак",
        "header" => "Верхний колонтитул",
        "footer" => "Нижний колонтитул",
        "left" => "Левый",
        "center" => "Центральный",
        "right" => "Правый",
        _ => return None,
    })
}

fn is_opaque_option(name: &str) -> bool {
    name.strip_prefix('k').is_some_and(|value| {
        !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
    })
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}
