use crate::i18n::Lang;
use crate::persistence::ColorScheme;
use iced::theme::Palette;
use iced::widget::{Text, text};
use iced::{Color, Font, Theme, font, theme};
use std::sync::atomic::{AtomicU8, Ordering};

pub(super) const UI_FONT: Font = Font::with_name("Segoe UI Variable");
pub(super) const UI_BOLD_FONT: Font = Font {
    weight: font::Weight::Bold,
    ..UI_FONT
};
pub(super) const MONO_FONT: Font = Font::MONOSPACE;

pub(crate) const DARK_COLOR_SCHEMES: [ColorScheme; 7] = [
    ColorScheme::TokyoNight,
    ColorScheme::BlackWhiteDark,
    ColorScheme::KanagawaWave,
    ColorScheme::CatppuccinMocha,
    ColorScheme::Nord,
    ColorScheme::GruvboxDark,
    ColorScheme::MaterialOcean,
];

pub(crate) const LIGHT_COLOR_SCHEMES: [ColorScheme; 5] = [
    ColorScheme::TokyoNightLight,
    ColorScheme::BlackWhiteLight,
    ColorScheme::KanagawaLotus,
    ColorScheme::CatppuccinLatte,
    ColorScheme::GruvboxLight,
];

static ACTIVE_COLOR_SCHEME: AtomicU8 = AtomicU8::new(ColorScheme::TokyoNight as u8);

#[derive(Clone, Copy)]
pub(crate) struct UiTokens {
    pub(crate) board: Color,
    pub(crate) surface: Color,
    pub(crate) surface_2: Color,
    pub(crate) surface_3: Color,
    pub(crate) border: Color,
    pub(crate) text: Color,
    pub(crate) text_selection: Color,
    pub(crate) muted: Color,
    pub(crate) blue: Color,
    pub(crate) selection_blue: Color,
    pub(crate) cyan: Color,
    pub(crate) green: Color,
    pub(crate) yellow: Color,
    pub(crate) red: Color,
    pub(crate) magenta: Color,
}

pub(crate) fn set_active_color_scheme(scheme: ColorScheme) {
    ACTIVE_COLOR_SCHEME.store(scheme.index(), Ordering::Relaxed);
}

pub(crate) fn iced_theme_for_scheme(scheme: ColorScheme) -> Theme {
    match scheme {
        ColorScheme::TokyoNight => Theme::TokyoNight,
        ColorScheme::TokyoNightLight => Theme::TokyoNightLight,
        ColorScheme::BlackWhiteDark => Theme::custom("Black & White Dark", black_white_dark()),
        ColorScheme::BlackWhiteLight => Theme::custom("Black & White Light", black_white_light()),
        ColorScheme::KanagawaWave => Theme::KanagawaWave,
        ColorScheme::KanagawaLotus => Theme::KanagawaLotus,
        ColorScheme::CatppuccinMocha => Theme::CatppuccinMocha,
        ColorScheme::CatppuccinLatte => Theme::custom("Catppuccin Latte", catppuccin_latte()),
        ColorScheme::Nord => Theme::Nord,
        ColorScheme::GruvboxDark => Theme::GruvboxDark,
        ColorScheme::GruvboxLight => Theme::GruvboxLight,
        ColorScheme::MaterialOcean => Theme::custom("Material Ocean", material_ocean()),
    }
}

pub(crate) fn app_base_style(scheme: ColorScheme) -> theme::Style {
    let tokens = scheme_tokens(scheme);
    theme::Style {
        background_color: tokens.board,
        text_color: tokens.text,
    }
}

pub(crate) fn color_scheme_label(scheme: ColorScheme, lang: Lang) -> &'static str {
    match (scheme, lang) {
        (ColorScheme::TokyoNight, _) => "Tokyo Night",
        (ColorScheme::TokyoNightLight, _) => "Tokyo Night Light",
        (ColorScheme::BlackWhiteDark, Lang::Ru) => "Черно-белая темная",
        (ColorScheme::BlackWhiteDark, Lang::En) => "Black & White Dark",
        (ColorScheme::BlackWhiteLight, Lang::Ru) => "Черно-белая светлая",
        (ColorScheme::BlackWhiteLight, Lang::En) => "Black & White Light",
        (ColorScheme::KanagawaWave, _) => "Kanagawa Wave",
        (ColorScheme::KanagawaLotus, _) => "Kanagawa Lotus",
        (ColorScheme::CatppuccinMocha, _) => "Catppuccin Mocha",
        (ColorScheme::CatppuccinLatte, _) => "Catppuccin Latte",
        (ColorScheme::Nord, _) => "Nord",
        (ColorScheme::GruvboxDark, _) => "Gruvbox Dark",
        (ColorScheme::GruvboxLight, _) => "Gruvbox Light",
        (ColorScheme::MaterialOcean, _) => "Material Ocean",
    }
}

pub(crate) fn color_scheme_group_label(dark: bool, lang: Lang) -> &'static str {
    match (dark, lang) {
        (true, Lang::Ru) => "Темные",
        (false, Lang::Ru) => "Светлые",
        (true, Lang::En) => "Dark",
        (false, Lang::En) => "Light",
    }
}

