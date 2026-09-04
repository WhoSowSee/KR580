mod cp866;

pub(super) const GLYPH_WIDTH: usize = 8;
pub(super) const GLYPH_HEIGHT: usize = 8;
pub(super) const CELL_WIDTH: usize = GLYPH_WIDTH;
pub(super) const CELL_HEIGHT: usize = 12;

pub(super) fn pixel_lit(code: u8, col: usize, row: usize) -> bool {
    let shift = GLYPH_WIDTH * GLYPH_HEIGHT - 1 - row * GLYPH_WIDTH - col;
    cp866::GLYPHS[code as usize] >> shift & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cp866_defines_every_non_blank_byte() {
        for code in 0..=u8::MAX {
            let blank = matches!(code, 0x00 | 0x20 | 0xFF);
            assert_eq!(cp866::GLYPHS[code as usize] == 0, blank, "{code:02X}");
        }
    }

    #[test]
    fn cp866_cyrillic_glyphs_are_distinct_and_msb_first() {
        assert_ne!(cp866::GLYPHS[0x8C], cp866::GLYPHS[0x88]);
        assert_ne!(cp866::GLYPHS[0x88], cp866::GLYPHS[0x8A]);
        assert!(pixel_lit(0x08, 0, 0));
        assert!(!pixel_lit(0x08, 4, 0));
    }
}
