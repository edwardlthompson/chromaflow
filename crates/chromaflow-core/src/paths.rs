//! Config / data / runtime directories — delegates to `platform/`.

use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    crate::platform::config_dir()
}

pub fn data_dir() -> PathBuf {
    crate::platform::data_dir()
}

pub fn runtime_dir() -> PathBuf {
    crate::platform::runtime_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_respects_override() {
        std::env::set_var("CHROMAFLOW_CONFIG_HOME", "/tmp/cf-cfg-test");
        assert_eq!(config_dir(), PathBuf::from("/tmp/cf-cfg-test"));
        std::env::remove_var("CHROMAFLOW_CONFIG_HOME");
    }
}
