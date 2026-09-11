"""Tauri 2 host exists and CI installs WebKit instead of skipping both distros."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class TauriStubTests(unittest.TestCase):
    def test_desktop_crate_is_tauri(self) -> None:
        cargo = (ROOT / "apps/desktop/src-tauri/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn("tauri-build", cargo)
        self.assertIn("tauri.workspace", cargo)
        main = (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8")
        self.assertIn("refuse_if_root", main)
        self.assertIn("support_dry_run", main)
        self.assertIn("tauri::Builder", main)
        self.assertNotIn("set_pwm", main)

    def test_support_invokes_when_tauri(self) -> None:
        page = (ROOT / "apps/desktop/src/pages/Support.svelte").read_text(encoding="utf-8")
        self.assertIn("__TAURI_INTERNALS__", page)
        self.assertIn("support_dry_run", page)
        self.assertIn("@tauri-apps/api/core", page)

    def test_ci_installs_webkit_on_both_ubuntu(self) -> None:
        workflow = (ROOT / ".github/workflows/chromaflow.yml").read_text(encoding="utf-8")
        self.assertIn("libwebkit2gtk-4.1-dev", workflow)
        self.assertIn("ubuntu-22.04", workflow)
        self.assertIn("ubuntu-24.04", workflow)
        self.assertIn("continue-on-error: ${{ matrix.os == 'ubuntu-22.04' }}", workflow)
        self.assertIn("cargo test --workspace --exclude chromaflow-desktop", workflow)
        self.assertIn("cargo build -p chromaflow-desktop", workflow)

    def test_default_members_skip_tauri(self) -> None:
        cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn("default-members", cargo)
        self.assertIn("crates/chromaflow-cli", cargo)
        self.assertIn("apps/desktop/src-tauri", cargo)


if __name__ == "__main__":
    unittest.main()
