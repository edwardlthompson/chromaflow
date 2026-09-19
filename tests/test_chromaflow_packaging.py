"""chromaflow .deb is the Mint install unit. No OpenRGB Start-menu entry."""
from __future__ import annotations

import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

LIB = Path(__file__).resolve().parent.parent / "scripts" / "lib"
if str(LIB) not in sys.path:
    sys.path.insert(0, str(LIB))

from product_version_align import check_repo  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "build-chromaflow-deb.sh"


class DebPackagingTests(unittest.TestCase):
    def test_script_is_one_package_no_openrgb_desktop(self) -> None:
        text = SCRIPT.read_text(encoding="utf-8")
        self.assertIn("Package: chromaflow", text)
        self.assertIn("chromaflow.desktop", text)
        self.assertIn("chromaflow-gui", text)
        self.assertIn("libwebkit2gtk-4.1-0", text)
        self.assertIn("libfuse2", text)
        self.assertIn("libxcb-cursor0", text)
        self.assertIn("chromaflow-sdk.service", text)
        self.assertIn("chromaflow-gui.service", text)
        self.assertIn("enable-session.sh", text)
        self.assertIn("/etc/xdg/autostart/chromaflow.desktop", text)
        self.assertIn("chromaflow-gui.desktop", text)
        self.assertIn("favorite-apps", (ROOT / "packaging/enable-session.sh").read_text(encoding="utf-8"))
        self.assertIn("fetch-openrgb-engine.sh", text)
        self.assertIn("OpenRGB.AppImage", text)
        self.assertIn("liquidctl", text)
        self.assertIn("i2c-tools", text)
        self.assertIn("60-chromaflow.rules", text)
        self.assertIn("update-desktop-database", text)
        self.assertIn("npm run build", text)
        self.assertIn("--features custom-protocol", text)
        self.assertNotIn("OpenRGB.desktop", text)
        self.assertNotIn("set_pwm", text)
        desktop = (ROOT / "packaging" / "chromaflow.desktop").read_text(encoding="utf-8")
        self.assertIn("Exec=chromaflow-gui", desktop)
        self.assertIn("SingleMainWindow=true", desktop)
        self.assertIn("TryExec=chromaflow-gui", desktop)
        readme = (ROOT / "packaging" / "README.md").read_text(encoding="utf-8")
        self.assertIn("dpkg -i", readme)
        self.assertNotIn("user-local `.desktop` copy", readme)

    def test_staged_deb_paths(self) -> None:
        with tempfile.TemporaryDirectory(prefix="cf-deb-") as raw:
            tmp = Path(raw)
            gui = tmp / "chromaflow-gui"
            cli = tmp / "chromaflow"
            for path in (gui, cli):
                path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
                path.chmod(path.stat().st_mode | stat.S_IEXEC)
            env = os.environ.copy()
            env["CHROMAFLOW_DEB_SKIP_CARGO"] = "1"
            env["CHROMAFLOW_DEB_GUI"] = str(gui)
            env["CHROMAFLOW_DEB_CLI"] = str(cli)
            proc = subprocess.run(
                ["bash", str(SCRIPT)],
                check=False,
                capture_output=True,
                text=True,
                env=env,
                cwd=str(ROOT),
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            ver = subprocess.run(
                ["bash", str(ROOT / "scripts" / "product-release-version.sh")],
                check=True,
                capture_output=True,
                text=True,
                cwd=str(ROOT),
            ).stdout.strip()
            deb = ROOT / "target" / "deb" / f"chromaflow_{ver}_amd64.deb"
            self.assertTrue(deb.is_file(), proc.stdout)
            info = subprocess.run(
                ["dpkg-deb", "-I", str(deb)],
                check=True,
                capture_output=True,
                text=True,
            ).stdout
            self.assertIn("Package: chromaflow", info)
            names = subprocess.run(
                ["dpkg-deb", "-c", str(deb)],
                check=True,
                capture_output=True,
                text=True,
            ).stdout
            self.assertIn("./usr/bin/chromaflow-gui", names)
            self.assertIn("./usr/share/applications/chromaflow.desktop", names)
            self.assertIn("./usr/share/applications/chromaflow-gui.desktop", names)
            self.assertIn("./etc/xdg/autostart/chromaflow.desktop", names)
            self.assertIn("./usr/lib/systemd/user/chromaflow-gui.service", names)
            self.assertIn("./usr/libexec/chromaflow/enable-session.sh", names)
            self.assertIn("./usr/share/icons/hicolor/512x512/apps/chromaflow.png", names)
            self.assertIn("./usr/share/icons/hicolor/16x16/apps/chromaflow.png", names)
            self.assertIn("./usr/share/icons/hicolor/48x48/apps/chromaflow.png", names)
            self.assertIn("./usr/libexec/chromaflow/install-support.sh", names)
            self.assertIn("./usr/libexec/chromaflow/install-update.sh", names)
            self.assertIn("./usr/share/polkit-1/actions/org.chromaflow.install-update.policy", names)
            self.assertIn("./usr/libexec/chromaflow/manage-competitors.sh", names)
            self.assertIn("./usr/libexec/chromaflow/pwm-failsafe.sh", names)
            self.assertIn("./usr/libexec/chromaflow/pwm-acl.sh", names)
            self.assertIn("./usr/libexec/chromaflow/chromaflow_it87.py", names)
            self.assertIn("./usr/lib/systemd/user/chromaflowd.service", names)
            self.assertIn("./usr/lib/systemd/user/chromaflow-sdk.service", names)
            self.assertIn("./usr/bin/chromaflowd", names)
            self.assertIn("./usr/lib/udev/rules.d/60-chromaflow.rules", names)
            self.assertIn("liquidctl", info)
            self.assertNotIn("OpenRGB.desktop", names)


class ProductReleaseSbomTests(unittest.TestCase):
    def test_release_tag_uses_changelog_not_template(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        script = ROOT / "scripts" / "product-release-version.sh"
        text = script.read_text(encoding="utf-8")
        self.assertIn("product-release-version.sh", release)
        self.assertIn("CHANGELOG product version", release)
        self.assertNotIn("does not match .template-version", release)
        self.assertNotIn("cat .template-version", release)
        self.assertIn("CHANGELOG.md", text)
        self.assertNotIn("set_pwm", text)
        self.assertNotIn("i2cdump", text)
        proc = subprocess.run(
            ["bash", str(script)],
            check=True,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.stdout.strip(), "0.2.2")
        spec = (ROOT / "docs/features/product-release-sbom.md").read_text(encoding="utf-8")
        self.assertIn("product-release-version.sh", spec)
        job = release.split("linux-deb:", 1)[1].split("sbom-assets:", 1)[0]
        self.assertIn("github.event_name == 'release'", job)
        self.assertNotIn("workflow_dispatch", job)
        self.assertNotIn("CHROMAFLOW_DEB_SKIP_ENGINE", job)
        self.assertIn("npm ci", job)
        self.assertIn("chromaflow_${VERSION}_amd64.deb", job)
        self.assertIn("exit 1", job)
        self.assertIn("gh release upload", job)


class ProductVersionAlignTests(unittest.TestCase):
    def test_workspace_matches_changelog(self) -> None:
        self.assertEqual(check_repo(ROOT), [])
        self.assertIn(
            "product-release-version.sh",
            (ROOT / "scripts/build-chromaflow-deb.sh").read_text(encoding="utf-8"),
        )
        spec = (ROOT / "docs/features/product-version-align.md").read_text(encoding="utf-8")
        self.assertIn("product-release-version.sh", spec)


if __name__ == "__main__":
    unittest.main()
