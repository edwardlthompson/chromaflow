"""Fail when product crate/desktop/deb versions drift from CHANGELOG."""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def changelog_version(root: Path) -> str:
    script = root / "scripts" / "product-release-version.sh"
    proc = subprocess.run(
        ["bash", str(script)],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    return proc.stdout.strip()


def cargo_workspace_version(root: Path) -> str:
    text = (root / "Cargo.toml").read_text(encoding="utf-8")
    block = text.split("[workspace.package]", 1)[-1]
    match = re.search(r'^version\s*=\s*"([^"]+)"', block, re.MULTILINE)
    return match.group(1) if match else ""


def json_version(path: Path, key: str = "version") -> str:
    data = json.loads(path.read_text(encoding="utf-8"))
    value = data.get(key)
    return value if isinstance(value, str) else ""


def check_repo(root: Path | None = None) -> list[str]:
    base = root or ROOT
    ver = changelog_version(base)
    errors: list[str] = []
    cargo = cargo_workspace_version(base)
    if cargo != ver:
        errors.append(f"Cargo.toml workspace version {cargo!r} != CHANGELOG {ver!r}")
    tauri = json_version(base / "apps/desktop/src-tauri/tauri.conf.json")
    if tauri != ver:
        errors.append(f"tauri.conf.json version {tauri!r} != CHANGELOG {ver!r}")
    npm = json_version(base / "apps/desktop/package.json")
    if npm != ver:
        errors.append(f"apps/desktop/package.json version {npm!r} != CHANGELOG {ver!r}")
    needle = "product-release-version.sh"
    for rel in ("scripts/build-chromaflow-deb.sh", "scripts/build-helper-deb.sh"):
        text = (base / rel).read_text(encoding="utf-8")
        if needle not in text:
            errors.append(f"{rel} must call {needle}")
        if re.search(r'^VER="0\.', text, re.MULTILINE):
            errors.append(f"{rel} must not hardcode VER")
    install = json.loads((base / "branding/product.json").read_text(encoding="utf-8"))
    blob = str(install.get("install") or "")
    if f"chromaflow_{ver}_amd64.deb" not in blob:
        errors.append(f"branding/product.json install must name chromaflow_{ver}_amd64.deb")
    packaging = (base / "packaging/README.md").read_text(encoding="utf-8")
    if f"chromaflow_{ver}_amd64.deb" not in packaging:
        errors.append(f"packaging/README.md must name chromaflow_{ver}_amd64.deb")
    golden = json_version(base / "examples/web/package.json")
    if golden != "0.1.0":
        errors.append("examples/web package.json must stay Golden Path stub 0.1.0")
    return errors


def main() -> int:
    errors = check_repo()
    if errors:
        print("Product version alignment failed:")
        for item in errors:
            print(f"  {item}")
        return 1
    print(f"Product versions match CHANGELOG {changelog_version(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
