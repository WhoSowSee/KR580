use crate::backend::{MonitorState, TextCell};
use iced::widget::canvas::Cache;
use iced::widget::{Canvas, container, stack, text::Wrapping};
use iced::{Element, Length, Padding};

use crate::app::Message;
use crate::i18n::{Key, Lang};
use crate::view::theme::{mono_text, tokyo_muted, tokyo_text, ui_text};

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
    let (text, empty) = text_layer_text(&state.text_cells);

    let body: Element<'_, Message> = container(
        mono_text(text, 13, tokyo_text())
            .width(Length::Fill)
            .wrapping(Wrapping::Glyph),
    )
    .padding(framebuffer_padding(empty))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(framebuffer_style)
    .into();

    framed_layer(body, empty.then(|| lang.t(Key::MonitorTextLayer)))
}

fn text_layer_text(cells: &[TextCell]) -> (String, bool) {
    let visible_len = cells
        .iter()
        .rposition(|cell| cell.ch != 0)
        .map_or(0, |index| index + 1);
    let mut text = String::with_capacity(visible_len);
    let mut empty = true;

    for cell in &cells[..visible_len] {
        let glyph = if cell.ch.is_ascii_graphic() || cell.ch == b' ' {
            cell.ch as char
        } else if cell.ch == 0 {
            ' '
        } else {
            '·'
        };
        if cell.ch != 0 && cell.ch != b' ' {
            empty = false;
        }
        text.push(glyph);
    }

    (text, empty)
}

fn framed_layer<'a>(canvas: Element<'a, Message>, title: Option<&'a str>) -> Element<'a, Message> {
    let Some(title) = title else {
        return canvas;
    };

    let labels = container(ui_text(title, 11, tokyo_muted()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_only_layer_does_not_insert_framebuffer_row_breaks() {
        let cells: Vec<TextCell> = (b'a'..=b'z')
            .cycle()
            .take(104)
            .map(|ch| TextCell { ch, color: 0x40 })
            .collect();

        let (text, empty) = text_layer_text(&cells);

        assert!(!empty);
        assert_eq!(text.len(), 104);
        assert!(!text.contains('\n'));
    }
}
