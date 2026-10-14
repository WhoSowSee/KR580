use iced::Task;
use std::time::Instant;

use super::updates::UpdatePhase;
use super::{DesktopApp, MenuId, Message, PendingAction};

impl DesktopApp {
    pub(super) fn update_release_message(&mut self, message: &Message) -> Option<Task<Message>> {
        match message {
            Message::UpdateChecked(generation, result) => {
                self.shell.updates.checked(
                    *generation,
                    result.clone(),
                    self.shell.open_menu == Some(MenuId::Help),
                    Instant::now(),
                );
            }
            Message::UpdateDownloaded(generation, result) => {
                self.shell.updates.downloaded(*generation, result.clone());
            }
            Message::UpdateRestartReady(result) => match result {
                Ok(()) => return Some(Task::done(Message::WindowClose)),
                Err(error) => {
                    self.shell.updates.phase = UpdatePhase::Failed {
                        error: error.clone(),
                        release: None,
                    }
                }
            },
            Message::UpdateAnimationTick => {}
            Message::UpdateActivated => return Some(self.activate_update()),
            _ => return None,
        }
        Some(Task::none())
    }

    fn activate_update(&mut self) -> Task<Message> {
        let release = match &self.shell.updates.phase {
            UpdatePhase::Available(release)
            | UpdatePhase::Failed {
                release: Some(release),
                ..
            } => Some(release.clone()),
            UpdatePhase::Ready(_) => {
                if self.document.dirty {
                    self.open_discard_modal(PendingAction::RestartForUpdate);
                    return Task::none();
                }
                return self.restart_for_update();
            }
            UpdatePhase::Checking(_)
            | UpdatePhase::Downloading { .. }
            | UpdatePhase::Current(_) => return Task::none(),
            _ => None,
        };
        if let Some(release) = release {
            if release.asset.is_none() {
                let url = release.page;
                return Task::future(async move {
                    if let Err(error) = super::update_overlays::open_external_url(&url) {
                        tracing::warn!(%error, "open release page failed");
                    }
                })
                .discard();
            }
            let (generation, progress) =
                self.shell.updates.download(release.clone(), Instant::now());
            return Task::perform(
                crate::runtime::updates::download(release, progress),
                move |result| Message::UpdateDownloaded(generation, result),
            );
        }
        let generation = self.shell.updates.check(Instant::now());
        Task::perform(crate::runtime::updates::check(), move |result| {
            Message::UpdateChecked(generation, result)
        })
    }

    pub(crate) fn restart_for_update(&mut self) -> Task<Message> {
        let UpdatePhase::Ready(prepared) = &self.shell.updates.phase else {
            return Task::none();
        };
        let marker = prepared.restart_flag.clone();
        let paths: Vec<_> = self
            .document
            .current_snapshot_path
            .clone()
            .into_iter()
            .collect();
        Task::perform(
            async move {
                let bytes = serde_json::to_vec(&paths).map_err(|error| error.to_string())?;
                tokio::fs::write(marker, bytes)
                    .await
                    .map_err(|error| error.to_string())
            },
            Message::UpdateRestartReady,
        )
    }
}
