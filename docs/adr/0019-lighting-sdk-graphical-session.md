# ADR-0019: Lighting SDK after graphical session

- **Status:** Accepted
- **Date:** 2026-09-13
- **Deciders:** ChromaFlow lighting boot
- **Related:** [ADR-0013](0013-bundled-openrgb-appimage.md)

## Context

`chromaflow daemon --sdk` used `WantedBy=default.target` with no `DISPLAY`. OpenRGB enumerated before Cinnamon/NVIDIA/hidraw settled, then stayed on that list because `ensure_sdk` reuses a live `127.0.0.1:6742`. The Lighting tab skipped further polls once any controller existed, so the GPU vanished and the Keychron hidraw sat in Uncontrolled devices until a manual SDK restart.

## Decision

1. `chromaflow-sdk.service` and `chromaflow-gui.service` are `After=` / `WantedBy=graphical-session.target`. SDK sets `DISPLAY=:0` and `XAUTHORITY=%h/.Xauthority` like the GUI. `chromaflowd` stays on `default.target` (PWM must not wait on X).
2. OpenRGB spawn inherits the same X11 env (`CHROMAFLOW_DISPLAY` or `:0`).
3. `daemon --sdk` waits up to 20s for `/tmp/.X11-unix/X0`, then for 45s may SIGTERM **only** `openrgb.pid` once if hidraw VID `3434` or `/dev/nvidia0` is present but SDK names lack `keychron`/`q6` or `nvidia`/`geforce`. Never touch `chromaflowd`.
4. Do not bump `CLIENT_PROTO` to 5 for `REQUEST_RESCAN_DEVICES` (packet 140) in this sprint. Late start plus one sibling restart re-enumerates on protocol 3.

## Alternatives considered

- Protocol-5 rescan without restart — deferred; DATA layout risk vs proto 3 parser
- Restart `chromaflowd` — rejected; PWM watchdog is unrelated

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty DISPLAY or missing X socket | 20s wait then spawn with `:0` + `~/.Xauthority`; no hid/nvidia skips restart |
| Network timeout | N/A — localhost 200ms SDK connect |
| Race vs GUI apply | Restart only in the 45s SDK boot window, once; `lock_sdk` serializes TCP |
| Unhandled exceptions | Spawn errors stay in `openrgb.log`; missing binary is a no-op |
| Existing units | `enable-session.sh` `daemon-reload` + `enable --now` SDK/GUI |

## Consequences

After reboot the sibling engine should see the GPU and Keychron. Workaround remains `systemctl --user restart chromaflow-sdk` (not the watchdog). Proto-5 rescan is a follow-up if USB still lands after that one restart.
