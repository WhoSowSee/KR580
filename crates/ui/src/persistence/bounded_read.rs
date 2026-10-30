use std::io::{self, Read};

pub(super) fn read(reader: impl Read, max_len: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(max_len as u64 + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::read;
    use std::io::{self, Read};

    #[test]
    fn reads_at_most_one_byte_past_the_payload_limit() {
        for limit in [0, 1, 65_536, 65_549] {
            let mut source = io::repeat(0xA5).take((limit + 9) as u64);
            let bytes = read(&mut source, limit).unwrap();
            assert_eq!(bytes, vec![0xA5; limit + 1], "limit={limit}");
            assert_eq!(source.limit(), 8, "limit={limit}");
        }
    }
}
