use k580_app::{AppCommand, Emulator};
use k580_core::Cpu8080State;
use k580_persistence::Snapshot580Serializer;
use std::{fs, path::PathBuf};

#[test]
fn square_program_paints_8x8_outline_into_pixel_layer() {
    let mut emulator = Emulator::default();
    let path = write_square_snapshot();
    let events = emulator.handle_command(AppCommand::LoadSnapshot(path));
    let _ = fs::remove_file(square_snapshot_path());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, k580_app::AppEvent::ErrorRaised(_))),
        "LoadSnapshot raised an error: {events:?}"
    );

    for _ in 0..50_000 {
        emulator.handle_command(AppCommand::StepInstruction);
        if emulator.cpu().halted {
            break;
        }
    }
    assert!(emulator.cpu().halted, "program did not reach HLT");

    let monitor = emulator.bus().monitor.state();

    // 8×8 outline: top + bottom + 2 sides without corners = 8 + 8 + 6 + 6.
    assert_eq!(
        monitor.pixels.len(),
        28,
        "expected 28 outline pixels, got {} ({:?})",
        monitor.pixels.len(),
        &monitor.pixels[..monitor.pixels.len().min(8)]
    );

    for &(x, y, intensity) in &monitor.pixels {
        assert!(x < 8 && y < 8, "outline pixel outside 8×8: ({x},{y})");
        let on_edge = y == 0 || y == 7 || x == 0 || x == 7;
        assert!(on_edge, "non-edge pixel was painted: ({x},{y})");
        assert_eq!(
            intensity, 0x7F,
            "pixel ({x},{y}) not max-bright: {intensity:02X}"
        );
    }

    // Interior must be untouched: only edge cells exist in the pixel list.
    for y in 1..7u8 {
        for x in 1..7u8 {
            let lit = monitor.pixels.iter().any(|&(px, py, _)| px == x && py == y);
            assert!(!lit, "interior ({x},{y}) was lit");
        }
    }
}

fn write_square_snapshot() -> PathBuf {
    let mut state = Cpu8080State::default();
    let mut offset = 0usize;

    for x in 0..8 {
        write_pixel(&mut state, &mut offset, x, 0);
    }
    for x in 0..8 {
        write_pixel(&mut state, &mut offset, x, 7);
    }
    for y in 1..7 {
        write_pixel(&mut state, &mut offset, 0, y);
        write_pixel(&mut state, &mut offset, 7, y);
    }
    state.memory.write(offset as u16, 0x76);

    let path = square_snapshot_path();
    fs::write(&path, Snapshot580Serializer::to_bytes(&state)).expect("write square snapshot");
    path
}

fn write_pixel(state: &mut Cpu8080State, offset: &mut usize, x: u8, y: u8) {
    for byte in [0xFF, x, y] {
        write_monitor_byte(state, offset, byte);
    }
}

fn write_monitor_byte(state: &mut Cpu8080State, offset: &mut usize, byte: u8) {
    for opcode in [0x3E, byte, 0xD3, 0x00] {
        state.memory.write(*offset as u16, opcode);
        *offset += 1;
    }
}

fn square_snapshot_path() -> PathBuf {
    std::env::temp_dir().join(format!("k580-square-{}.580", std::process::id()))
}
