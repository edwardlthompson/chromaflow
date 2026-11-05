//! Startup elevation guard — delegates to `platform/`.

pub fn euid() -> u32 {
    crate::platform::euid()
}

pub fn refuse_if_root() -> Result<(), String> {
    crate::platform::refuse_if_elevated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_root_ok() {
        if euid() == 0 {
            assert!(refuse_if_root().is_err());
        } else {
            assert!(refuse_if_root().is_ok());
        }
    }
}
