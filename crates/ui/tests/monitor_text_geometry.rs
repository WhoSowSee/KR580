use k580_core::Cpu8080State;
use k580_ui::backend::{AppCommand, Emulator, TEXT_COLS};
use k580_ui::devices::IoBus;

const TASK1_LETTERS: [u8; 17] = [
    0x06, 0x61, 0x0E, 0x1A, 0x3E, 0x50, 0xD3, 0x00, 0x78, 0xD3, 0x00, 0x04, 0x0D, 0xC2, 0x04, 0x00,
    0x76,
];

#[test]
fn task1_letters_uses_64_column_framebuffer_rows() {
    assert_eq!(TEXT_COLS, 64);

    let mut cpu = Cpu8080State::default();
    cpu.memory.as_mut_slice()[..TASK1_LETTERS.len()].copy_from_slice(&TASK1_LETTERS);
    let mut emulator = Emulator::new(cpu, IoBus::default());

    for run in 0..4 {
        if run != 0 {
            emulator.handle_command(AppCommand::SetPc(0));
            emulator.handle_command(AppCommand::ClearHalt);
        }
        for _ in 0..500 {
            emulator.handle_command(AppCommand::StepInstruction);
            if emulator.cpu().halted {
                break;
            }
        }
        assert!(emulator.cpu().halted);
    }

    let cells = &emulator.bus().monitor.state().text_cells;
    let alphabet: Vec<u8> = (b'a'..=b'z').collect();
    let output = alphabet.repeat(4);
    let first_row: Vec<u8> = cells[..TEXT_COLS as usize]
        .iter()
        .map(|cell| cell.ch)
        .collect();
    let second_row: Vec<u8> = cells[TEXT_COLS as usize..output.len()]
        .iter()
        .map(|cell| cell.ch)
        .collect();

    assert_eq!(first_row, output[..TEXT_COLS as usize]);
    assert_eq!(second_row, output[TEXT_COLS as usize..]);
}
