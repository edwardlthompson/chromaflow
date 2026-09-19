#!/bin/sh
# Install one ChromaFlow deb. Does not stop the fan daemon. Does not write PWM.
set -eu
if [ "$#" -ne 2 ]; then
  echo "usage: install-update.sh DEB SHA256" >&2
  exit 2
fi
DEB="$1"
SHA=$(printf '%s' "$2" | tr 'A-F' 'a-f')
UID_WANT="${PKEXEC_UID:-}"
if [ -z "$UID_WANT" ] || [ "${#SHA}" -ne 64 ]; then
  echo "refusing arguments" >&2
  exit 2
fi
case "$SHA" in
  *[!0-9a-f]*) echo "refusing digest" >&2; exit 2 ;;
esac
if [ -L "$DEB" ] || [ ! -f "$DEB" ]; then
  echo "refusing path" >&2
  exit 2
fi
REAL=$(realpath -e "$DEB")
ROOT="/run/user/${UID_WANT}/chromaflow-update"
PARENT=$(dirname "$REAL")
NAME=$(basename "$REAL")
if [ "$PARENT" != "$ROOT" ]; then
  echo "refusing path" >&2
  exit 2
fi
printf '%s\n' "$NAME" | grep -Eq '^chromaflow_[0-9]+\.[0-9]+\.[0-9]+_amd64\.deb$' || {
  echo "refusing name" >&2
  exit 2
}
OWNER=$(stat -c '%u' "$REAL")
MODE=$(stat -c '%a' "$REAL")
if [ "$OWNER" != "$UID_WANT" ] || [ "$MODE" != "600" ]; then
  echo "refusing mode" >&2
  exit 2
fi
GOT=$(sha256sum "$REAL" | awk 'NR==1 { print $1 }')
if [ "$GOT" != "$SHA" ]; then
  echo "digest mismatch" >&2
  exit 2
fi
VER=$(printf '%s\n' "$NAME" | sed -n 's/^chromaflow_\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)_amd64\.deb$/\1/p')
PKG=$(dpkg-deb -f "$REAL" Package)
ARCH=$(dpkg-deb -f "$REAL" Architecture)
DVER=$(dpkg-deb -f "$REAL" Version)
if [ "$PKG" != "chromaflow" ] || [ "$ARCH" != "amd64" ] || [ "$DVER" != "$VER" ]; then
  echo "control mismatch" >&2
  exit 2
fi
export CHROMAFLOW_SKIP_SESSION=1
exec dpkg -i "$REAL"
