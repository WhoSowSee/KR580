//! Visual tokens shared by every view module.
//!
//! Splitting the colours and font handles out of the rest of the UI keeps
//! the styling-heavy modules from re-defining them, and gives the rest of
//! the codebase a single place to look up a swatch when the design tweaks.

use iced::widget::{Text, text};
use iced::{Color, Font};

pub(super) const UI_FONT: Font = Font::with_name("Segoe UI Variable");
pub(super) const MONO_FONT: Font = Font::MONOSPACE;

pub(super) const TOKYO_BG: Color = Color::from_rgb8(0x1A, 0x1B, 0x26);
pub(super) const TOKYO_BOARD: Color = Color::from_rgb8(0x12, 0x13, 0x20);
pub(super) const TOKYO_SURFACE: Color = Color::from_rgb8(0x1D, 0x20, 0x30);
pub(super) const TOKYO_SURFACE_2: Color = Color::from_rgb8(0x2F, 0x33, 0x4D);
pub(super) const TOKYO_SURFACE_3: Color = Color::from_rgb8(0x36, 0x3B, 0x59);
pub(super) const TOKYO_BORDER: Color = Color::from_rgb8(0x41, 0x48, 0x68);
pub(super) const TOKYO_TEXT: Color = Color::from_rgb8(0xC0, 0xCA, 0xF5);
pub(super) const TOKYO_MUTED: Color = Color::from_rgb8(0x56, 0x5F, 0x89);
pub(super) const TOKYO_BLUE: Color = Color::from_rgb8(0x7A, 0xA2, 0xF7);
pub(super) const TOKYO_SELECTION_BLUE: Color = Color {
    r: 0x7A as f32 / 255.0,
    g: 0xA2 as f32 / 255.0,
    b: 0xF7 as f32 / 255.0,
    a: 0.18,
};
pub(super) const TOKYO_CYAN: Color = Color::from_rgb8(0x7D, 0xCF, 0xFF);
pub(super) const TOKYO_GREEN: Color = Color::from_rgb8(0x9E, 0xCE, 0x6A);
pub(super) const TOKYO_YELLOW: Color = Color::from_rgb8(0xE0, 0xAF, 0x68);
pub(super) const TOKYO_RED: Color = Color::from_rgb8(0xF7, 0x76, 0x8E);
pub(super) const TOKYO_MAGENTA: Color = Color::from_rgb8(0xBB, 0x9A, 0xF7);

/// Builds a UI-font text widget with the given size and colour.
pub(super) fn ui_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(UI_FONT).size(size).color(color)
}

/// Builds a monospaced text widget with the given size and colour. Used for
/// every register/memory readout that should align by column.
pub(super) fn mono_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(MONO_FONT).size(size).color(color)
}
