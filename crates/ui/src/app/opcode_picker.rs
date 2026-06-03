use k580_core::decode_opcode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OpcodeChoice {
    pub(crate) value: u8,
    pub(crate) mnemonic: String,
}

impl OpcodeChoice {
    fn new(value: u8) -> Option<Self> {
        let mnemonic = decode_opcode(value).ok()?.mnemonic;
        Some(Self { value, mnemonic })
    }
}

pub(crate) fn filtered_opcode_choices(search: &str) -> Vec<OpcodeChoice> {
    let search = search.trim().to_ascii_uppercase();

    (0..=u8::MAX)
        .filter_map(OpcodeChoice::new)
        .filter(|choice| {
            search.is_empty()
                || format!("{:02X} {}", choice.value, choice.mnemonic)
                    .to_ascii_uppercase()
                    .contains(&search)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::filtered_opcode_choices;

    #[test]
    fn opcode_choices_filter_by_hex_or_mnemonic() {
        assert_eq!(filtered_opcode_choices("3E")[0].value, 0x3E);
        assert!(
            filtered_opcode_choices("MVI A")
                .iter()
                .any(|choice| choice.value == 0x3E)
        );
    }
}
