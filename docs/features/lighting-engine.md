# Feature: lighting-engine

> Sibling OpenRGB AppImage (GPL, separate process). No vendored C++. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting uses the OpenRGB AppImage shipped at `/usr/libexec/chromaflow/OpenRGB.AppImage`; `chromaflow daemon --sdk` keeps `127.0.0.1:6742` up (no Start-menu `.desktop`); Install remains a fallback download into `~/.local/share/chromaflow/`
- ✅ Offline/error behavior: missing binary is a no-op spawn; hash/timeout/wrong host is `role="alert"`; `CHROMAFLOW_NO_SPAWN=1` never execs; FUSE failures mention `libfuse2`
- ✅ Accessibility: engine-missing banner uses `role="alert"`; Install is a labelled button
- ✅ i18n: `lighting.engineMissing` `lighting.installEngine` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ no SDK on 6742 and `/usr/libexec/chromaflow/OpenRGB.AppImage` is executable
2. _When_ the GUI loads (or `chromaflow daemon --sdk`)
3. _Then_ `ensure_sdk` starts one unprivileged server on localhost, the SDK list appears, and PWM is not written

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/openrgb_spawn.rs`, `openrgb_engine.rs` |
| View | `apps/desktop/src/pages/Lighting.svelte` |
| Pin | `data/openrgb-engine.yaml` |
| Tests | `openrgb_spawn` / `openrgb_engine` unit tests + `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — path order, refuse Flatpak/bwrap, hash reject, `CHROMAFLOW_NO_SPAWN`, no `set_pwm`
- Coverage: missing binary does not `Command`; two `ensure_sdk` calls spawn once when the port is down

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Sibling engine may replace the OpenRGB desktop app. No OpenRGB C++ in this tree. PWM remains ADR-0010.
