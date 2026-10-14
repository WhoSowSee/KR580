use crate::app::Message;
use iced::widget::svg;
use iced::{Color, Element};
use std::sync::LazyLock;

pub(super) enum Kind {
    Refresh(f32),
    Download,
    Spinner(f32),
    Progress(f32),
    Check,
    Error,
}

// iced 0.14 tiny-skia clips externally rotated SVGs; rotate inside the fixed viewport.
static REFRESH: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
    let source = include_str!("../../../assets/icons/actions/refresh-ccw.svg");
    (0..60)
        .map(|frame| {
            handle(format!(
                "<g transform='rotate(-{} 12 12)'>{source}</g>",
                frame * 6
            ))
        })
        .collect()
});
static DOWNLOAD: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(
        include_bytes!("../../../assets/icons/actions/download.svg").as_slice(),
    )
});

static SPINNER: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
    (0..60).map(|frame| handle(format!(
        "<g transform='rotate({} 12 12)'><circle cx='12' cy='12' r='9' opacity='.2'/><path d='M12 3a9 9 0 1 1-8.56 6.22'/></g>", frame * 6
    ))).collect()
});
static CHECK: LazyLock<svg::Handle> = LazyLock::new(|| {
    handle("<circle cx='12' cy='12' r='9'/><path d='m7 12 3.5 3.5L17 9'/>".into())
});
static ERROR: LazyLock<svg::Handle> = LazyLock::new(|| {
    handle("<circle cx='12' cy='12' r='9'/><path d='M12 7v5'/><circle cx='12' cy='16.5' r='1.35' fill='currentColor' stroke='none'/>".into())
});
static PROGRESS: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
    (0..=100)
        .map(|percent| {
            let total = std::f32::consts::TAU * 9.0;
            let filled = total * percent as f32 / 100.0;
            let arc = if percent == 0 {
                String::new()
            } else {
                format!("<circle cx='12' cy='12' r='9' stroke-dasharray='{filled:.3} {total:.3}' transform='rotate(-90 12 12)'/>")
            };
            handle(format!("<circle cx='12' cy='12' r='9' opacity='.2'/>{arc}"))
        })
        .collect()
});

pub(super) fn symbol(kind: Kind, color: Color) -> Element<'static, Message> {
    let handle = match kind {
        Kind::Refresh(elapsed) => REFRESH[(elapsed.fract() * 60.0) as usize].clone(),
        Kind::Download => DOWNLOAD.clone(),
        Kind::Spinner(elapsed) => SPINNER[(elapsed.fract() * 60.0) as usize].clone(),
        Kind::Progress(fraction) => {
            PROGRESS[(fraction.clamp(0.0, 1.0) * 100.0).round() as usize].clone()
        }
        Kind::Check => CHECK.clone(),
        Kind::Error => ERROR.clone(),
    };
    svg(handle)
        .width(16)
        .height(16)
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

fn handle(body: String) -> svg::Handle {
    svg::Handle::from_memory(format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'>{body}</svg>"
    ).into_bytes())
}
