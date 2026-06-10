mod floppy;
mod hdd;
mod text;

pub(in crate::view) use floppy::floppy_window_overlay;
pub(in crate::view) use hdd::hdd_window_overlay;

use iced::widget::{
    Space, button, column, container, mouse_area, opaque, row, scrollable, stack, svg,
};
use iced::{Background, Border, Color, Element, Length, Padding, Theme, alignment};
use k580_app::{DeviceStatus, StorageState};
use std::time::Duration;

use super::styles::scrollable_style;
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_MUTED, TOKYO_SELECTION_BLUE,
    TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT, ui_text,
};
use super::tooltips::hover_tooltip;
use crate::app::Message;
use crate::i18n::{Key, Lang};
use text::storage_buffer_text;

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_GLYPH_SIZE: f32 = 18.0;
const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 340.0;

pub(super) struct StorageKeys {
    pub(super) content: Key,
    pub(super) image_content: Key,
    pub(super) status: Key,
    pub(super) path: Key,
    pub(super) path_missing: Key,
    pub(super) image_path_missing: Key,
    pub(super) bytes_queued: Key,
    pub(super) debug_enabled: Key,
}

pub(super) const FLOPPY_KEYS: StorageKeys = StorageKeys {
    content: Key::FloppyContent,
    image_content: Key::FloppyImageContent,
    status: Key::FloppyStatus,
    path: Key::FloppyPath,
    path_missing: Key::FloppyPathMissing,
    image_path_missing: Key::FloppyImagePathMissing,
    bytes_queued: Key::FloppyBytesQueued,
    debug_enabled: Key::FloppyDebugEnabled,
};

