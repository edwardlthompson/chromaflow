//! GitHub release compare. No network. No PWM.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReport {
    pub status: &'static str,
    pub current: String,
    pub latest: String,
    pub asset_url: String,
    pub sha256: String,
}

pub fn parse_semver(raw: &str) -> Option<(u64, u64, u64)> {
    let s = raw.trim().trim_start_matches('v');
    let mut it = s.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next()?.parse().ok()?;
    if it.next().is_some() || s.contains(['-', '+']) {
        return None;
    }
    Some((major, minor, patch))
}

pub fn deb_name(version: &str) -> String {
    format!("chromaflow_{version}_amd64.deb")
}

pub fn pinned_asset_url(version: &str) -> String {
    format!(
        "https://github.com/edwardlthompson/chromaflow/releases/download/v{version}/{}",
        deb_name(version)
    )
}

pub fn parse_sha256_digest(raw: &str) -> Option<String> {
    let hex = raw.strip_prefix("sha256:")?;
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

pub fn sha_arg_ok(hex: &str) -> bool {
    hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn report(status: &'static str, current: &str, latest: &str, url: &str, sha: &str) -> UpdateReport {
    UpdateReport {
        status,
        current: current.into(),
        latest: latest.into(),
        asset_url: url.into(),
        sha256: sha.into(),
    }
}

pub fn classify_release(body: &str, current: &str, arch: &str) -> UpdateReport {
    if arch != "x86_64" {
        return report("no_package", current, "", "", "");
    }
    let Some(cur) = parse_semver(current) else {
        return report("bad_release", current, "", "", "");
    };
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return report("bad_release", current, "", "", "");
    };
    let Some(tag) = value.get("tag_name").and_then(Value::as_str) else {
        return report("bad_release", current, "", "", "");
    };
    let Some(ver) = tag.strip_prefix('v').filter(|s| parse_semver(s).is_some()) else {
        return report("bad_release", current, "", "", "");
    };
    let latest = parse_semver(ver).unwrap_or((0, 0, 0));
    if latest <= cur {
        return report("current", current, ver, "", "");
    }
    let name = deb_name(ver);
    let url = pinned_asset_url(ver);
    let Some(assets) = value.get("assets").and_then(Value::as_array) else {
        return report("no_package", current, ver, "", "");
    };
    let Some(asset) = assets
        .iter()
        .find(|a| a.get("name").and_then(Value::as_str) == Some(name.as_str()))
    else {
        return report("no_package", current, ver, "", "");
    };
    if asset.get("browser_download_url").and_then(Value::as_str) != Some(url.as_str()) {
        return report("bad_release", current, ver, "", "");
    }
    let Some(sha) = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(parse_sha256_digest)
    else {
        return report("no_package", current, ver, "", "");
    };
    report("available", current, ver, &url, &sha)
}
#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn body(url: &str, digest: &str) -> String {
        format!(
            r#"{{"tag_name":"v0.3.0","assets":[{{"name":"chromaflow_0.3.0_amd64.deb","browser_download_url":"{url}","digest":"{digest}"}}]}}"#
        )
    }

    #[test]
    fn semver_orders_patch_numerically() {
        assert!(parse_semver("0.2.10") > parse_semver("0.2.9"));
        assert!(parse_semver("v0.2.1").is_some());
        assert!(parse_semver("0.2").is_none());
        assert!(parse_semver("0.2.1-rc.1").is_none());
    }

    #[test]
    fn accepts_exact_asset_and_rejects_host() {
        let good = pinned_asset_url("0.3.0");
        let hit = classify_release(&body(&good, SHA), "0.2.1", "x86_64");
        assert_eq!(hit.status, "available");
        assert_eq!(hit.asset_url, good);
        let evil = body("https://evil.example/chromaflow_0.3.0_amd64.deb", SHA);
        assert_eq!(
            classify_release(&evil, "0.2.1", "x86_64").status,
            "bad_release"
        );
    }

    #[test]
    fn missing_digest_or_assets_is_no_package() {
        let url = pinned_asset_url("0.3.0");
        assert_eq!(
            classify_release(&body(&url, ""), "0.2.1", "x86_64").status,
            "no_package"
        );
        let empty = r#"{"tag_name":"v0.3.0","assets":[]}"#;
        assert_eq!(
            classify_release(empty, "0.2.1", "x86_64").status,
            "no_package"
        );
        assert_eq!(
            classify_release("{}", "0.2.1", "aarch64").status,
            "no_package"
        );
    }

    #[test]
    fn same_version_is_current() {
        let url = pinned_asset_url("0.2.1");
        let raw = body(&url, SHA).replace("0.3.0", "0.2.1");
        assert_eq!(classify_release(&raw, "0.2.1", "x86_64").status, "current");
        assert!(sha_arg_ok("ab".repeat(32).as_str()));
        assert!(!sha_arg_ok("nope"));
    }
}
