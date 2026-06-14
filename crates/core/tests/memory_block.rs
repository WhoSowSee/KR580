use k580_core::Cpu8080State;

#[test]
fn memory_block_writes_consecutive_bytes() {
    let mut cpu = Cpu8080State::default();

    cpu.set_memory_block(0x0100, &[0x3E, 0x41, 0xD3, 0x03, 0x76])
        .unwrap();

    assert_eq!(
        &cpu.memory.as_slice()[0x0100..0x0105],
        &[0x3E, 0x41, 0xD3, 0x03, 0x76]
    );
}

#[test]
fn overflowing_memory_block_is_rejected_atomically() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0xFFFE, 0xAA);
    cpu.memory.write(0xFFFF, 0xBB);

    assert!(cpu.set_memory_block(0xFFFE, &[0x3E, 0x41, 0x76]).is_err());
    assert_eq!(&cpu.memory.as_slice()[0xFFFE..], &[0xAA, 0xBB]);
}
