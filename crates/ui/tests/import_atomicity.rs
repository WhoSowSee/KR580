use k580_core::Cpu8080State;
use k580_ui::persistence::ExportModel;

#[test]
fn rejected_import_leaves_the_entire_cpu_unchanged() {
    let mut original = Cpu8080State::default();
    original.registers.a = 0x12;
    original.pc = 0x3456;
    original.memory.write(0x0100, 0x78);
    for (registers, flags) in [
        (vec![("A", "41"), ("INVALID", "00")], vec![]),
        (vec![("A", "41"), ("SP", "invalid")], vec![]),
        (vec![("A", "41")], vec![("S", true), ("INVALID", true)]),
    ] {
        let model = ExportModel {
            registers: registers
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect(),
            flags: flags
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
            memory: vec![(0x0100, 0xAA)],
        };
        let mut state = original.clone();
        assert!(model.apply_to(&mut state).is_err());
        assert_eq!(state, original);
    }
}
