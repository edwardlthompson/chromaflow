# Feature: native-desktop

> Standalone Cinnamon window for ChromaFlow. Not a browser tab. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: user launches **ChromaFlow** from the Cinnamon menu or `chromaflow-gui` installed by `chromaflow_*.deb`; the webview shows Cooling (not a blank surface); a second launch focuses the existing window; a native window (no URL bar) shows Cooling / Lighting / Profiles / Support; inventory is **this machine**, not the sample `nct6775` fixture; closing the window stores size in `~/.config/chromaflow/window.json` and the next launch restores it
- ✅ Offline/error behavior: if WebKit host fails to start, stderr explains missing `libwebkit2gtk-4.1-dev` / runtime; GUI still refuses euid 0; Vite `preview` is labelled developer-only in docs
- 🔲 Accessibility: tab buttons are keyboard-reachable; conflict banner uses `role="alert"`; English strings
- 🔲 i18n: move product strings into `apps/desktop/src/locales/en.json` (no new copy in CSS)

## Smoke scenario

1. _Given_ GTK/WebKit **dev** packages are installed and `cargo build -p chromaflow-desktop` succeeds
2. _When_ the user installs `chromaflow_*.deb` and runs `chromaflow-gui` on Cinnamon (X11)
3. _Then_ a native window opens with the Cooling UI visible (not a black WebKit), a second launch focuses it, Cooling lists live hwmon (this host: `it87952` plus temps), the banner shows `fancontrol` / `coolercontrold` if present, and there is no browser chrome

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core` inventory; Tauri `inventory` / `support_dry_run` |
| View | `apps/desktop/src/` |
| Host | `apps/desktop/src-tauri/` |
| Launcher | `packaging/` (`.desktop`, icon) |
| Tests | `tests/test_chromaflow_tauri.py` |
| Wiring | Tauri `invoke_handler` ≤10 extra lines |
## Tests

- Automated: yes — `tests/test_chromaflow_tauri.py` (window label `main`, Vite `base: "./"`, DMA-BUF env, no `set_pwm`, `.desktop` Exec points at the GUI, docs do not treat `xdg-open http://` as product launch)
- Coverage: host commands + launcher file

## Fallback validation

- Why tests are not feasible: compiling the GUI on a machine without WebKit **dev** is a host package gap
- Command: `cargo build -p chromaflow-desktop` after `[HUMAN]` apt; `python3 scripts/agent-run.py feature-gate --stack web`

## Definition of Done

See [`docs/PRODUCT_GAPS.md`](../PRODUCT_GAPS.md) G-APP and BUILD_PLAN Sprint 2.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
- Do not vendor Fan Control or OpenRGB UI code
- Release `chromaflow-gui` embeds `apps/desktop/dist` after `npm run build` and `cargo build -p chromaflow-desktop --features custom-protocol`. Without that feature the webview loads Vite at `http://127.0.0.1:1420` (connection refused = white error page). Vite `base` must be `./` so `./assets/…` loads. Production JS is a classic IIFE (`inlineDynamicImports`) because WebKit custom-protocol CORS skips `type="module"` (CSS still loads, so the window is `#121418` with no header). On NVIDIA + Cinnamon, `main` sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` and `WEBKIT_DISABLE_COMPOSITING_MODE=1` unless the user already set them (DMA-BUF off alone can still be a blank `#121418` surface).
