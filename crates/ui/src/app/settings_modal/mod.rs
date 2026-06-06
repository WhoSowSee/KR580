//! Settings dialog state container.
//!
//! `SettingsDialog` holds the live draft state for the modal – which
//! category is selected, the search query, and the draft language /
//! speed values. The dialog edits a draft snapshot rather than
//! mutating `DesktopApp` directly; Cancel rolls back to `original_*`,
//! Save commits and persists.

mod dialog;
mod focus;
mod routing;

#[cfg(test)]
mod tests;

pub(crate) use dialog::SettingsDialog;
pub(crate) use focus::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsCategory, SettingsSection,
};
