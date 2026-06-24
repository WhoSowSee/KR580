use crate::backend::{GRAPHICS_HEIGHT, GRAPHICS_WIDTH, MonitorState, TEXT_COLS, TEXT_ROWS};
use iced::widget::canvas::{Cache, Geometry, Path, Program};
use iced::{Color, Rectangle, Renderer, Theme};

use crate::app::Message;
use crate::view::monitor_font::{CELL_HEIGHT, CELL_WIDTH, GLYPH_HEIGHT, GLYPH_WIDTH, pixel_lit};
use crate::view::theme::TOKYO_BOARD;

pub(super) struct PixelCanvas<'a> {
    pub(super) state: &'a MonitorState,
    pub(super) cache: Cache,
}

impl<'a> Program<Message> for PixelCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let bg = Path::rectangle(iced::Point::ORIGIN, frame.size());
            frame.fill(&bg, TOKYO_BOARD);

            let scale = (frame.width() / GRAPHICS_WIDTH as f32)
                .min(frame.height() / GRAPHICS_HEIGHT as f32);
            for &(x, y, intensity) in &self.state.pixels {
                if intensity == 0 {
                    continue;
                }
                let color = pixel_color(intensity);
                let path = Path::rectangle(
                    iced::Point::new(x as f32 * scale, y as f32 * scale),
                    iced::Size::new(scale.max(1.0), scale.max(1.0)),
                );
                frame.fill(&path, color);
            }
        });
        vec![geometry]
    }
}

pub(super) struct UnifiedCanvas<'a> {
    pub(super) state: &'a MonitorState,
    pub(super) cache: Cache,
}

impl<'a> Program<Message> for UnifiedCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let bg = Path::rectangle(iced::Point::ORIGIN, frame.size());
            frame.fill(&bg, TOKYO_BOARD);

            let scale = (frame.width() / GRAPHICS_WIDTH as f32)
                .min(frame.height() / GRAPHICS_HEIGHT as f32);

            for &(x, y, intensity) in &self.state.pixels {
                if intensity == 0 {
                    continue;
                }
                let color = pixel_color(intensity);
                let path = Path::rectangle(
                    iced::Point::new(x as f32 * scale, y as f32 * scale),
                    iced::Size::new(scale.max(1.0), scale.max(1.0)),
                );
                frame.fill(&path, color);
            }

            let cols = TEXT_COLS as usize;
            let rows = TEXT_ROWS as usize;
            let text_logical_w = (cols * CELL_WIDTH) as f32;
            let text_logical_h = (rows * CELL_HEIGHT) as f32;
            let text_scale = (frame.width() / text_logical_w).min(frame.height() / text_logical_h);
            let cell_w = CELL_WIDTH as f32;
            let cell_h = CELL_HEIGHT as f32;
            for r in 0..rows {
                for c in 0..cols {
                    let cell = match self.state.text_cells.get(r * cols + c) {
                        Some(cell) => cell,
                        None => continue,
                    };
                    if cell.ch == 0 {
                        continue;
                    }
                    let color = pixel_color(cell.color);
                    let origin_x = c as f32 * cell_w;
                    let origin_y = r as f32 * cell_h;
                    for gy in 0..GLYPH_HEIGHT {
                        for gx in 0..GLYPH_WIDTH {
                            if !pixel_lit(cell.ch, gx, gy) {
                                continue;
                            }
                            let px = (origin_x + gx as f32) * text_scale;
                            let py = (origin_y + gy as f32) * text_scale;
                            let path = Path::rectangle(
                                iced::Point::new(px, py),
                                iced::Size::new(text_scale.max(1.0), text_scale.max(1.0)),
                            );
                            frame.fill(&path, color);
                        }
                    }
                }
            }
        });
        vec![geometry]
    }
}

/// Mirrors the reference KP580 emulator's Delphi formula
/// `0xFFFFFF / 127 * intensity`, reinterpreted as a `TColor` (LE DWORD,
/// low byte = R). Carries between bytes scatter the channels into a
/// 128-step pseudo-coloured palette, not a grayscale ramp.
pub(super) fn pixel_color(intensity: u8) -> Color {
    let value = (intensity & 0x7F) as u32;
    let packed = (0xFF_FFFF_u32 / 127).wrapping_mul(value);
    let r = (packed & 0xFF) as f32 / 255.0;
    let g = ((packed >> 8) & 0xFF) as f32 / 255.0;
    let b = ((packed >> 16) & 0xFF) as f32 / 255.0;
    Color { r, g, b, a: 1.0 }
}

#[cfg(test)]
mod tests {
    use super::pixel_color;

    #[test]
    fn pixel_color_zero_is_black() {
        let c = pixel_color(0);
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn pixel_color_max_is_near_white() {
        let c = pixel_color(0x7F);
        assert_eq!((c.r * 255.0).round() as u32, 0xF8);
        assert_eq!((c.g * 255.0).round() as u32, 0xFF);
        assert_eq!((c.b * 255.0).round() as u32, 0xFF);
    }

    #[test]
    fn pixel_color_strips_high_bit() {
        let dim = pixel_color(0x40);
        let same = pixel_color(0xC0);
        assert_eq!(dim.r, same.r);
        assert_eq!(dim.g, same.g);
        assert_eq!(dim.b, same.b);
    }

    #[test]
    fn pixel_color_palette_is_not_grayscale() {
        let c = pixel_color(0x40);
        assert_eq!((c.r * 255.0).round() as u32, 0x00);
        assert_eq!((c.g * 255.0).round() as u32, 0x02);
        assert_eq!((c.b * 255.0).round() as u32, 0x81);
    }

    #[test]
    fn pixel_color_low_intensity_is_orange() {
        let c = pixel_color(0x10);
        assert_eq!((c.r * 255.0).round() as u32, 0x80);
        assert_eq!((c.g * 255.0).round() as u32, 0x40);
        assert_eq!((c.b * 255.0).round() as u32, 0x20);
    }
}
