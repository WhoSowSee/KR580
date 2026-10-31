use super::{UpdatePhase, UpdateState};
use crate::runtime::updates::{PreparedUpdate, Release};
use std::time::{Duration, Instant};

fn release() -> Release {
    Release {
        version: "5.1.0".into(),
        page: "https://github.com/WhoSowSee/KR580/releases/tag/v5.1.0".into(),
        asset: None,
    }
}

#[test]
fn no_update_result_expires_or_resets_on_menu_close() {
    let now = Instant::now();
    let mut state = UpdateState::default();
    let generation = state.check(now);
    state.checked(generation, Ok(None), true, now);
    state.tick(now + Duration::from_millis(2999));
    assert!(matches!(state.phase, UpdatePhase::Current(_)));
    state.tick(now + Duration::from_secs(3));
    assert!(matches!(state.phase, UpdatePhase::Idle));
    let generation = state.check(now);
    state.checked(generation, Ok(None), true, now);
    state.menu_closed();
    assert!(matches!(state.phase, UpdatePhase::Idle));
    let generation = state.check(now);
    state.checked(generation, Ok(None), false, now);
    assert!(matches!(state.phase, UpdatePhase::Idle));
}

#[test]
fn closing_menu_preserves_check_release_download_and_prepared_update() {
    let now = Instant::now();
    let mut state = UpdateState::default();
    let generation = state.check(now);
    state.menu_closed();
    assert!(matches!(state.phase, UpdatePhase::Checking(_)));
    state.checked(generation, Ok(Some(release())), false, now);
    state.menu_closed();
    assert!(matches!(state.phase, UpdatePhase::Available(_)));
    let (generation, _) = state.download(release(), now);
    state.menu_closed();
    state.tick(now + Duration::from_secs(5));
    assert!(matches!(state.phase, UpdatePhase::Downloading { .. }));
    state.downloaded(
        generation,
        Ok(PreparedUpdate {
            version: "5.1.0".into(),
            restart_flag: "restart.json".into(),
        }),
    );
    state.menu_closed();
    assert!(matches!(state.phase, UpdatePhase::Ready(_)));
}

#[test]
fn stale_completions_and_network_errors_do_not_report_up_to_date() {
    let now = Instant::now();
    let mut state = UpdateState::default();
    let old = state.check(now);
    let current = state.check(now);
    state.checked(old, Ok(None), true, now);
    assert!(matches!(state.phase, UpdatePhase::Checking(_)));
    state.checked(current, Err("offline".into()), true, now);
    assert!(matches!(
        state.phase,
        UpdatePhase::Failed { release: None, .. }
    ));
    let (generation, _) = state.download(release(), now);
    state.downloaded(generation, Err("incomplete download".into()));
    assert!(matches!(
        state.phase,
        UpdatePhase::Failed {
            release: Some(_),
            ..
        }
    ));
}

#[test]
fn cancelling_dirty_restart_preserves_prepared_update_without_requesting_relaunch() {
    use crate::app::{DesktopApp, MenuId, Message, PendingAction};
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("restart.json");
    let (mut app, _) = DesktopApp::with_initial_path(None);
    app.shell.updates.phase = UpdatePhase::Ready(PreparedUpdate {
        version: "5.1.0".into(),
        restart_flag: marker.clone(),
    });
    app.document.dirty = true;
    let _ = app.update(Message::MenuToggled(MenuId::Help));
    let _ = app.update(Message::UpdateActivated);
    assert!(matches!(
        app.document.pending_action,
        Some(PendingAction::RestartForUpdate)
    ));
    assert!(!marker.exists());
    let _ = app.update(Message::CancelDiscard);
    assert!(app.document.pending_action.is_none());
    assert!(matches!(app.shell.updates.phase, UpdatePhase::Ready(_)));
    assert!(!marker.exists());
}
