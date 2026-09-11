pub fn module_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn apt_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_injection() {
        assert!(!module_name_ok("../../x"));
        assert!(!module_name_ok("nvidia-drm;reboot"));
        assert!(!module_name_ok(""));
        assert!(!module_name_ok("nct6775 foo"));
        assert!(module_name_ok("nct6775"));
        assert!(module_name_ok("i2c-dev"));
        assert!(apt_name_ok("libusb-1.0-0"));
        assert!(!apt_name_ok("lm-sensors; rm -rf /"));
    }
}
