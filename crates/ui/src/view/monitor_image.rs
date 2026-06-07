use std::io::Cursor;

use k580_app::{GRAPHICS_HEIGHT, GRAPHICS_WIDTH, MonitorState, TEXT_COLS, TEXT_ROWS};

use super::monitor_font::{CELL_HEIGHT, CELL_WIDTH, GLYPH_HEIGHT, GLYPH_WIDTH, pixel_lit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MonitorImageFormat {
    Png,
    Jpeg,
    WebP,
    Bmp,
}

impl MonitorImageFormat {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
        }
    }
}

fn intensity_to_rgb(intensity: u8) -> [u8; 3] {
    let value = (intensity & 0x7F) as u32;
    let packed = (0xFF_FFFF_u32 / 127).wrapping_mul(value);
    let r = (packed & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = ((packed >> 16) & 0xFF) as u8;
    [r, g, b]
}

fn render_rgb_buffer(state: &MonitorState) -> (Vec<u8>, usize, usize) {
    let text_w = TEXT_COLS as usize * CELL_WIDTH;
    let text_h = TEXT_ROWS as usize * CELL_HEIGHT;
    let width = (GRAPHICS_WIDTH as usize).max(text_w);
    let height = (GRAPHICS_HEIGHT as usize).max(text_h);

    let mut buf = vec![0u8; width * height * 3];

    for &(x, y, intensity) in &state.pixels {
        if intensity == 0 {
            continue;
        }
        let px = x as usize;
        let py = y as usize;
        if px >= width || py >= height {
            continue;
        }
        let off = (py * width + px) * 3;
        let rgb = intensity_to_rgb(intensity);
        buf[off] = rgb[0];
        buf[off + 1] = rgb[1];
        buf[off + 2] = rgb[2];
    }

    let cols = TEXT_COLS as usize;
    let rows = TEXT_ROWS as usize;
    for r in 0..rows {
        for c in 0..cols {
            let cell = match state.text_cells.get(r * cols + c) {
                Some(cell) => cell,
                None => continue,
            };
            if cell.ch == 0 {
                continue;
            }
            let rgb = intensity_to_rgb(cell.color);
            let origin_x = c * CELL_WIDTH;
            let origin_y = r * CELL_HEIGHT;
            for gy in 0..GLYPH_HEIGHT {
                for gx in 0..GLYPH_WIDTH {
                    if !pixel_lit(cell.ch, gx, gy) {
                        continue;
                    }
                    let px = origin_x + gx;
                    let py = origin_y + gy;
                    if px >= width || py >= height {
                        continue;
                    }
                    let off = (py * width + px) * 3;
                    buf[off] = rgb[0];
                    buf[off + 1] = rgb[1];
                    buf[off + 2] = rgb[2];
                }
            }
        }
    }

    (buf, width, height)
}

pub(crate) fn render_monitor_image(
    state: &MonitorState,
    format: MonitorImageFormat,
) -> Result<Vec<u8>, String> {
    let (buf, width, height) = render_rgb_buffer(state);

    match format {
        MonitorImageFormat::Png => encode_png(&buf, width, height),
        MonitorImageFormat::Jpeg => encode_jpeg(&buf, width, height),
        MonitorImageFormat::WebP => encode_webp(&buf, width, height),
        MonitorImageFormat::Bmp => encode_bmp(&buf, width, height),
    }
}

fn encode_png(buf: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(buf.len() / 4);
    {
        let cursor = Cursor::new(&mut out);
        let mut encoder = png::Encoder::new(cursor, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("png header: {e}"))?;
        writer
            .write_image_data(buf)
            .map_err(|e| format!("png data: {e}"))?;
    }
    Ok(out)
}

fn encode_jpeg(buf: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90);
    encoder
        .encode(buf, width as u32, height as u32, image::ColorType::Rgb8.into())
        .map_err(|e| format!("jpeg: {e}"))?;
    Ok(out)
}

fn encode_webp(buf: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut out);
    encoder
        .encode(buf, width as u32, height as u32, image::ColorType::Rgb8.into())
        .map_err(|e| format!("webp: {e}"))?;
    Ok(out)
}

fn encode_bmp(buf: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut out);
    encoder
        .encode(buf, width as u32, height as u32, image::ColorType::Rgb8.into())
        .map_err(|e| format!("bmp: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k580_app::{DeviceStatus, MonitorPhase, TextCell};

    fn empty_state() -> MonitorState {
        MonitorState {
            text_cells: vec![TextCell::default(); (TEXT_COLS as usize) * (TEXT_ROWS as usize)],
            text_cursor: 0,
            pixels: Vec::new(),
            phase: MonitorPhase::default(),
            last_command: None,
            hex_buffer: Vec::new(),
            status: DeviceStatus::Ready,
        }
    }

    #[test]
    fn all_formats_encode_without_error() {
        let state = empty_state();
        for format in [
            MonitorImageFormat::Png,
            MonitorImageFormat::Jpeg,
            MonitorImageFormat::WebP,
            MonitorImageFormat::Bmp,
        ] {
            let data = render_monitor_image(&state, format).expect("encode");
            assert!(!data.is_empty(), "{format:?} produced empty output");
        }
    }

    #[test]
    fn png_has_valid_header() {
        let png = render_monitor_image(&empty_state(), MonitorImageFormat::Png).expect("encodes");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn jpeg_has_valid_header() {
        let jpg = render_monitor_image(&empty_state(), MonitorImageFormat::Jpeg).expect("encodes");
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn bmp_has_valid_header() {
        let bmp = render_monitor_image(&empty_state(), MonitorImageFormat::Bmp).expect("encodes");
        assert_eq!(&bmp[..2], b"BM");
    }

    #[test]
    fn webp_has_valid_header() {
        let webp = render_monitor_image(&empty_state(), MonitorImageFormat::WebP).expect("encodes");
        assert_eq!(&webp[..4], b"RIFF");
    }
}
