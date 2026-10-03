use k580_ui::backend::AppError;
use k580_ui::backend::error::AppErrorKind;
use k580_ui::persistence::ProgramError;
use std::error::Error;

#[test]
fn cloning_a_backend_error_preserves_the_original_typed_io_cause() {
    let error = AppError::from(ProgramError::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "arbitrary vendor wording",
    )));
    let cloned = error.clone();
    assert_eq!(cloned.kind(), AppErrorKind::PermissionDenied);
    let cause = cloned.source().unwrap().source().unwrap();
    assert!(cause.downcast_ref::<ProgramError>().is_some());
    assert_eq!(
        cause
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
}
