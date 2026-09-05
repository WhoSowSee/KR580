use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURL, CFURLRef};
use std::path::Path;

use crate::macos_bundle::{BUNDLE_ID, SNAPSHOT_UTI, SUBPROGRAM_UTI};

const ALL_ROLES: u32 = u32::MAX;

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
    let bundle_id = CFString::new(BUNDLE_ID);
    for content_type in [SNAPSHOT_UTI, SUBPROGRAM_UTI] {
        let content_type = CFString::new(content_type);
        // SAFETY: Both CFString values remain alive for the duration of the framework call.
        let status = unsafe {
            LSSetDefaultRoleHandlerForContentType(
                content_type.as_concrete_TypeRef(),
                ALL_ROLES,
                bundle_id.as_concrete_TypeRef(),
            )
        };
        if status != 0 {
            return Err(format!(
                "setting the default handler for {content_type} failed: {status}"
            ));
        }
    }
    Ok(())
}

pub fn is_default_handler() -> bool {
    [SNAPSHOT_UTI, SUBPROGRAM_UTI]
        .iter()
        .all(|content_type| default_handler(content_type).as_deref() == Some(BUNDLE_ID))
}

fn default_handler(content_type: &str) -> Option<String> {
    let content_type = CFString::new(content_type);
    // SAFETY: `content_type` remains alive during the call and a non-null result follows Create Rule.
    let handler = unsafe {
        LSCopyDefaultRoleHandlerForContentType(content_type.as_concrete_TypeRef(), ALL_ROLES)
    };
    if handler.is_null() {
        None
    } else {
        Some(unsafe { CFString::wrap_under_create_rule(handler) }.to_string())
    }
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSRegisterURL(url: CFURLRef, update: u8) -> i32;
    fn LSSetDefaultRoleHandlerForContentType(
        content_type: CFStringRef,
        role: u32,
        handler: CFStringRef,
    ) -> i32;
    fn LSCopyDefaultRoleHandlerForContentType(content_type: CFStringRef, role: u32) -> CFStringRef;
}
