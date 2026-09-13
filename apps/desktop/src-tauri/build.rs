use std::fs;
use std::io;
use std::path::PathBuf;

fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist")
}

fn ensure_frontend() -> io::Result<()> {
    let dist = dist_dir();
    let html = dist.join("index.html");
    let release = std::env::var("PROFILE").ok().as_deref() == Some("release");
    if html.is_file() {
        if release {
            let body = fs::read_to_string(&html)?;
            if body.contains("\"/assets/") || !body.contains("./assets/") {
                panic!(
                    "release dist needs Vite base './'; run npm run build in apps/desktop"
                );
            }
        }
        return Ok(());
    }
    if release {
        panic!("apps/desktop/dist/index.html missing; run npm run build in apps/desktop");
    }
    fs::create_dir_all(&dist)?;
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("splash/index.html"),
        html,
    )?;
    Ok(())
}

fn main() {
    let html = dist_dir().join("index.html");
    println!("cargo:rerun-if-changed={}", html.display());
    if let Err(err) = ensure_frontend() {
        panic!("frontend dist: {err}");
    }
    tauri_build::build();
}
