use super::tables;
use super::{has_cyrillic, is_opaque_option, split_words};
use crate::i18n::Lang;
use crate::view::printer_setup::driver_locale;

pub(super) fn feature(name: &str, driver_label: &str) -> String {
    tables::feature(name, Lang::En)
        .map(str::to_owned)
        .or_else(|| driver_english(driver_label))
        .unwrap_or_else(|| humanize(name))
}

pub(super) fn option(feature: &str, name: &str, driver_label: &str) -> String {
    tables::contextual(feature, name, Lang::En)
        .or_else(|| tables::option(name, Lang::En))
        .map(str::to_owned)
        .or_else(|| driver_locale::english(driver_label))
        .or_else(|| driver_english(driver_label))
        .unwrap_or_else(|| {
            if is_opaque_option(name) {
                format!("Option {}", &name[1..])
            } else {
                humanize(name)
            }
        })
}

pub(super) fn parameter(name: &str, driver_label: &str) -> String {
    tables::parameter(name, Lang::En)
        .map(str::to_owned)
        .or_else(|| driver_english(driver_label))
        .unwrap_or_else(|| humanize(name))
}

fn driver_english(label: &str) -> Option<String> {
    let label = label.trim();
    (!label.is_empty() && !has_cyrillic(label)).then(|| label.to_owned())
}

fn humanize(value: &str) -> String {
    split_words(value).join(" ")
}
