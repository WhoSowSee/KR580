use core_foundation::base::TCFType;
use core_foundation::url::{CFURL, CFURLRef};
use std::path::Path;

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

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSRegisterURL(url: CFURLRef, update: u8) -> i32;
}
