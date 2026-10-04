use super::{Emulator, IoCompletion, IoUpdate};
use crate::backend::{AppCommand, AppEvent, CommandResult, RequestId};
use crate::persistence::ExportModel;
use crate::persistence::import::CpuPatch;

#[test]
fn superseded_storage_open_does_not_restore_old_attachment_or_error() {
    for kind in [
        crate::devices::StorageKind::Floppy,
        crate::devices::StorageKind::Hdd,
    ] {
        let mut emulator = Emulator::default();
        emulator.storage_generation[super::storage_slot(kind)] = 2;
        let events = emulator.finish_io(IoCompletion {
            id: RequestId(1),
            generation: super::IoGeneration::Storage(kind, 1),
            result: Ok((
                CommandResult::Completed,
                Some(IoUpdate::Storage {
                    kind,
                    path: "old.kpd".into(),
                    file: Err(crate::devices::DeviceError::NotReady),
                }),
            )),
        });
        assert_eq!(
            events,
            vec![AppEvent::CommandFinished {
                id: RequestId(1),
                result: Ok(CommandResult::Superseded)
            }]
        );
        assert!(emulator.bus().floppy.state().path.is_none());
        assert!(emulator.bus().hdd.state().path.is_none());
    }
}

#[test]
fn partial_load_completion_keeps_memory_edits_made_after_parsing() {
    let patch = CpuPatch::owned(ExportModel {
        registers: vec![("A".into(), "41".into())],
        flags: vec![],
        memory: vec![],
    })
    .unwrap();
    for (result, update, expected_a, expected_memory) in [
        (CommandResult::Imported, IoUpdate::Import(patch), 0x41, 0),
        (
            CommandResult::LoadedSubprogram { end: 0x2000 },
            IoUpdate::Subprogram {
                start: 0x2000,
                values: vec![0x41],
            },
            0,
            0x41,
        ),
    ] {
        let mut emulator = Emulator::default();
        emulator.handle_command(AppCommand::SetMemory(0x1234, 0xAA));
        emulator.finish_io(IoCompletion {
            id: RequestId(1),
            generation: super::IoGeneration::Document(emulator.document_generation),
            result: Ok((result, Some(update))),
        });
        assert_eq!(emulator.cpu().memory.read(0x1234), 0xAA);
        assert_eq!(emulator.cpu().registers.a, expected_a);
        assert_eq!(emulator.cpu().memory.read(0x2000), expected_memory);
    }
}

#[test]
fn replacement_document_rejects_old_completion_and_error() {
    let mut emulator = Emulator::default();
    let generation = emulator.document_generation;
    let mut replacement = k580_core::Cpu8080State::default();
    replacement.pc = 0x1234;
    emulator.handle_command(AppCommand::ApplyCpuState(Box::new(replacement.clone())));
    for result in [
        Ok((
            CommandResult::LoadedProgram,
            Some(IoUpdate::Program(Box::default())),
        )),
        Err(crate::backend::AppError::Io("old error".into())),
    ] {
        let events = emulator.finish_io(IoCompletion {
            id: RequestId(1),
            generation: super::IoGeneration::Document(generation),
            result,
        });
        assert_eq!(emulator.cpu(), &replacement);
        assert_eq!(
            events,
            vec![AppEvent::CommandFinished {
                id: RequestId(1),
                result: Ok(CommandResult::Superseded)
            }]
        );
    }
}
