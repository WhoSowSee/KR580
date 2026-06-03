pub(super) fn storage_buffer_text(buffer: &[u8]) -> String {
    let mut out = String::new();
    let mut previous_cr = false;
    for &byte in buffer {
        let ch = match byte {
            b'\r' => {
                out.push('\n');
                previous_cr = true;
                continue;
            }
            b'\n' => {
                if previous_cr {
                    previous_cr = false;
                    continue;
                }
                '\n'
            }
            b'\t' => '\t',
            0x20..=0x7E => byte as char,
            0x80..=0xFF => cp866(byte),
            _ => '·',
        };
        out.push(ch);
        previous_cr = false;
    }
    out
}

fn cp866(byte: u8) -> char {
    const TABLE: [char; 128] = [
        'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П', 'Р', 'С',
        'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я', 'а', 'б', 'в', 'г',
        'д', 'е', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', '░', '▒', '▓', '│', '┤', '╡',
        '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
        '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧', '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘',
        '┌', '█', '▄', '▌', '▐', '▀', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы',
        'ь', 'э', 'ю', 'я', 'Ё', 'ё', 'Є', 'є', 'Ї', 'ї', 'Ў', 'ў', '°', '∙', '·', '√', '№', '¤',
        '■', '\u{00A0}',
    ];
    TABLE[(byte - 0x80) as usize]
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
