use super::{style, widgets};
use iced::widget::{Space, button as iced_button, column, container, mouse_area, row, svg, text};
use iced::{Alignment, Element, Length, alignment};
use std::sync::LazyLock;

const ICON_SIZE: f32 = 14.0;
const CLOSE_ICON_SIZE: f32 = 16.0;
const BUTTON_WIDTH: f32 = 32.0;
const BUTTON_HEIGHT: f32 = 24.0;

static MINIMIZE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-minimize").as_slice()));
static MAXIMIZE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-maximize").as_slice()));
static RESTORE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-restore").as_slice()));
static CLOSE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("window-close").as_slice()));
static CPU: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("cpu").as_slice()));
pub(super) static CHIP: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(action_icon_bytes!("installer-chip").as_slice()));

#[derive(Clone, Copy)]
enum CaptionKind {
    Neutral,
    Close,
}

pub(super) struct CaptionMessages<M> {
    pub(super) drag: M,
    pub(super) minimize: M,
    pub(super) toggle_maximize: M,
    pub(super) close: Option<M>,
}

pub(super) fn title_bar<M: Clone + 'static>(
    title: &'static str,
    maximized: bool,
    messages: CaptionMessages<M>,
) -> Element<'static, M> {
    let title = row![
        svg(CPU.clone())
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .style(|_theme, _status| svg::Style {
                color: Some(style::TEXT),
            }),
        text(title)
            .font(style::FONT_BOLD)
            .size(13)
            .color(style::TEXT),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let drag_handle: Element<'static, M> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(messages.drag)
    .into();

    let body = row![
        container(title)
            .padding(iced::Padding::ZERO.left(11))
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center),
        drag_handle,
        button(
            MINIMIZE.clone(),
            Some(messages.minimize),
            CaptionKind::Neutral
        ),
        button(
            if maximized {
                RESTORE.clone()
            } else {
                MAXIMIZE.clone()
            },
            Some(messages.toggle_maximize),
            CaptionKind::Neutral,
        ),
        button(CLOSE.clone(), messages.close, CaptionKind::Close),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    column![
        container(body)
            .height(Length::Fixed(34.0))
            .width(Length::Fill),
        widgets::horizontal_rule(),
    ]
    .into()
}

fn button<M: Clone + 'static>(
    icon: svg::Handle,
    message: Option<M>,
    kind: CaptionKind,
) -> Element<'static, M> {
    let size = match kind {
        CaptionKind::Neutral => ICON_SIZE,
        CaptionKind::Close => CLOSE_ICON_SIZE,
    };
    let glyph = svg(icon)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(|_theme, _status| svg::Style {
            color: Some(style::TEXT),
        });

    iced_button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(BUTTON_WIDTH))
    .height(Length::Fixed(BUTTON_HEIGHT))
    .style(move |_theme, status| match kind {
        CaptionKind::Neutral => style::caption_button(status),
        CaptionKind::Close => style::close_caption_button(status),
    })
    .on_press_maybe(message)
    .into()
}

pub(super) fn platform_label() -> String {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let mut os = std::env::consts::OS.to_owned();
    if let Some(first) = os.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    format!("{os} · {arch}")
}

#[cfg(test)]
mod tests {
    use super::platform_label;

    #[test]
    fn platform_label_capitalizes_only_the_os_name_start() {
        let os = std::env::consts::OS;
        let expected = format!("{}{}", os[..1].to_uppercase(), &os[1..]);
        assert!(platform_label().starts_with(&format!("{expected} ·")));
    }
}
