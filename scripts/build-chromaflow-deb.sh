#!/usr/bin/env bash
# Build chromaflow_*.deb: GUI, CLI, Cinnamon menu, helper. No OpenRGB .desktop.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VER="$(bash "$ROOT/scripts/product-release-version.sh")"
OUT="$ROOT/target/deb"
STAGE="$OUT/chromaflow"
PROFILE="${CHROMAFLOW_DEB_PROFILE:-release}"
rm -rf "$STAGE"
install -d "$STAGE/DEBIAN"
install -d "$STAGE/usr/bin"
install -d "$STAGE/usr/libexec/chromaflow"
install -d "$STAGE/usr/share/chromaflow"
install -d "$STAGE/usr/share/applications"
install -d "$STAGE/usr/share/icons/hicolor/scalable/apps"
install -d "$STAGE/usr/share/icons/hicolor/512x512/apps"
install -d "$STAGE/usr/share/icons/hicolor/256x256/apps"
install -d "$STAGE/usr/share/icons/hicolor/128x128/apps"
install -d "$STAGE/usr/share/icons/hicolor/48x48/apps"
install -d "$STAGE/usr/share/icons/hicolor/32x32/apps"
install -d "$STAGE/usr/share/icons/hicolor/16x16/apps"
install -d "$STAGE/usr/share/polkit-1/actions"

GUI="${CHROMAFLOW_DEB_GUI:-}"
CLI="${CHROMAFLOW_DEB_CLI:-}"
if [ -z "$GUI" ] || [ -z "$CLI" ]; then
  if [ "${CHROMAFLOW_DEB_SKIP_CARGO:-}" = "1" ]; then
    echo "CHROMAFLOW_DEB_GUI and CHROMAFLOW_DEB_CLI are required when skipping cargo" >&2
    exit 2
  fi
  (cd "$ROOT/apps/desktop" && npm run build)
  (cd "$ROOT" && cargo build --"$PROFILE" -p chromaflow-cli)
  (cd "$ROOT" && cargo build --"$PROFILE" -p chromaflow-desktop --features custom-protocol)
  TARGET_DIR="$ROOT/target/$PROFILE"
  GUI="${GUI:-$TARGET_DIR/chromaflow-gui}"
  CLI="${CLI:-$TARGET_DIR/chromaflow}"
fi
ENGINE="${CHROMAFLOW_ENGINE_FILE:-}"
if [ -z "$ENGINE" ] && [ "${CHROMAFLOW_DEB_SKIP_CARGO:-}" != "1" ] && [ "${CHROMAFLOW_DEB_SKIP_ENGINE:-}" != "1" ]; then
  ENGINE="$(bash "$ROOT/scripts/fetch-openrgb-engine.sh")"
