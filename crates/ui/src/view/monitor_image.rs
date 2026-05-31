//! PNG snapshot of the KR580 monitor.

use std::io::Cursor;

use k580_app::{GRAPHICS_HEIGHT, GRAPHICS_WIDTH, MonitorState, TEXT_COLS, TEXT_ROWS};

use super::monitor_font::{CELL_HEIGHT, CELL_WIDTH, GLYPH_HEIGHT, GLYPH_WIDTH, pixel_lit};

fn intensity_to_rgb(intensity: u8) -> [u8; 3] {
    let value = (intensity & 0x7F) as u32;
    let packed = (0xFF_FFFF_u32 / 127).wrapping_mul(value);
    let r = (packed & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = ((packed >> 16) & 0xFF) as u8;
    [r, g, b]
}

pub(crate) fn render_monitor_png(state: &MonitorState) -> Result<Vec<u8>, String> {
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
            .write_image_data(&buf)
            .map_err(|e| format!("png data: {e}"))?;
    }
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
    fn empty_monitor_renders_to_valid_png() {
        let png = render_monitor_png(&empty_state()).expect("encodes");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn lit_pixel_produces_non_black_output() {
        let mut state = empty_state();
        state.pixels.push((0, 0, 0x40));
        let png = render_monitor_png(&state).expect("encodes");
        let empty = render_monitor_png(&empty_state()).unwrap();
        assert!(
            png.len() != empty.len() || png != empty,
            "lit pixel should change PNG output"
        );
    }
}
