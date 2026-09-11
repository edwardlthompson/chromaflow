use std::fs;
use std::io;
use std::path::PathBuf;

fn ensure_frontend() -> io::Result<()> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest.join("../dist");
    if dist.join("index.html").is_file() {
        return Ok(());
    }
    fs::create_dir_all(&dist)?;
    fs::copy(manifest.join("splash/index.html"), dist.join("index.html"))?;
    Ok(())
}

fn main() {
    if let Err(err) = ensure_frontend() {
        panic!("frontend dist: {err}");
    }
    tauri_build::build();
}
