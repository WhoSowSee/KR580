use iced::widget::canvas::Cache;
use iced::widget::{Canvas, column, container, stack};
use iced::{Element, Length, Padding};
use k580_app::{MonitorState, TEXT_COLS, TEXT_ROWS};

use crate::app::Message;
use crate::i18n::{Key, Lang};
use crate::view::theme::{TOKYO_MUTED, TOKYO_TEXT, mono_text, ui_text};

use super::canvas::{PixelCanvas, UnifiedCanvas};
use super::styles::{framebuffer_padding, framebuffer_style};

pub(super) fn unified_screen_section<'a>(
    state: &'a MonitorState,
    lang: Lang,
) -> Element<'a, Message> {
    let empty = state.pixels.is_empty() && state.text_cells.iter().all(|c| c.ch == 0);

    let canvas: Element<'a, Message> = container(
        Canvas::new(UnifiedCanvas {
            state,
            cache: Cache::new(),
        })
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(framebuffer_padding(empty))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(framebuffer_style)
    .into();

    framed_layer(canvas, empty.then(|| lang.t(Key::MonitorUnifiedScreen)))
}

pub(super) fn pixel_layer_section<'a>(state: &'a MonitorState, lang: Lang) -> Element<'a, Message> {
    let empty = state.pixels.is_empty();

    let canvas: Element<'a, Message> = container(
        Canvas::new(PixelCanvas {
            state,
            cache: Cache::new(),
        })
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(framebuffer_padding(empty))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(framebuffer_style)
    .into();

    framed_layer(canvas, empty.then(|| lang.t(Key::MonitorPixelLayer)))
}

pub(super) fn text_layer_section<'a>(state: &MonitorState, lang: Lang) -> Element<'a, Message> {
    let cols = TEXT_COLS as usize;
    let rows = TEXT_ROWS as usize;
    let cells = &state.text_cells;

    let mut grid = column![].spacing(1);
    let mut empty = true;
    for r in 0..rows {
        let mut line = String::with_capacity(cols);
        for c in 0..cols {
            let idx = r * cols + c;
            let ch = cells.get(idx).map(|cell| cell.ch).unwrap_or(0);
            let glyph = if ch.is_ascii_graphic() || ch == b' ' {
                ch as char
            } else if ch == 0 {
                ' '
            } else {
                '·'
            };
            if ch != 0 && ch != b' ' {
                empty = false;
            }
            line.push(glyph);
        }
        grid = grid.push(mono_text(line, 13, TOKYO_TEXT));
    }

    let body: Element<'_, Message> = container(grid)
        .padding(framebuffer_padding(empty))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(framebuffer_style)
        .into();

    framed_layer(body, empty.then(|| lang.t(Key::MonitorTextLayer)))
}

fn framed_layer<'a>(canvas: Element<'a, Message>, title: Option<&'a str>) -> Element<'a, Message> {
    let Some(title) = title else {
        return canvas;
    };

    let labels = container(ui_text(title, 11, TOKYO_MUTED))
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .width(Length::Fill);

    stack![canvas, labels]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
