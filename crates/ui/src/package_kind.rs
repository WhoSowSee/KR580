use std::ffi::OsStr;

pub(crate) fn is_managed() -> bool {
    managed_kind(std::env::var_os("KR580_PACKAGE_KIND").as_deref())
}

fn managed_kind(kind: Option<&OsStr>) -> bool {
    matches!(kind.and_then(OsStr::to_str), Some("nix" | "rpm" | "arch"))
}

#[cfg(test)]
mod tests {
    use super::managed_kind;
    use std::ffi::OsStr;

    #[test]
    fn managers_use_user_settings_and_their_own_update_mechanism() {
        for kind in ["nix", "rpm", "arch"] {
            assert!(managed_kind(Some(OsStr::new(kind))));
        }
        for kind in [None, Some(OsStr::new("")), Some(OsStr::new("cargo"))] {
            assert!(!managed_kind(kind));
        }
    }
}