fi
install -m 0755 "$GUI" "$STAGE/usr/bin/chromaflow-gui"
install -m 0755 "$CLI" "$STAGE/usr/bin/chromaflow"
install -m 0644 "$ROOT/packaging/chromaflow.desktop" "$STAGE/usr/share/applications/chromaflow.desktop"
install -m 0644 "$ROOT/packaging/chromaflow.desktop" "$STAGE/usr/share/applications/chromaflow-gui.desktop"
install -d "$STAGE/etc/xdg/autostart"
install -m 0644 "$ROOT/packaging/autostart/chromaflow.desktop" "$STAGE/etc/xdg/autostart/chromaflow.desktop"
install -m 0644 "$ROOT/packaging/icons/chromaflow.svg" "$STAGE/usr/share/icons/hicolor/scalable/apps/chromaflow.svg"
install -m 0644 "$ROOT/packaging/icons/chromaflow.png" "$STAGE/usr/share/icons/hicolor/512x512/apps/chromaflow.png"
install -m 0644 "$ROOT/apps/desktop/src-tauri/icons/128x128@2x.png" "$STAGE/usr/share/icons/hicolor/256x256/apps/chromaflow.png"
install -m 0644 "$ROOT/apps/desktop/src-tauri/icons/128x128.png" "$STAGE/usr/share/icons/hicolor/128x128/apps/chromaflow.png"
install -m 0644 "$ROOT/apps/desktop/src-tauri/icons/48x48.png" "$STAGE/usr/share/icons/hicolor/48x48/apps/chromaflow.png"
install -m 0644 "$ROOT/apps/desktop/src-tauri/icons/32x32.png" "$STAGE/usr/share/icons/hicolor/32x32/apps/chromaflow.png"
install -m 0644 "$ROOT/apps/desktop/src-tauri/icons/16x16.png" "$STAGE/usr/share/icons/hicolor/16x16/apps/chromaflow.png"
install -m 0755 "$ROOT/scripts/install-support.sh" "$STAGE/usr/libexec/chromaflow/install-support.sh"
install -m 0755 "$ROOT/scripts/manage-competitors.sh" "$STAGE/usr/libexec/chromaflow/manage-competitors.sh"
install -m 0755 "$ROOT/packaging/pwm-failsafe.sh" "$STAGE/usr/libexec/chromaflow/pwm-failsafe.sh"
install -m 0755 "$ROOT/packaging/pwm-acl.sh" "$STAGE/usr/libexec/chromaflow/pwm-acl.sh"
install -m 0755 "$ROOT/packaging/chromaflowd" "$STAGE/usr/bin/chromaflowd"
install -d "$STAGE/usr/lib/systemd/user"
install -m 0644 "$ROOT/packaging/chromaflowd.service" "$STAGE/usr/lib/systemd/user/chromaflowd.service"
install -m 0644 "$ROOT/packaging/chromaflow-sdk.service" "$STAGE/usr/lib/systemd/user/chromaflow-sdk.service"
install -m 0644 "$ROOT/packaging/chromaflow-gui.service" "$STAGE/usr/lib/systemd/user/chromaflow-gui.service"
install -m 0755 "$ROOT/packaging/enable-session.sh" "$STAGE/usr/libexec/chromaflow/enable-session.sh"
if [ -n "${ENGINE:-}" ] && [ -f "$ENGINE" ]; then
  install -m 0755 "$ENGINE" "$STAGE/usr/libexec/chromaflow/OpenRGB.AppImage"
fi
for py in chromaflow_support.py chromaflow_select.py chromaflow_yaml.py chromaflow_apply.py chromaflow_extras.py chromaflow_it87.py chromaflow_competitors.py; do
  install -m 0644 "$ROOT/scripts/lib/$py" "$STAGE/usr/libexec/chromaflow/$py"
done
install -m 0644 "$ROOT"/data/*.yaml "$STAGE/usr/share/chromaflow/"
install -d "$STAGE/usr/lib/udev/rules.d"
install -m 0644 "$ROOT/packaging/udev/60-chromaflow.rules" "$STAGE/usr/lib/udev/rules.d/60-chromaflow.rules"
install -m 0644 "$ROOT/packaging/polkit/org.chromaflow.install-support.policy" \
  "$STAGE/usr/share/polkit-1/actions/org.chromaflow.install-support.policy"
install -m 0644 "$ROOT/packaging/polkit/org.chromaflow.competitors.policy" \
  "$STAGE/usr/share/polkit-1/actions/org.chromaflow.competitors.policy"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: chromaflow
Version: $VER
Section: utils
Priority: optional
Architecture: amd64
Maintainer: ChromaFlow <chromaflow@localhost>
Depends: libwebkit2gtk-4.1-0, libgtk-3-0, libjavascriptcoregtk-4.1-0, libc6, libfuse2, libxcb-cursor0
Recommends: liquidctl, i2c-tools, lm-sensors, libayatana-appindicator3-1
Provides: chromaflow-helper
Conflicts: chromaflow-helper
Replaces: chromaflow-helper
Description: Fan curves and RGB lighting for Linux Mint
 Unprivileged GUI and CLI. Polkit helper is the only root path. PWM via watchdog + failsafe.
EOF
cat > "$STAGE/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
  uid=$(id -u "$SUDO_USER" 2>/dev/null) || uid=""
  if [ -n "$uid" ] && [ -x /usr/libexec/chromaflow/enable-session.sh ]; then
    sudo -u "$SUDO_USER" env \
      XDG_RUNTIME_DIR="/run/user/$uid" \
      DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" \
      /usr/libexec/chromaflow/enable-session.sh || true
  fi
fi
exit 0
EOF
chmod 0755 "$STAGE/DEBIAN/postinst"
DEB="$OUT/chromaflow_${VER}_amd64.deb"
dpkg-deb --build "$STAGE" "$DEB"
echo "$DEB"
