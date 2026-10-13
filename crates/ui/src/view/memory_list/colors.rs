use iced::Color;

use super::MemoryRowVisuals;
use crate::persistence::ColorScheme;
use crate::view::theme::scheme_tokens;

#[cfg(test)]
mod tests;

pub(super) fn value_color(visuals: &MemoryRowVisuals, scheme: ColorScheme) -> Color {
    let tokens = scheme_tokens(scheme);
    if !visuals.operand_highlighting {
        return tokens.green;
    }
    let [opcode, address, data, port] = match scheme {
        ColorScheme::BlackWhiteDark if visuals.selected || visuals.halted_here => {
            [gray(182), gray(255), gray(206), gray(230)]
        }
        ColorScheme::BlackWhiteDark => [gray(136), gray(255), gray(176), gray(216)],
        ColorScheme::BlackWhiteLight => [gray(72), gray(0), gray(48), gray(24)],
        _ => [tokens.green, tokens.yellow, tokens.cyan, tokens.magenta],
    };
    if visuals.is_port_operand {
        port
    } else if visuals.is_address_operand {
        address
    } else if visuals.is_data_operand {
        data
    } else {
        opcode
    }
}

fn gray(value: u8) -> Color {
    Color::from_rgb8(value, value, value)
}
