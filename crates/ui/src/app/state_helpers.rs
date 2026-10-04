use iced::Theme;
use std::time::{Duration, Instant};

use super::messages::SpeedTier;
use super::state::DesktopApp;
use super::status::StatusKind;
use crate::i18n::{Key, Lang};

impl DesktopApp {
    pub(crate) fn theme(&self, _window: iced::window::Id) -> Option<Theme> {
        Some(crate::view::theme::iced_theme_for_scheme(
            self.preferences.color_scheme,
        ))
    }

    pub(crate) fn set_status(&mut self, kind: StatusKind) {
        if let Some(rendered) = kind.render(self.preferences.lang) {
            self.shell.status = rendered;
            self.shell.status_kind = kind;
        }
    }

    pub(crate) fn set_status_custom(&mut self, text: String) {
        self.shell.status = text;
        self.shell.status_kind = StatusKind::Custom;
    }

    pub(crate) fn apply_language(&mut self, lang: Lang) {
        if self.preferences.lang == lang {
            return;
        }
        let previous_lang = self.preferences.lang;
        relocalize_target_names(
            &mut self.export.export_xlsx_pages,
            &mut self.export.export_xlsx_page_input,
            previous_lang.t(Key::ExportPageNameBase),
            lang.t(Key::ExportPageNameBase),
        );
        relocalize_target_names(
            &mut self.export.export_text_sections,
            &mut self.export.export_text_section_input,
            previous_lang.t(Key::ExportSectionNameBase),
            lang.t(Key::ExportSectionNameBase),
        );
        self.preferences.lang = lang;
        self.refresh_localized_status();
    }

    pub(crate) fn refresh_localized_status(&mut self) {
        if let Some(rendered) = self.shell.status_kind.render(self.preferences.lang) {
            self.shell.status = rendered;
        }
    }

    pub(crate) fn clear_error_notice(&mut self) {
        self.shell.error_notice = None;
        self.shell.error_notice_dismiss_at = None;
    }

    pub(crate) fn show_error_notice(&mut self, message: impl Into<String>) {
        self.shell.error_notice = Some(message.into());
        self.shell.error_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
    }

    pub(crate) fn clear_halt_notice(&mut self) {
        self.execution.halt_notice = None;
        self.execution.halt_notice_dismiss_at = None;
    }

    pub(crate) fn raise_halt_notice(&mut self) {
        self.execution.halt_notice = Some(self.preferences.lang.t(Key::HaltNotice).to_owned());
        self.execution.halt_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
        self.execution.run_blocked_after_halt = true;
    }

    pub(crate) fn run_new_file(&mut self) {
        let epoch = self.document.edit_epoch.wrapping_add(1);
        if !self.dispatch_action(
            crate::backend::AppCommand::ApplyCpuState(Box::default()),
            super::pending::BackendAction::NewFile { edit_epoch: epoch },
        ) {
            return;
        }
        self.document.edit_epoch = epoch;
        self.requests.pending_requests.retain(|_, request| {
            !matches!(
                request,
                super::pending::PendingRequest::CpuEdit { .. }
                    | super::pending::PendingRequest::LoadProgram { .. }
                    | super::pending::PendingRequest::LoadSubprogram { .. }
                    | super::pending::PendingRequest::Import { .. }
            )
        });
        self.execution.running = false;
        self.document.current_snapshot_path = None;
        self.document.current_subprogram_range = None;
        self.document.subprogram_dialog = None;
        self.document.undo_stack.clear();
        self.execution.speed_tier = self.preferences.default_speed;
    }

    pub(crate) fn mark_saved(&mut self) {
        self.document.dirty = false;
        self.document.saved_cpu = self.snapshot.cpu.clone();
    }

    pub(crate) fn recompute_dirty(&mut self) {
        self.document.dirty = self.snapshot.cpu != self.document.saved_cpu
            || self.requests.pending_requests.values().any(|request| {
                matches!(
                    request,
                    super::pending::PendingRequest::CpuEdit {
                        undo: super::pending::UndoPolicy::Record,
                        ..
                    }
                )
            });
    }

    pub(crate) fn apply_speed_tier(&mut self, tier: SpeedTier) {
        self.execution.speed_tier = tier;
        let hz = super::tier_hz(tier);
        let interval = Duration::from_micros(1_000_000 / u64::from(hz.max(1)));
        self.dispatch(crate::backend::AppCommand::SetStepInterval(interval));
        let mode = match tier {
            SpeedTier::Max => crate::backend::RunMode::Burst {
                slice: Duration::from_millis(16),
            },
            _ => crate::backend::RunMode::Paced,
        };
        self.dispatch(crate::backend::AppCommand::SetRunMode(mode));
    }
}

fn relocalize_target_names(
    options: &mut [super::ExportTarget],
    input: &mut String,
    previous_base: &str,
    current_base: &str,
) {
    let selected = options.iter().position(|target| &target.name == input);
    for target in options.iter_mut() {
        if let Some(index) = generated_target_index(&target.name, previous_base) {
            target.name = format!("{current_base} {index}");
        }
    }
    if let Some(index) = selected {
        *input = options[index].name.clone();
    }
}

fn generated_target_index(name: &str, base: &str) -> Option<usize> {
    let suffix = name.strip_prefix(base)?.strip_prefix(' ')?;
    let index: usize = suffix.parse().ok()?;
    (index > 0 && suffix == index.to_string()).then_some(index)
}
