# Feature: desktop-update

> Support check and launch check against GitHub Releases. Install is a pinned amd64 `.deb`. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Support has a secondary **Check for updates** button. **Install all** stays the only primary button on that page
- ✅ Launch: one non-blocking check per process. A newer `.deb` opens the existing confirm dialog once. Not now does not install and does not persist a dismiss
- ✅ Empty state: no matching `chromaflow_X.Y.Z_amd64.deb` or missing `sha256` digest shows `no_package` and does not download
- ✅ Error / loading: the button is disabled while checking and shows a status line. Launch failures stay quiet. `dpkg` stderr is shown on the Support status line
- ✅ Offline/error behavior: timeout, missing `curl`, or HTTP failure is `offline`. HTTP 403 is `rate_limited`. No retry loop
- ✅ Accessibility: the control is a labeled button. The confirm dialog keeps its labeled Install / Not now buttons, focuses Install, and Escape cancels
- ✅ i18n: new strings in `apps/desktop/src/locales/en.json` only (`support.update*`). File stays at or under 300 lines

## Smoke scenario

1. _Given_ chromaflow-gui is open on amd64 and GitHub `releases/latest` has a newer `chromaflow_X.Y.Z_amd64.deb` with a `sha256` digest
2. _When_ the process starts, or the user presses **Check for updates** and confirms Install
3. _Then_ the app downloads that asset, checks the digest, and `dpkg -i` runs only through `/usr/libexec/chromaflow/install-update.sh`. `chromaflowd` is not stopped. A second GUI is not started

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/update_check.rs` |
| View | `apps/desktop/src/pages/Support.svelte`, `apps/desktop/src/App.svelte` |
| Install | `scripts/install-update.sh`, `packaging/polkit/`, `scripts/build-chromaflow-deb.sh` |
| Release | `.github/workflows/release.yml` `linux-deb` job |
| Tests | `crates/chromaflow-core` unit tests, `tests/test_chromaflow_support.py` |
| Wiring | Tauri `update_check` / `update_install` in `apps/desktop/src-tauri/src/main.rs` ≤10 lines each |

## Contract

- Product version is `CARGO_PKG_VERSION` (CHANGELOG), never `.template-version`.
- `GET https://api.github.com/repos/edwardlthompson/chromaflow/releases/latest` with `User-Agent: ChromaFlow`, `--max-time 10`, body cap 1 MiB. Drafts and prereleases are not checked.
- Accept only tag `vX.Y.Z` and asset `https://github.com/edwardlthompson/chromaflow/releases/download/vX.Y.Z/chromaflow_X.Y.Z_amd64.deb`. Compare three integers. `0.2.10` is newer than `0.2.9`.
- Install requires asset `digest` `sha256:` plus 64 hex chars. Download goes to `$XDG_RUNTIME_DIR/chromaflow-update/` at mode `0600`, cap 200 MiB. Mismatch deletes the file.
- Helper accepts one realpath under `/run/user/$PKEXEC_UID/chromaflow-update/`, not a symlink, owner `PKEXEC_UID`, mode `0600`, name `chromaflow_[0-9]+.[0-9]+.[0-9]+_amd64.deb`, sha256 argument, and `dpkg-deb -f` of Package `chromaflow`, Architecture `amd64`, Version equal to the filename.
- `dpkg -i` runs with `CHROMAFLOW_SKIP_SESSION=1`. `postinst` then skips `enable-session.sh`. No `systemctl stop`. No `pkill`. No `set_pwm`.
- Non-`x86_64` returns `no_package` and does not download.
- If the helper is missing, open the pinned asset URL with `open_url`. Do not run `dpkg` from the GUI.
- The webview never supplies the download URL. Install re-fetches `releases/latest`.
- `linux-deb` runs only on a tag push whose tag equals `v` plus `scripts/product-release-version.sh`. The job exits 1 if `chromaflow_<semver>_amd64.deb` is missing. Do not skip the OpenRGB engine fetch. Do not attach a `main` build to an older tag.
- Status values: `current`, `available`, `no_package`, `offline`, `rate_limited`, `bad_release`.

## Tests

- Automated: yes — Rust parser tests and `tests/test_chromaflow_support.py`
- Coverage: semver, URL pin, missing digest, helper has no `systemctl stop` or `set_pwm`, release job uploads the deb only on tag push

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

See `docs/FEATURE_MODULES.md`. PWM unchanged. GUI never root.
