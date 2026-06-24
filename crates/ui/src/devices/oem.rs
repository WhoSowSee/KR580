pub fn decode_oem_text(buffer: &[u8]) -> String {
    let mut output = String::new();
    let mut previous_cr = false;
    for &byte in buffer {
        let character = match byte {
            b'\r' => {
                output.push('\n');
                previous_cr = true;
                continue;
            }
            b'\n' if previous_cr => {
                previous_cr = false;
                continue;
            }
            b'\n' => '\n',
            b'\t' => '\t',
            0x20..=0x7E => byte as char,
            0x80..=0xFF => cp866(byte),
            _ => '·',
        };
        output.push(character);
        previous_cr = false;
    }
    output
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
