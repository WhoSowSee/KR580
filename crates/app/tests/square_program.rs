use k580_app::{AppCommand, Emulator};
use std::path::PathBuf;

#[test]
fn square_program_paints_8x8_outline_into_pixel_layer() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("square.580");

    let mut emulator = Emulator::default();
    let events = emulator.handle_command(AppCommand::LoadSnapshot(path));
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
