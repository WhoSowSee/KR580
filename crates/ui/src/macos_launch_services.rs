use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURL, CFURLRef};
use std::path::Path;

use crate::macos_bundle::{BUNDLE_ID, SNAPSHOT_UTI, SUBPROGRAM_UTI};

const ALL_ROLES: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[repr(u32)]
#[serde(rename_all = "camelCase")]
pub enum HandlerRole {
    Viewer = 2,
    Editor = 4,
    Shell = 8,
}

#[derive(serde::Serialize)]
pub struct HandlerPreference {
    pub content_type: &'static str,
    pub role: HandlerRole,
    pub handler: Option<String>,
}

struct SavedHandler {
    original: HandlerPreference,
    expected: Option<String>,
}

pub struct DefaultHandlers {
    handlers: Vec<SavedHandler>,
}

impl DefaultHandlers {
    /// Captures each document role separately; an absent handler is never explicitly overwritten.
    pub fn capture() -> Self {
        Self::capture_with(default_handler)
    }

    fn capture_with(query: impl Fn(&str, HandlerRole) -> Option<String>) -> Self {
        let mut handlers = Vec::new();
        for content_type in [SNAPSHOT_UTI, SUBPROGRAM_UTI] {
            for role in [HandlerRole::Viewer, HandlerRole::Editor, HandlerRole::Shell] {
                let handler = query(content_type, role);
                handlers.push(SavedHandler {
                    expected: handler.clone(),
                    original: HandlerPreference {
                        content_type,
                        role,
                        handler,
                    },
                });
            }
        }
        Self { handlers }
    }

    pub fn originals(&self) -> impl Iterator<Item = &HandlerPreference> {
        self.handlers.iter().map(|handler| &handler.original)
    }

    /// Chooses KR580 where a prior default can be restored; bundle claims cover unassigned roles.
    pub fn apply(&mut self) -> Result<(), String> {
        self.apply_with(default_handler, |content_type, role, handler| {
            set_handler(content_type, role as u32, handler)
        })
    }

    fn apply_with(
        &mut self,
        query: impl Fn(&str, HandlerRole) -> Option<String>,
        mut set: impl FnMut(&str, HandlerRole, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        for saved in &self.handlers {
            let current = query(saved.original.content_type, saved.original.role);
            if saved.original.handler.is_some()
                && current != saved.expected
                && current.as_deref() != Some(BUNDLE_ID)
            {
                return Err(format!(
                    "default handler changed before installation: {}",
                    saved.original.content_type
                ));
            }
        }
        for saved in &mut self.handlers {
            if saved.original.handler.is_none() || saved.expected.as_deref() == Some(BUNDLE_ID) {
                continue;
            }
            let result = set(saved.original.content_type, saved.original.role, BUNDLE_ID);
            let current = query(saved.original.content_type, saved.original.role);
            if result.is_ok() || current.as_deref() == Some(BUNDLE_ID) {
                saved.expected = Some(BUNDLE_ID.to_owned());
            }
            result?;
            if current.as_deref() != Some(BUNDLE_ID) {
                return Err(format!(
                    "default handler was not accepted: {}",
                    saved.original.content_type
                ));
            }
        }
        Ok(())
    }

    /// Restores only defaults that still match the value installed by this operation.
    pub fn rollback(&mut self) -> Result<(), String> {
        self.rollback_with(default_handler, |content_type, role, handler| {
            set_handler(content_type, role as u32, handler)
        })
    }

    fn rollback_with(
        &mut self,
        query: impl Fn(&str, HandlerRole) -> Option<String>,
        mut set: impl FnMut(&str, HandlerRole, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut errors = Vec::new();
        for saved in self.handlers.iter_mut().rev() {
            if saved.original.handler == saved.expected {
                continue;
            }
            if query(saved.original.content_type, saved.original.role) != saved.expected {
                errors.push(format!(
                    "default handler rollback conflict: {}",
                    saved.original.content_type
                ));
                continue;
            }
            if let Some(original) = &saved.original.handler {
                match set(saved.original.content_type, saved.original.role, original) {
                    Ok(()) => saved.expected = Some(original.clone()),
                    Err(error) => errors.push(error),
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

fn default_handler(content_type: &str, role: HandlerRole) -> Option<String> {
    let content_type = CFString::new(content_type);
    // SAFETY: The CFString is live; a non-null Copy result is an owned CFStringRef.
    let handler = unsafe {
        LSCopyDefaultRoleHandlerForContentType(content_type.as_concrete_TypeRef(), role as u32)
    };
    if handler.is_null() {
        return None;
    }
    // SAFETY: The non-null Copy result carries one retain which this CFString guard releases.
    Some(unsafe { CFString::wrap_under_create_rule(handler) }.to_string())
}

fn set_handler(content_type: &str, role: u32, handler: &str) -> Result<(), String> {
    let content_type = CFString::new(content_type);
    let handler = CFString::new(handler);
    // SAFETY: Both CFStrings remain live for the synchronous framework call.
    let status = unsafe {
        LSSetDefaultRoleHandlerForContentType(
            content_type.as_concrete_TypeRef(),
            role,
            handler.as_concrete_TypeRef(),
        )
    };
    if status == 0 {
        Ok(())
    } else {
        Err(format!(
            "setting the default handler for {content_type} failed: {status}"
        ))
    }
}

pub fn register_bundle(bundle: &Path) -> Result<(), String> {
    let url = CFURL::from_path(bundle, true).ok_or_else(|| {
        format!(
            "app bundle path is not a valid file URL: {}",
            bundle.display()
        )
    })?;
    // SAFETY: `url` owns a valid CFURLRef for the duration of the framework call.
    let status = unsafe { LSRegisterURL(url.as_concrete_TypeRef(), 1) };
    if status == 0 {
        Ok(())
    } else {
        Err(format!(
            "LSRegisterURL {} failed: {status}",
            bundle.display()
        ))
    }
}

pub fn set_default_handlers() -> Result<(), String> {
    for content_type in [SNAPSHOT_UTI, SUBPROGRAM_UTI] {
        set_handler(content_type, ALL_ROLES, BUNDLE_ID)?;
    }
    Ok(())
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSRegisterURL(url: CFURLRef, update: u8) -> i32;
    fn LSCopyDefaultRoleHandlerForContentType(content_type: CFStringRef, role: u32) -> CFStringRef;
    fn LSSetDefaultRoleHandlerForContentType(
        content_type: CFStringRef,
        role: u32,
        handler: CFStringRef,
    ) -> i32;
}

#[cfg(test)]
mod tests;
