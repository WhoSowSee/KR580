use iced::widget::{
    Space, button, column, container, mouse_area, opaque, row, scrollable, stack, svg, tooltip,
};
use iced::{Background, Border, Color, Element, Length, Padding, Theme, alignment};
use k580_app::{DeviceStatus, StorageState};
use std::time::Duration;

use super::icons;
use super::styles::{inset_style, scrollable_style};
use super::theme::{
    MONO_FONT, TOKYO_BLUE, TOKYO_BOARD, TOKYO_BORDER, TOKYO_MUTED, TOKYO_SELECTION_BLUE,
    TOKYO_SURFACE, TOKYO_SURFACE_2, TOKYO_TEXT, ui_text,
};
use crate::app::Message;
use crate::i18n::{Key, Lang};

mod text;

use text::storage_buffer_text;

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_GLYPH_SIZE: f32 = 18.0;
const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 340.0;

pub(in crate::view) fn floppy_window_overlay<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
) -> Element<'a, Message> {
    let backdrop: Element<'_, Message> = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(backdrop_style),
    )
    .on_press(Message::CloseFloppy)
    .into();

    let body = column![
        header(state, show_image_contents, lang),
        Space::new().height(Length::Fixed(12.0)),
        dialog_body(
            state,
            show_image_contents,
            image_contents,
            image_error,
            lang
        ),
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

fn header<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    lang: Lang,
) -> Element<'a, Message> {
    let debug_icon = if state.debug_buffer {
        icons::bug()
    } else {
        icons::bug_off()
    };

    row![
        Space::new().width(Length::Fill),
        icon_button(
            icons::hard_drive_download(),
            Message::OpenFloppyImage,
            lang.t(Key::FloppyOpenImage),
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_upload(),
            Message::SaveFloppyBuffer,
            lang.t(Key::FloppySaveBuffer),
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::hard_drive_x(),
            Message::DetachFloppyImage,
            lang.t(Key::FloppyDetachImage),
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::binary(),
            Message::ToggleFloppyImageContents,
            lang.t(Key::FloppyShowImageContents),
            show_image_contents,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            debug_icon,
            Message::ToggleFloppyDebugBuffer,
            lang.t(Key::FloppyDebugBuffer),
            state.debug_buffer,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::brush_cleaning(),
            Message::ClearFloppyBuffer,
            lang.t(Key::FloppyClearBuffer),
            false,
        ),
        Space::new().width(Length::Fixed(6.0)),
        icon_button(
            icons::window_close(),
            Message::CloseFloppy,
            lang.t(Key::MonitorClose),
            false,
        ),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

fn dialog_body<'a>(
    state: &'a StorageState,
    show_image_contents: bool,
    image_contents: &'a [u8],
    image_error: Option<&'a str>,
    lang: Lang,
) -> Element<'a, Message> {
    let (bytes, title_key, error) = if show_image_contents {
        (image_contents, Key::FloppyImageContent, image_error)
    } else {
        (state.visible_buffer.as_slice(), Key::FloppyContent, None)
    };
    let image_path_missing = show_image_contents && state.path.is_none();
    let text = error
        .filter(|_| !image_path_missing)
        .map(str::to_owned)
        .unwrap_or_else(|| storage_buffer_text(bytes));
    let empty = text.is_empty();
    let label = if image_path_missing {
        Some(lang.t(Key::FloppyImagePathMissing))
    } else {
        empty.then(|| lang.t(title_key))
    };
    let buffer = scrollable(
        container(
            iced::widget::text(text)
                .font(MONO_FONT)
                .size(15)
                .color(TOKYO_TEXT)
                .wrapping(iced::widget::text::Wrapping::None),
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
        footer(state, lang),
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

fn footer<'a>(state: &'a StorageState, lang: Lang) -> Element<'a, Message> {
    let path = state
        .path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| lang.t(Key::FloppyPathMissing).to_owned());
    let status = status_label(&state.status, lang);
    let mut meta = format!(
        "{}: {status}   {}: {path}   {}: {}",
        lang.t(Key::FloppyStatus),
        lang.t(Key::FloppyPath),
        lang.t(Key::FloppyBytesQueued),
        state.bytes_queued,
    );
    if let Some(error) = state.last_error.as_deref() {
        meta.push_str("   ");
        meta.push_str(error);
    }

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

fn icon_button(
    handle: svg::Handle,
    on_press: Message,
    hint: &'static str,
    active: bool,
) -> Element<'static, Message> {
    let glyph_color = if active { TOKYO_BLUE } else { TOKYO_TEXT };
    let glyph = svg(handle)
        .width(Length::Fixed(ICON_GLYPH_SIZE))
        .height(Length::Fixed(ICON_GLYPH_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let face = button(
        container(glyph)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(on_press)
    .padding(0)
    .width(Length::Fixed(ICON_BUTTON_SIZE))
    .height(Length::Fixed(ICON_BUTTON_SIZE))
    .style(move |_theme, status| icon_button_style(status, active));

    tooltip_body(face.into(), hint)
}

fn tooltip_body(face: Element<'static, Message>, hint: &'static str) -> Element<'static, Message> {
    let body = container(ui_text(hint.to_owned(), 12, TOKYO_TEXT))
        .padding([4, 8])
        .style(inset_style);

    tooltip(face, body, tooltip::Position::Bottom)
        .gap(4.0)
        .padding(0.0)
        .delay(Duration::from_millis(450))
        .snap_within_viewport(true)
        .into()
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
    let background = match (status, active) {
        (iced::widget::button::Status::Pressed, _) => TOKYO_SURFACE_2,
        (iced::widget::button::Status::Hovered, _) => TOKYO_SURFACE,
        (_, true) => TOKYO_SELECTION_BLUE,
        _ => TOKYO_BOARD,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: TOKYO_TEXT,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if active { TOKYO_BLUE } else { TOKYO_BORDER },
        },
        ..button::Style::default()
    }
}
