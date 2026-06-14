//! Free helper functions used by the runtime: hex parsing, normalization,
//! and a couple of small calculators that don't need access to
//! `DesktopApp` state.

use crate::app::{MEMORY_SCROLL_ID, Message, REGISTER_ORDER};
use iced::Task;
use iced::widget::operation;
use k580_core::RegisterName;

pub(super) fn scroll_memory_to(offset: f32) -> Task<Message> {
    operation::scroll_to(
        MEMORY_SCROLL_ID,
        operation::AbsoluteOffset {
            x: None,
            y: Some(offset),
        },
    )
}

pub(super) fn parse_hex_u8(input: &str) -> Result<u8, String> {
    u8::from_str_radix(hex_digits(input), 16).map_err(|_| format!("Invalid byte hex: {input}"))
}

pub(crate) fn parse_hex_u16(input: &str) -> Result<u16, String> {
    u16::from_str_radix(hex_digits(input), 16).map_err(|_| format!("Invalid address hex: {input}"))
}

fn hex_digits(input: &str) -> &str {
    input
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
}

pub(super) fn bounded_hex_input(input: &str, max_len: usize) -> Option<String> {
    let input = hex_digits(input);
    if input.len() > max_len || !input.chars().all(|char| char.is_ascii_hexdigit()) {
        return None;
    }

    Some(input.to_ascii_uppercase())
}

pub(super) fn parse_hex_byte_sequence(input: &str) -> Result<Option<Vec<u8>>, ()> {
    let tokens = input.split_whitespace().collect::<Vec<_>>();
    if tokens.len() <= 1 && input.trim().len() <= 2 {
        return if input.trim().chars().all(|char| char.is_ascii_hexdigit()) {
            Ok(None)
        } else {
            Err(())
        };
    }

    let mut values = Vec::with_capacity(tokens.len());
    for token in tokens {
        if token.len() != 2 || !token.chars().all(|char| char.is_ascii_hexdigit()) {
            return Err(());
        }
        values.push(u8::from_str_radix(token, 16).map_err(|_| ())?);
    }
    Ok(Some(values))
}

pub(super) fn parse_hex_byte_sequence_edit(
    input: &str,
    existing: &str,
) -> Result<Option<Vec<u8>>, ()> {
    match parse_hex_byte_sequence(input) {
        Err(()) => {}
        result => return result,
    }
    if existing.len() != 2 || !existing.is_ascii() || !input.is_ascii() {
        return Err(());
    }
    for split in 0..=existing.len() {
        let (prefix, suffix) = existing.split_at(split);
        if let Some(candidate) = input
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(suffix))
            && let Ok(Some(values)) = parse_hex_byte_sequence(candidate)
        {
            return Ok(Some(values));
        }
    }
    Err(())
}

const VALID_REGISTER_CHARS: [char; 7] = ['A', 'B', 'C', 'D', 'E', 'H', 'L'];

pub(super) fn bounded_register_input(input: &str) -> Option<String> {
    let input = input.trim();
    if input.len() > 1 {
        return None;
    }

    let upper = input.to_ascii_uppercase();
    if upper.is_empty() || upper.chars().all(|c| VALID_REGISTER_CHARS.contains(&c)) {
        Some(upper)
    } else {
        None
    }
}

pub(super) fn register_index(register: RegisterName) -> usize {
    REGISTER_ORDER
        .iter()
        .position(|candidate| *candidate == register)
        .unwrap_or(0)
}

/// Adds `delta` to a byte and clamps the result into `0x00..=0xFF`. Used
/// by the ArrowUp/ArrowDown handlers on byte-typed inputs so that
/// stepping past either end of the range becomes a no-op instead of
/// wrapping around (which would silently change a `00` into `FF` on a
/// single keystroke). `delta` is `i32` for ergonomic call sites; only
/// values in `i16`'s range can ever change the result, which is well
/// within what we ever pass.
pub(super) fn saturating_step_u8(value: u8, delta: i32) -> u8 {
    (value as i32 + delta).clamp(0, u8::MAX as i32) as u8
}
