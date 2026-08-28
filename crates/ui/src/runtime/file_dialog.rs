use iced::{Task, window};
use std::path::PathBuf;

pub(crate) fn run<T>(
    parent: Option<window::Id>,
    dialog: rfd::FileDialog,
    action: impl FnOnce(rfd::FileDialog) -> Option<PathBuf> + Send + 'static,
    map: impl Fn(PathBuf) -> T + Send + 'static,
) -> Task<T>
where
    T: Send + 'static,
{
    let Some(parent) = parent else {
        return Task::none();
    };
    window::run(parent, move |window| action(dialog.set_parent(window)))
        .then(move |path| path.map(&map).map_or_else(Task::none, Task::done))
}