pub(super) const HDD_KEYS: StorageKeys = StorageKeys {
    content: Key::HddContent,
    image_content: Key::HddImageContent,
    status: Key::HddStatus,
    path: Key::HddPath,
    path_missing: Key::HddPathMissing,
    image_path_missing: Key::HddImagePathMissing,
    bytes_queued: Key::HddBytesQueued,
    debug_enabled: Key::HddDebugEnabled,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn storage_window_overlay<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
    close_msg: Message,
    header_fn: impl FnOnce(&'a StorageState, bool, Lang) -> Element<'a, Message>,
    keys: StorageKeys,
) -> Element<'a, Message> {
    let backdrop: Element<'_, Message> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(backdrop_style),
    )
    .on_press(close_msg)
    .into();

    let body = column![
        header_fn(state, show_image_contents, lang),
        Space::new().height(Length::Fixed(12.0)),
        dialog_body(state, show_image_contents, image_contents, image_error, lang, keys),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let dialog = container(body)
        .padding(16)
        .style(dialog_style)
        .width(Length::Fixed(WINDOW_WIDTH))
        .height(Length::Fixed(WINDOW_HEIGHT));

    let centered = column![
        Space::new().height(Length::FillPortion(1)),
        row![
            Space::new().width(Length::FillPortion(1)),
            opaque(dialog),
            Space::new().width(Length::FillPortion(1)),
        ]
        .align_y(alignment::Vertical::Center),
        Space::new().height(Length::FillPortion(1)),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn dialog_body<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
    keys: StorageKeys,
) -> Element<'a, Message> {
    let (bytes, title_key, error) = if show_image_contents {
        (image_contents, keys.image_content, image_error)
    } else {
        (state.visible_buffer.as_slice(), keys.content, None)
    };
    let image_path_missing = show_image_contents && state.path.is_none();
    let text = error
        .filter(|_| !image_path_missing)
        .map(str::to_owned)
        .unwrap_or_else(|| storage_buffer_text(bytes));
    let empty = text.is_empty();
    let label = if image_path_missing {
        Some(lang.t(keys.image_path_missing))
    } else {
        empty.then(|| lang.t(title_key))
    };
    let buffer = scrollable(
        container(
            iced::widget::text(text)
                .font(MONO_FONT)
                .size(15)
                .color(TOKYO_TEXT)
                .wrapping(iced::widget::text::Wrapping::Glyph),
        )
        .padding(buffer_padding(empty))
        .width(Length::Fill)
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme, status| scrollable_style(true, theme, status));

    let buffer_frame = container(buffer)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(buffer_style)
        .clip(true);

    column![
        framed_buffer(buffer_frame.into(), label),
        storage_footer(state, lang, keys),
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn framed_buffer<'a>(buffer: Element<'a, Message>, title: Option<&'a str>) -> Element<'a, Message> {
    let Some(title) = title else {
        return buffer;
    };

    let label = container(ui_text(title, 13, TOKYO_MUTED))
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .width(Length::Fill);

    stack![buffer, label]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn storage_footer<'a>(
    state: &'a StorageState,
    lang: Lang,
    keys: StorageKeys,
) -> Element<'a, Message> {
    let path = if state.debug_buffer {
        lang.t(keys.debug_enabled).to_owned()
    } else {
        state
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| lang.t(keys.path_missing).to_owned())
    };
    let status = status_label(&state.status, lang);
    let meta = format!(
        "{}: {status}   {}: {path}   {}: {}",
        lang.t(keys.status),
        lang.t(keys.path),
        lang.t(keys.bytes_queued),
        state.bytes_queued,
    );

    row![
        container(
            iced::widget::text(meta)
                .font(MONO_FONT)
                .size(12)
                .color(TOKYO_TEXT)
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .width(Length::Fill)
        .clip(true),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

pub(super) fn icon_button(
    handle: svg::Handle,
    on_press: Option<Message>,
    hint: &'static str,
    active: bool,
    shortcut: Option<&'static str>,
) -> Element<'static, Message> {
    let is_disabled = on_press.is_none() && !active;
    let glyph_color = if active {
        TOKYO_BLUE
    } else if is_disabled {
        TOKYO_MUTED
    } else {
        TOKYO_TEXT
    };
    let glyph = svg(handle)
        .width(Length::Fixed(ICON_GLYPH_SIZE))
        .height(Length::Fixed(ICON_GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let mut btn = button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    );
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    let face = btn
        .padding(0)
        .width(Length::Fixed(ICON_BUTTON_SIZE))
        .height(Length::Fixed(ICON_BUTTON_SIZE))
        .style(move |_theme, status| icon_button_style(status, active));

    hover_tooltip(
        face.into(),
        hint,
        shortcut,
        iced::widget::tooltip::Position::Bottom,
        Duration::from_millis(450),
    )
}

fn status_label(status: &DeviceStatus, lang: Lang) -> String {
    match status {
        DeviceStatus::Ready => lang.t(Key::DeviceStatusReady).to_owned(),
        DeviceStatus::NotReady => lang.t(Key::DeviceStatusNotReady).to_owned(),
        DeviceStatus::Busy => lang.t(Key::DeviceStatusBusy).to_owned(),
        DeviceStatus::NoData => lang.t(Key::DeviceStatusNoData).to_owned(),
        DeviceStatus::Connected => lang.t(Key::DeviceStatusConnected).to_owned(),
        DeviceStatus::Listening => lang.t(Key::DeviceStatusListening).to_owned(),
        DeviceStatus::Disconnected => lang.t(Key::DeviceStatusDisconnected).to_owned(),
        DeviceStatus::Error(error) => error.clone(),
    }
}

fn backdrop_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba8(0x12, 0x12, 0x21, 0.85))),
        ..iced::widget::container::Style::default()
    }
}

fn dialog_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

fn buffer_style(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(TOKYO_TEXT),
        background: Some(Background::Color(TOKYO_BOARD)),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: TOKYO_BORDER,
        },
        ..iced::widget::container::Style::default()
    }
}

fn buffer_padding(empty: bool) -> Padding {
    Padding {
        top: if empty { 34.0 } else { 12.0 },
        right: 12.0,
        bottom: 12.0,
        left: 12.0,
    }
}

fn icon_button_style(status: iced::widget::button::Status, active: bool) -> button::Style {
    let disabled = matches!(status, button::Status::Disabled) && !active;
    let background = match (status, active) {
        (iced::widget::button::Status::Pressed, _) if !disabled => TOKYO_SURFACE_2,
        (iced::widget::button::Status::Hovered, _) if !disabled => TOKYO_SURFACE,
        (_, true) if !disabled => TOKYO_SELECTION_BLUE,
        _ => TOKYO_BOARD,
    };
    let border_color = if disabled {
        Color {
            a: 0.35,
            ..TOKYO_BORDER
        }
    } else if active {
        TOKYO_BLUE
    } else {
        TOKYO_BORDER
    };
    let text_color = if disabled { TOKYO_MUTED } else { TOKYO_TEXT };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..button::Style::default()
    }
}
