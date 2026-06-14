pub(super) fn storage_buffer_text(buffer: &[u8]) -> String {
    k580_app::decode_oem_text(buffer)
}

#[cfg(test)]
mod tests {
    use super::storage_buffer_text;

    #[test]
    fn storage_buffer_text_preserves_terminal_text_and_decodes_cp866() {
        assert_eq!(
            storage_buffer_text(&[b'A', b'\r', b'\n', b'B', b'\t', 0x01, 0x80]),
            "A\nB\t·А"
        );
    }
}
