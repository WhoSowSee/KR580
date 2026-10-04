use super::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[test]
fn partial_role_failure_restores_distinct_defaults_and_leaves_unassigned_roles_alone() {
    let values = RefCell::new(HashMap::from([
        (
            (SNAPSHOT_UTI.to_owned(), HandlerRole::Viewer as u32),
            "viewer.app".to_owned(),
        ),
        (
            (SNAPSHOT_UTI.to_owned(), HandlerRole::Editor as u32),
            "editor.app".to_owned(),
        ),
    ]));
    let query = |kind: &str, role: HandlerRole| {
        values
            .borrow()
            .get(&(kind.to_owned(), role as u32))
            .cloned()
    };
    let mut defaults = DefaultHandlers::capture_with(query);
    let result = defaults.apply_with(query, |kind, role, handler| {
        if role as u32 == HandlerRole::Editor as u32 {
            return Err("injected failure".into());
        }
        values
            .borrow_mut()
            .insert((kind.to_owned(), role as u32), handler.to_owned());
        Ok(())
    });
    assert!(result.is_err());
    defaults
        .rollback_with(query, |kind, role, handler| {
            values
                .borrow_mut()
                .insert((kind.to_owned(), role as u32), handler.to_owned());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        query(SNAPSHOT_UTI, HandlerRole::Viewer).as_deref(),
        Some("viewer.app")
    );
    assert_eq!(
        query(SNAPSHOT_UTI, HandlerRole::Editor).as_deref(),
        Some("editor.app")
    );
    assert!(query(SNAPSHOT_UTI, HandlerRole::Shell).is_none());
    assert!(query(SUBPROGRAM_UTI, HandlerRole::Viewer).is_none());
}