pub(crate) fn color_scheme_palette(scheme: ColorScheme) -> [Color; 6] {
    let tokens = scheme_tokens(scheme);
    [
        tokens.board,
        tokens.surface,
        tokens.text,
        tokens.blue,
        tokens.green,
        tokens.red,
    ]
}

pub(crate) fn tokyo_board() -> Color {
    active_tokens().board
}

pub(crate) fn tokyo_surface() -> Color {
    active_tokens().surface
}

pub(crate) fn tokyo_surface_2() -> Color {
    active_tokens().surface_2
}

pub(crate) fn tokyo_surface_3() -> Color {
    active_tokens().surface_3
}

pub(crate) fn tokyo_border() -> Color {
    active_tokens().border
}

pub(crate) fn tokyo_text() -> Color {
    active_tokens().text
}

pub(crate) fn tokyo_text_selection() -> Color {
    active_tokens().text_selection
}

pub(crate) fn tokyo_muted() -> Color {
    active_tokens().muted
}

pub(crate) fn tokyo_blue() -> Color {
    active_tokens().blue
}

pub(crate) fn tokyo_device_accent(fallback: Color) -> Color {
    if is_black_white(active_color_scheme()) {
        active_tokens().blue
    } else {
        fallback
    }
}

pub(crate) fn tokyo_inactive_lamp() -> Color {
    if is_black_white(active_color_scheme()) {
        active_tokens().muted
    } else {
        active_tokens().text
    }
}

pub(crate) fn tokyo_selection_blue() -> Color {
    active_tokens().selection_blue
}

pub(crate) fn tokyo_modal_backdrop() -> Color {
    let board = tokyo_board();
    Color {
        r: board.r * 0.58,
        g: board.g * 0.58,
        b: board.b * 0.58,
        a: 0.74,
    }
}

pub(crate) fn tokyo_subtle_line() -> Color {
    Color {
        a: 0.26,
        ..active_tokens().border
    }
}

pub(crate) fn tokyo_cyan() -> Color {
    active_tokens().cyan
}

pub(crate) fn tokyo_green() -> Color {
    active_tokens().green
}

pub(crate) fn tokyo_yellow() -> Color {
    active_tokens().yellow
}

pub(crate) fn tokyo_red() -> Color {
    active_tokens().red
}

pub(crate) fn tokyo_red_fill() -> Color {
    Color {
        a: 0.22,
        ..active_tokens().red
    }
}

pub(crate) fn tokyo_surface_3_tint() -> Color {
    Color {
        a: 0.45,
        ..active_tokens().surface_3
    }
}

pub(crate) fn tokyo_magenta() -> Color {
    active_tokens().magenta
}

pub(super) fn ui_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(UI_FONT).size(size).color(color)
}

pub(super) fn mono_text(content: impl Into<String>, size: u32, color: Color) -> Text<'static> {
    text(content.into()).font(MONO_FONT).size(size).color(color)
}

fn active_tokens() -> UiTokens {
    scheme_tokens(ColorScheme::from_index(
        ACTIVE_COLOR_SCHEME.load(Ordering::Relaxed),
    ))
}

fn active_color_scheme() -> ColorScheme {
    ColorScheme::from_index(ACTIVE_COLOR_SCHEME.load(Ordering::Relaxed))
}

fn is_black_white(scheme: ColorScheme) -> bool {
    matches!(
        scheme,
        ColorScheme::BlackWhiteDark | ColorScheme::BlackWhiteLight
    )
}

