pub(super) fn wide_string(bytes: &[u8], offset: usize) -> Result<String, String> {
    if !offset.is_multiple_of(2) || offset >= bytes.len() {
        return Err("printer string offset is outside its buffer".to_owned());
    }
    let mut units = Vec::new();
    for pair in bytes[offset..].as_chunks::<2>().0 {
        let unit = u16::from_le_bytes([pair[0], pair[1]]);
        if unit == 0 {
            return String::from_utf16(&units).map_err(|error| error.to_string());
        }
        units.push(unit);
    }
    Err("printer string is not terminated inside its buffer".to_owned())
}

#[cfg(test)]
mod tests {
    use super::wide_string;

    #[test]
    fn native_strings_cannot_escape_their_owned_byte_buffer() {
        assert_eq!(wide_string(&[0, 0, 65, 0, 0, 0], 2).unwrap(), "A");
        for (bytes, offset) in [
            (&[65, 0][..], 0),
            (&[65, 0, 0][..], 0),
            (&[0, 0][..], 2),
            (&[0, 0][..], 1),
            (&[0, 0xD8, 0, 0][..], 0),
        ] {
            assert!(wide_string(bytes, offset).is_err());
        }
    }
}
