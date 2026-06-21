use super::{Key, Lang};

pub(super) fn stack_view_area_label(lang: Lang, active: bool) -> &'static str {
    if !active {
        return lang.t(Key::ViewStackArea);
    }

    match lang {
        Lang::Ru => "Скрыть стековую область памяти",
        Lang::En => "Hide stack memory area",
    }
}
