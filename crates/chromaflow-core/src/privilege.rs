use std::fs;

pub fn euid() -> u32 {
    let Ok(text) = fs::read_to_string("/proc/self/status") else {
        return 0xffff_ffff;
    };
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("Uid:") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let _real = parts.next();
        if let Some(effective) = parts.next() {
            if let Ok(id) = effective.parse() {
                return id;
            }
        }
    }
    0xffff_ffff
}

pub fn refuse_if_root() -> Result<(), String> {
    if euid() == 0 {
        Err("do not run as root; use Install detection support (polkit)".into())
    } else {
        Ok(())
    }
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
