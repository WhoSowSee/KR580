use super::{Message, PROGRESS_DRIFT_STEP, StageState, T, UninstallStage, Uninstaller};
use std::path::PathBuf;

fn app() -> Uninstaller {
    Uninstaller::new(PathBuf::from("KR580")).0
}

#[test]
fn uninstall_progress_starts_empty() {
    let mut app = app();

    assert_eq!(app.display_progress, 0.0);
    assert_eq!(app.target_progress(), 0.0);

    app.started = true;
    assert_eq!(app.confirmed_progress(), UninstallStage::System.progress());
    assert_eq!(
        app.target_progress(),
        UninstallStage::System.animation_limit()
    );
}

#[test]
fn successful_completion_waits_for_the_progress_animation() {
    let mut app = app();
    app.stage = UninstallStage::Files;
    app.result = Some(Ok(()));
    app.display_progress = 0.98;

    assert_eq!(app.target_progress(), 1.0);
    assert!(app.progress_animating());
    assert!(!app.can_close());
    assert_eq!(app.stage_state(UninstallStage::Files), StageState::Active);
    assert_eq!(app.status().0, app.locale.t(T::RemovingFiles));

    app.display_progress = 1.0;
    assert!(!app.progress_animating());
    assert!(app.can_close());
    assert_eq!(app.stage_state(UninstallStage::Files), StageState::Complete);
    assert_eq!(app.status().0, app.locale.t(T::RemovalComplete));
}

#[test]
fn failed_uninstall_stops_animation_and_allows_close() {
    let mut app = app();
    app.result = Some(Err("failed".to_owned()));
    app.display_progress = 0.37;

    assert_eq!(app.target_progress(), 0.37);
    assert!(!app.progress_animating());
    assert!(app.can_close());
    assert_eq!(app.status().0, "failed");
    assert_eq!(app.status().1, super::style::RED);
}

#[test]
fn progress_tick_advances_to_the_current_stage_target() {
    let mut app = app();
    app.started = true;

    drop(app.update(Message::ProgressTick));
    assert!((app.display_progress - 0.02).abs() < f32::EPSILON);

    app.display_progress = 0.11;
    drop(app.update(Message::ProgressTick));
    assert_eq!(app.display_progress, UninstallStage::System.progress());
}

#[test]
fn active_links_stage_keeps_moving_beyond_confirmed_progress() {
    let mut app = app();
    app.started = true;
    app.stage = UninstallStage::Links;
    app.display_progress = UninstallStage::Links.progress();

    drop(app.update(Message::ProgressTick));

    assert!(app.display_progress > UninstallStage::Links.progress());

    app.display_progress = UninstallStage::Links.animation_limit() - PROGRESS_DRIFT_STEP / 2.0;
    drop(app.update(Message::ProgressTick));
    assert_eq!(
        app.display_progress,
        UninstallStage::Links.animation_limit()
    );
    assert!(app.display_progress < UninstallStage::Files.progress());
}

#[test]
fn stage_states_follow_real_operation_order() {
    let mut app = app();
    app.started = true;
    assert_eq!(app.stage_state(UninstallStage::System), StageState::Active);
    assert_eq!(app.stage_state(UninstallStage::Links), StageState::Pending);

    app.stage = UninstallStage::Links;
    assert_eq!(
        app.stage_state(UninstallStage::System),
        StageState::Complete
    );
    assert_eq!(app.stage_state(UninstallStage::Links), StageState::Active);
    assert_eq!(app.stage_state(UninstallStage::Files), StageState::Pending);

    app.result = Some(Err("failed".to_owned()));
    assert_eq!(app.stage_state(UninstallStage::Links), StageState::Failed);
}
