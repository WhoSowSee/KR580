use crate::devices::printer::PrinterError;
pub(super) fn wide_string(bytes: &[u8], offset: usize) -> Result<String, PrinterError> {
    if !offset.is_multiple_of(2) || offset >= bytes.len() {
        return Err(PrinterError::invalid(
            "printer string offset is outside its buffer",
        ));
    }
    let mut units = Vec::new();
    for pair in bytes[offset..].as_chunks::<2>().0 {
        let unit = u16::from_le_bytes([pair[0], pair[1]]);
        if unit == 0 {
            return String::from_utf16(&units)
                .map_err(|error| PrinterError::invalid_source(error.to_string(), error));
        }
        units.push(unit);
    }
    Err(PrinterError::invalid(
        "printer string is not terminated inside its buffer",
    ))
}

#[cfg(test)]
mod tests {
    use super::wide_string;
    use crate::devices::printer::PrinterErrorKind;
    use std::error::Error;

    #[test]
    fn invalid_utf16_preserves_its_encoding_error() {
        let error = wide_string(&[0, 0xD8, 0, 0], 0).unwrap_err();
        assert_eq!(error.kind(), PrinterErrorKind::InvalidData);
        let source = error.source().unwrap();
        assert!(
            source
                .downcast_ref::<std::string::FromUtf16Error>()
                .is_some()
        );
        assert_eq!(error.to_string(), source.to_string());
    }

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
