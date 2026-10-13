use iced::Color;

use super::{MemoryRowVisuals, value_color};
use crate::persistence::ColorScheme;
use crate::view::theme::{DARK_COLOR_SCHEMES, LIGHT_COLOR_SCHEMES, scheme_tokens};

const ROW_STATES: [(bool, bool); 4] = [(false, false), (true, false), (false, true), (true, true)];

#[test]
fn monochrome_shades_preserve_role_order_and_separation() {
    for scheme in [ColorScheme::BlackWhiteDark, ColorScheme::BlackWhiteLight] {
        let direction = if scheme == ColorScheme::BlackWhiteDark {
            1.0
        } else {
            -1.0
        };
        for (selected, halted) in ROW_STATES {
            let shades = row_colors(scheme, true, selected, halted);
            for shade in shades {
                assert_eq!(shade.r, shade.g);
                assert_eq!(shade.g, shade.b);
                assert_eq!(shade.a, 1.0);
                assert!(shade.relative_contrast(scheme_tokens(scheme).board) >= 4.5);
            }
            for gap in role_gaps(shades) {
                assert!(
                    gap * direction >= 0.08,
                    "operand shades overlap for {scheme:?}"
                );
            }
        }
    }
    let scheme = ColorScheme::BlackWhiteDark;
    let ordinary = row_colors(scheme, true, false, false);
    let selected = row_colors(scheme, true, true, false);
    assert_eq!(selected, row_colors(scheme, true, false, true));
    assert_eq!(selected, row_colors(scheme, true, true, true));
    for (ordinary, selected) in role_gaps(ordinary).into_iter().zip(role_gaps(selected)) {
        assert!(ordinary > selected);
    }
    for (ordinary, selected) in ordinary.into_iter().zip(selected) {
        assert!(selected.r >= ordinary.r);
    }
}

#[test]
fn colored_themes_and_disabled_highlighting_keep_their_palette() {
    for scheme in DARK_COLOR_SCHEMES.into_iter().chain(LIGHT_COLOR_SCHEMES) {
        let tokens = scheme_tokens(scheme);
        for (selected, halted) in ROW_STATES {
            assert_eq!(
                row_colors(scheme, false, selected, halted),
                [tokens.green; 4]
            );
            if !matches!(
                scheme,
                ColorScheme::BlackWhiteDark | ColorScheme::BlackWhiteLight
            ) {
                assert_eq!(
                    row_colors(scheme, true, selected, halted),
                    [tokens.green, tokens.yellow, tokens.cyan, tokens.magenta],
                );
            }
        }
    }
}

fn row_colors(scheme: ColorScheme, enabled: bool, selected: bool, halted: bool) -> [Color; 4] {
    std::array::from_fn(|role| {
        value_color(
            &MemoryRowVisuals {
                selected,
                next_selected: false,
                halted_here: halted,
                operand_highlighting: enabled,
                is_address_operand: role == 1,
                is_data_operand: role == 2,
                is_port_operand: role == 3,
            },
            scheme,
        )
    })
}

fn role_gaps(shades: [Color; 4]) -> [f32; 3] {
    let [opcode, address, data, port] = shades;
    [data.r - opcode.r, port.r - data.r, address.r - port.r]
}
