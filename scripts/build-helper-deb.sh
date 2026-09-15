#!/usr/bin/env bash
# Build a local helper .deb that installs the pinned polkit script. No PWM.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE="$ROOT/target/deb/chromaflow-helper"
VER="$(bash "$ROOT/scripts/product-release-version.sh")"
rm -rf "$STAGE"
install -d "$STAGE/DEBIAN"
install -d "$STAGE/usr/libexec/chromaflow"
install -d "$STAGE/usr/share/chromaflow"
install -d "$STAGE/usr/share/polkit-1/actions"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: chromaflow-helper
Version: $VER
Section: utils
Priority: optional
Architecture: all
Maintainer: ChromaFlow <chromaflow@localhost>
Description: Polkit helper and allowlist data for ChromaFlow Support apply
EOF
install -m 0755 "$ROOT/scripts/install-support.sh" "$STAGE/usr/libexec/chromaflow/install-support.sh"
install -m 0755 "$ROOT/scripts/manage-competitors.sh" "$STAGE/usr/libexec/chromaflow/manage-competitors.sh"
install -m 0755 "$ROOT/packaging/pwm-failsafe.sh" "$STAGE/usr/libexec/chromaflow/pwm-failsafe.sh"
install -m 0755 "$ROOT/packaging/pwm-acl.sh" "$STAGE/usr/libexec/chromaflow/pwm-acl.sh"
install -m 0644 "$ROOT/scripts/lib/chromaflow_support.py" "$STAGE/usr/libexec/chromaflow/chromaflow_support.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_select.py" "$STAGE/usr/libexec/chromaflow/chromaflow_select.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_yaml.py" "$STAGE/usr/libexec/chromaflow/chromaflow_yaml.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_apply.py" "$STAGE/usr/libexec/chromaflow/chromaflow_apply.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_extras.py" "$STAGE/usr/libexec/chromaflow/chromaflow_extras.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_it87.py" "$STAGE/usr/libexec/chromaflow/chromaflow_it87.py"
install -m 0644 "$ROOT/scripts/lib/chromaflow_competitors.py" "$STAGE/usr/libexec/chromaflow/chromaflow_competitors.py"
install -m 0644 "$ROOT"/data/*.yaml "$STAGE/usr/share/chromaflow/"
install -m 0644 "$ROOT/packaging/polkit/org.chromaflow.install-support.policy" \
  "$STAGE/usr/share/polkit-1/actions/org.chromaflow.install-support.policy"
install -m 0644 "$ROOT/packaging/polkit/org.chromaflow.competitors.policy" \
  "$STAGE/usr/share/polkit-1/actions/org.chromaflow.competitors.policy"
dpkg-deb --build "$STAGE" "$ROOT/target/deb/chromaflow-helper_${VER}_all.deb"
echo "$ROOT/target/deb/chromaflow-helper_${VER}_all.deb"
