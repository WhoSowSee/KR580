use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::i18n::Key;
use crate::runtime::updates::{PreparedUpdate, Release, TransferProgress};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(crate) struct UpdateState {
    pub(crate) phase: UpdatePhase,
    generation: u64,
}

#[derive(Default, Debug)]
pub(crate) enum UpdatePhase {
    #[default]
    Idle,
    Checking(Instant),
    Available(Release),
    Downloading {
        release: Release,
        progress: Arc<TransferProgress>,
        started: Instant,
    },
    Ready(PreparedUpdate),
    Current(Instant),
    Failed {
        error: String,
        release: Option<Release>,
    },
}

impl UpdateState {
    pub(crate) fn check(&mut self, now: Instant) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.phase = UpdatePhase::Checking(now);
        self.generation
    }

    pub(crate) fn checked(
        &mut self,
        generation: u64,
        result: Result<Option<Release>, String>,
        visible: bool,
        now: Instant,
    ) {
        if generation != self.generation || !matches!(self.phase, UpdatePhase::Checking(_)) {
            return;
        }
        self.phase = match result {
            Ok(Some(release)) => UpdatePhase::Available(release),
            Ok(None) if visible => UpdatePhase::Current(now + Duration::from_secs(3)),
            Ok(None) => UpdatePhase::Idle,
            Err(error) => UpdatePhase::Failed {
                error,
                release: None,
            },
        };
    }

    pub(crate) fn download(
        &mut self,
        release: Release,
        now: Instant,
    ) -> (u64, Arc<TransferProgress>) {
        self.generation = self.generation.wrapping_add(1);
        let progress = Arc::new(TransferProgress::default());
        self.phase = UpdatePhase::Downloading {
            release,
            progress: progress.clone(),
            started: now,
        };
        (self.generation, progress)
    }

    pub(crate) fn downloaded(&mut self, generation: u64, result: Result<PreparedUpdate, String>) {
        if generation != self.generation {
            return;
        }
        let UpdatePhase::Downloading { release, .. } = &self.phase else {
            return;
        };
        self.phase = match result {
            Ok(prepared) => UpdatePhase::Ready(prepared),
            Err(error) => UpdatePhase::Failed {
                error,
                release: Some(release.clone()),
            },
        };
    }

    pub(crate) fn menu_closed(&mut self) {
        if matches!(self.phase, UpdatePhase::Current(_)) {
            self.phase = UpdatePhase::Idle;
        }
    }

    pub(crate) fn tick(&mut self, now: Instant) {
        if matches!(self.phase, UpdatePhase::Current(deadline) if now >= deadline) {
            self.phase = UpdatePhase::Idle;
        }
    }
}

impl UpdatePhase {
    pub(crate) fn label(&self) -> Key {
        match self {
            Self::Idle => Key::UpdateCheck,
            Self::Checking(_) => Key::UpdateChecking,
            Self::Available(release) if release.asset.is_none() => Key::UpdateReleasePage,
            Self::Available(_) => Key::UpdateDownload,
            Self::Downloading { progress, .. } if progress.preparing() => Key::UpdatePreparing,
            Self::Downloading { .. } => Key::UpdateDownloading,
            Self::Ready(_) => Key::UpdateRestart,
            Self::Current(_) => Key::UpdateCurrent,
            Self::Failed { .. } => Key::UpdateRetry,
        }
    }

    pub(crate) fn busy(&self) -> bool {
        matches!(self, Self::Checking(_) | Self::Downloading { .. })
    }
}
