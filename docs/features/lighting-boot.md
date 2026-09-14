# Feature: lighting-boot

> Start OpenRGB after the graphical session so GPU and Keychron appear after reboot. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: after login, Lighting lists the NVIDIA controller when `/dev/nvidia0` exists and does not park Keychron VID `3434` under Uncontrolled once OpenRGB names include `keychron` or `q6`
- ✅ Offline/error behavior: missing X socket waits 20s then still spawns; missing OpenRGB binary is a no-op; PWM watchdog is never restarted
- ✅ Accessibility: Uncontrolled list still uses the existing research heading; no new unlabeled controls
- ✅ i18n: reuse `lighting.research` (no new strings)

## Smoke scenario

1. _Given_ a user session with `graphical-session.target` and the packaged SDK unit
2. _When_ the machine has just reached the desktop
3. _Then_ `chromaflow-sdk.service` has `DISPLAY=:0`, OpenRGB lists NVIDIA/Keychron (or one `openrgb.pid` restart within 45s), and Lighting keeps polling while research leftovers remain

## Container map

| Layer | Path |
|-------|------|
| Units | `packaging/chromaflow-sdk.service` `packaging/chromaflow-gui.service` `packaging/enable-session.sh` |
| Logic | `crates/chromaflow-core/src/openrgb_boot.rs` `crates/chromaflow-core/src/openrgb_spawn.rs` |
| View | `apps/desktop/src/App.svelte` `apps/desktop/src/lib/research.js` |
| Tests | `tests/test_chromaflow_daemon.py` `tests/test_chromaflow_lighting.py` |
| Decision | `docs/adr/0019-lighting-sdk-graphical-session.md` |

## Tests

- Automated: yes — unit files `WantedBy=graphical-session.target` + SDK `DISPLAY=`; `thin_sdk`; Q6 substring; `skipPoll` uses `researchDevices`; `restart_sdk_once` with `CHROMAFLOW_NO_SPAWN`; no `set_pwm`
- Coverage: no-hid/no-nvidia skips restart; `chromaflowd` stays `WantedBy=default.target`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
- Live: `systemctl --user restart chromaflow-sdk` after X is up; do not restart `chromaflowd`

## Definition of Done

ADR-0019. GUI/CLI never root. Sibling restart never targets the PWM watchdog.
