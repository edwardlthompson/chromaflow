//! Cross-platform permission tweaks. Unix chmod; no-op elsewhere.

use std::fs;
use std::path::Path;

pub fn set_mode(path: &Path, bits: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        perm.set_mode(bits);
        fs::set_permissions(path, perm).map_err(|e| e.to_string())
    }
    #[cfg(not(unix))]
    {
        let _ = bits;
        let _ = fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(())
    }
}