fn scheme_tokens(scheme: ColorScheme) -> UiTokens {
    match scheme {
        ColorScheme::TokyoNight => tokens([
            0x121320, 0x1D2030, 0x2F334D, 0x363B59, 0x414868, 0xC0CAF5, 0x565F89, 0x7AA2F7,
            0x7DCFFF, 0x9ECE6A, 0xE0AF68, 0xF7768E, 0xBB9AF7,
        ]),
        ColorScheme::TokyoNightLight => tokens([
            0xD5D6DB, 0xE8E9EF, 0xCACBD4, 0xB8BAC8, 0x9699A8, 0x343B58, 0x6C6F93, 0x2E7DE9,
            0x007197, 0x587539, 0x8F5E15, 0x8C4351, 0x7847BD,
        ]),
        ColorScheme::BlackWhiteDark => tokens([
            0x0B0B0C, 0x111112, 0x1A1A1C, 0x252527, 0x38383A, 0xD2D2D2, 0xA4A4A4, 0xC8C8C8,
            0xBEBEBE, 0xD8D8D8, 0xB0B0B0, 0xE2E2E2, 0x8F8F91,
        ]),
        ColorScheme::BlackWhiteLight => tokens([
            0xE1E1DF, 0xEEEEEC, 0xD4D4D1, 0xC6C6C2, 0xB9B9B5, 0x4B4B49, 0x747470, 0x5A5A57,
            0x6A6A66, 0x595955, 0x767670, 0x4A4A47, 0x696965,
        ]),
        ColorScheme::KanagawaWave => tokens([
            0x111116, 0x17171E, 0x222231, 0x303044, 0x45455E, 0xF2E8C9, 0xAAA297, 0x91B4E8,
            0x82C6D3, 0xA8CB75, 0xE8C780, 0xFF6E7D, 0xB69BDD,
        ]),
        ColorScheme::KanagawaLotus => tokens([
            0xE8DDA8, 0xD8C88F, 0xC6B17A, 0xAC9664, 0xB3A17A, 0x2A2826, 0x60594F, 0x2F4F84,
            0x436E68, 0x51743A, 0x965A16, 0x942A36, 0x593A78,
        ]),
        ColorScheme::CatppuccinMocha => tokens([
            0x0B0B12, 0x11111B, 0x1A1A2A, 0x26263A, 0x42465B, 0xF0F3FF, 0xAAB1CC, 0x8AADF4,
            0x91D7E3, 0xA6E3A1, 0xF9E2AF, 0xF38BA8, 0xCBA6F7,
        ]),
        ColorScheme::CatppuccinLatte => tokens([
            0xE4E2EA, 0xECE9F0, 0xD6D2DE, 0xC9C4D2, 0xBBB6C4, 0x57586F, 0x7A7B90, 0x42649D,
            0x427A82, 0x4E7047, 0x966628, 0xA94960, 0x7A559A,
        ]),
        ColorScheme::Nord => tokens([
            0x11151C, 0x181E28, 0x222C3A, 0x2D3A4C, 0x3D4A5D, 0xF2F6FC, 0xAAB5C8, 0x88B9EA,
            0x8AD8E8, 0xB4DA9A, 0xF0D38C, 0xD46F7A, 0xC79AC0,
        ]),
        ColorScheme::GruvboxDark => tokens([
            0x161819, 0x1D2021, 0x282828, 0x35302D, 0x50473F, 0xFBF1C7, 0xBDAE93, 0x83A598,
            0x8EC07C, 0xB8BB26, 0xFABD2F, 0xFB4934, 0xD3869B,
        ]),
        ColorScheme::GruvboxLight => tokens([
            0xFBF1C7, 0xEBDBB2, 0xD5C4A1, 0xBDAE93, 0xC5B68E, 0x282828, 0x7C6F64, 0x076678,
            0x427B58, 0x79740E, 0xB57614, 0x9D0006, 0x8F3F71,
        ]),
        ColorScheme::MaterialOcean => tokens([
            0x080B10, 0x0E131C, 0x162033, 0x1D2B42, 0x293950, 0xD2DBFF, 0x8190B5, 0x82AAFF,
            0x89DDFF, 0xC3E88D, 0xFFCB6B, 0xF07178, 0xC792EA,
        ]),
    }
}

fn tokens(colors: [u32; 13]) -> UiTokens {
    let [
        board,
        surface,
        surface_2,
        surface_3,
        border,
        text,
        muted,
        blue,
        cyan,
        green,
        yellow,
        red,
        magenta,
    ] = colors;

    UiTokens {
        board: rgb(board),
        surface: rgb(surface),
        surface_2: rgb(surface_2),
        surface_3: rgb(surface_3),
        border: rgb(border),
        text: rgb(text),
        text_selection: rgba(text, 0.28),
        muted: rgb(muted),
        blue: rgb(blue),
        selection_blue: rgba(blue, 0.18),
        cyan: rgb(cyan),
        green: rgb(green),
        yellow: rgb(yellow),
        red: rgb(red),
        magenta: rgb(magenta),
    }
}

fn black_white_dark() -> Palette {
    Palette {
        background: rgb(0x0B0B0C),
        text: rgb(0xD2D2D2),
        primary: rgb(0xC8C8C8),
        success: rgb(0xD8D8D8),
        warning: rgb(0xB0B0B0),
        danger: rgb(0xE2E2E2),
    }
}

fn black_white_light() -> Palette {
    Palette {
        background: rgb(0xE1E1DF),
        text: rgb(0x4B4B49),
        primary: rgb(0x5A5A57),
        success: rgb(0x595955),
        warning: rgb(0x767670),
        danger: rgb(0x4A4A47),
    }
}

fn catppuccin_latte() -> Palette {
    Palette {
        background: rgb(0xE4E2EA),
        text: rgb(0x57586F),
        primary: rgb(0x42649D),
        success: rgb(0x4E7047),
        warning: rgb(0x966628),
        danger: rgb(0xA94960),
    }
}

fn material_ocean() -> Palette {
    Palette {
        background: rgb(0x080B10),
        text: rgb(0xD2DBFF),
        primary: rgb(0x82AAFF),
        success: rgb(0xC3E88D),
        warning: rgb(0xFFCB6B),
        danger: rgb(0xF07178),
    }
}

fn rgb(hex: u32) -> Color {
    Color::from_rgb8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn rgba(hex: u32, alpha: f32) -> Color {
    Color {
        a: alpha,
        ..rgb(hex)
    }
}
