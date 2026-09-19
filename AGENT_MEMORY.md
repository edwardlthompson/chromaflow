# Agent Memory

> Centralized index of tech stack, threat models, persistent context, and retrospectives.
> Update only at session startups, milestone boundaries, or major architectural pivots.

## Tech Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Product | ChromaFlow | 0.2.2 | Linux Mint cooling + lighting |
| GUI | Tauri 2 + Svelte | 2.x | Real `apps/desktop/src-tauri` host; Vite preview for browser; `cargo test` skips GUI; GitHub `edwardlthompson/chromaflow` |
| Core | Rust | stable | `crates/chromaflow-core`, `chromaflow-cli` |
| Helper | bash + polkit | - | pinned `install-support.sh`, `install-update.sh`, and `manage-competitors.sh`; GUI pkexec only |
| License | MIT + NOTICE | - | No OpenRGB/Fan Control source |
| Distribution | `chromaflow_*.deb` + optional OpenRGB engine file | helper + GUI/CLI; engine is GPL sibling, not in git |
## Active Modules

- ✅ Web / PWA (`modules/web/MODULE.md`)
- ❌ Python (`modules/python/MODULE.md`)
- ❌ Android / F-Droid (`modules/android/MODULE.md`)
- ❌ Node API (`modules/node/MODULE.md`)
- ❌ Lightroom Classic (`modules/lightroom/MODULE.md`)
- ✅ Rust (`modules/rust/MODULE.md`)
- ❌ Go (`modules/go/MODULE.md`)

## Threat Model Checklist

- ✅ `docs/THREAT_MODEL.md` drafted (STRIDE, trust boundaries, top abuse cases, OWASP LLM Top 10 walk)
- ✅ No proprietary closed-source SDKs in production path
- ✅ Opt-in only telemetry (GDPR/CCPA compliant); see `docs/PRIVACY.md`
- ✅ Secrets excluded from VCS (Gitleaks pre-commit)
- ✅ Dependency vulnerability scanning enabled (local `update-deps --audit` + CodeQL + Trivy + Dependabot backup)
- ✅ Input validation at all data boundaries
- ✅ `SECURITY.md` and private vulnerability reporting enabled

## Persistent Context

### Project Purpose

ChromaFlow: Linux Mint cooling (hwmon + NVIDIA GPU fans) and lighting (native hidraw + liquidctl Fusion; OpenRGB only if `CHROMAFLOW_OPENRGB_SDK=1`) with a polkit install-support path. No bundled kernel modules. GUI never root. This Gigabyte X570S sees Fusion 2.0, Keychron Q6 HE, SteelSeries mouse/Arena 7, and the 4090 hybrid AIO; hidraw for those VID:PIDs is plugdev `0660` (udev also ships in the `.deb`). Support Extra kernel support lists `it87-dkms` first (DKMS srcversion, not in-tree) and `liquidctl` second. Live host has IT8689 + IT87952 after frankcrawford DKMS; Super I/O `pwm*` is plugdev `0660`. Take-over is live: `chromaflowd --watchdog` owns ITE PWM + NVIDIA fans (min 20%, never silent 0%). Quiet **Apply to all** persists in `curves.json` after this-machine inventory hydrates (not the sample fixture). Profiles **Apply** runs the same take-over confirm then `pwm_takeover` + `lighting_broadcast`. Login autostart + Cinnamon favorite + color tray. Product GUI is the `chromaflow_*.deb` `chromaflow-gui` (one window per session). Locked brand: `branding/assets/chromaflow-icon.png` and `chromaflow-icon-hero-glass.png`. OpenRGB AppImage may remain on disk but is not spawned (ADR-0021). Native Keychron VIA, Arena/Prime HID, Fusion HID `0xCC` via `fusion-hid.py` serve (liquidctl analog optional; usbfs claim means no hidraw / empty `liquidctl list`), and GPU I2C `0x68` static RGB when the ITE enumerates. Cycle All’s lamp thread does not paint Fusion; Lighting host-ticks Fusion so CPU usage can run while other lamps cycle. Rail tabs keep Cooling/Lighting/Profiles/Support mounted (`.tab-hidden`); OpenRGB inventory is cached after the first Lighting visit. Desktop chrome uses `--fc-*` navy/yellow tokens (`apps/desktop/src/fc-tokens.css`), not Golden Path teal. Type is 16/14/12 with 4/8/12/16 spacing and 44px hit targets. PWM/Support/device-report confirms are an in-app `alertdialog` (Cancel still does nothing). Support first screen is two sentences plus Install all; About/version sits in a Support details footer. Lighting Apply toasts and only that row is busy. First Cooling visit with writable PWM offers Quiet take-over. Auto-calibrate shows a determinate bar. Desktop stays Vite 5.4 / Svelte 4 until the Windows port (GHSA-fx2h-pf6j-xcff is Windows `vite --host` only). GitHub Release SBOM/tag follows CHANGELOG product version, not `.template-version`. Cargo/Tauri/desktop/`chromaflow_*.deb` follow that same `0.2.2`. Support checks `releases/latest` on each launch and can install `chromaflow_<semver>_amd64.deb` only through the pinned helper after a sha256 match. The `linux-deb` job uploads that file when a release is published, not on workflow_dispatch. Scheduled Gitleaks allowlists the privacy sanitizer oracle. Weekly BUILD_PLAN sync opens a PR when `main` is protected. Golden Path Pages demo is live.

### Key Constraints

- Max 300 lines per static data file (UI + i18n), 150 lines per pure logic file
- PWM duty only via watchdog + firmware failsafe (ADR-0018); never silent 0%
- Allowlist-only modprobe names; DMI never becomes a module name
- Trunk-based development with Conventional Commits
- Strict type safety and test coverage budgets

### First-run agent

Cline is the first-run agent in Cursor: GitHub sign-in, FREE models, no API keys (no OPENAI_API_KEY and no Codex CLI on the first-time path). Codex remains optional advanced review only (`/codex-review`) and is not part of onboarding, `/tour`, `/prerelease`, or `/ship`.

Golden Path Settings/About/Feedback are a route stack, not three booleans. Web History API and Android BackHandler pop one level; at home Back stays in the app. Persist key `gp.nav.v1` restores location after theme/crash/share-target (web) and rotation/process death (Android). Home chrome is Settings-only; theme, About, and donate live in sectioned Settings/About menus with dropdowns.

| 2026-09-14 | Tab keep-alive | Four pages stay mounted; OpenRGB not on every rail click; Support dry-run on first visit | First Lighting visit still scans; never chromaflowd |
| 2026-09-14 | Sprint 39 UX leftovers | Unused copy cut; scale/hit tokens; Support toolbox; Apply verb; per-row lighting toast; motion; first-run Quiet; alertdialog; About; calibrate bar | PWM still confirm-then-watchdog; no Golden Path teal |
| 2026-09-14 | Sprint 38 UX audit | Copy/a11y; picker width; names-first rows; Profiles Apply; `--fc-*` tokens | PWM still confirm-then-watchdog; no Golden Path teal |
| 2026-09-14 | Sprint 37 desktop chrome | Labeled rail; Watchdog/Firmware pills; no broadcast hex; Profiles form | PWM path unchanged; gtk-rs not bumped |
| 2026-09-14 | Sprint 36 audit findings | Gitleaks oracle allowlist; crate/deb 0.2.0; CI PR on GH006; Pages 200 | Do not bump gtk-rs 0.20; do not merge template 1.5.0 RP |
| 2026-09-14 | Fusion HID apply + product SBOM tag | HID-first Fusion; Cycle All no longer overwrites CPU usage; CHANGELOG tag gate | Never dump GPU I2C; never chromaflowd; do not merge template 1.5.0 RP |
| 2026-09-14 | Automate 0.2.0 HUMAN + Fusion rows | Actions PR perm; GitHub Release SBOM; hid 048d:5702 Fusion list | Never dump GPU I2C; never chromaflowd |
| 2026-09-14 | Rail icons + 0.2.0 leftovers | Even-height SVG rail; Actions PR / SBOM / gtk-rs HUMAN | Never dump GPU I2C; never chromaflowd |
| 2026-09-14 | Ship 0.2.0 native lighting | OpenRGB opt-in; GPU I2C double-buffer; Vite 8.3.0 | Never dump GPU I2C; never chromaflowd |
| 2026-09-13 | Cycle All Direct fill + Prime live | `set_fill` 108 keys / 120 ms; Prime `0x62` no `0x59` | GPU I2C still off cycle; never chromaflowd |
| 2026-09-13 | Cycle All SOLID hue clock | `lighting_cycle` walks VIA HSV; poll GET id 4 | GET_COLOR store can stay stale; never chromaflowd |
| 2026-09-13 | Buffer-then-broadcast + GET_COLOR retry | `lighting_broadcast` + Fusion `serve`; SOLID/Direct readback | GPU `0x68` ACK can be dummy `0x3f`; never chromaflowd |
| 2026-09-13 | CPU AIO HID + GPU I2C write | Fusion `0xCC` all headers; ITE static `0x13` at `0x68`; host Cycle All | Analog may still miss D_LED Direct; NVIDIA ACK can be dummy `0x3f`; never chromaflowd |
| 2026-09-13 | Sprint 31 desktop stability | Reap OpenRGB Child; skip same GPU percent; libxcb-cursor0 | Restart chromaflow-sdk after install; never chromaflowd |
| 2026-09-13 | Sprint 30 lighting boot | SDK after graphical-session + DISPLAY; one OpenRGB pid restart; Lighting keeps polling leftovers | Do not restart chromaflowd for RGB; proto-5 rescan still deferred |
| 2026-09-13 | ITE pwm4 + GPU DISPLAY | Inventory lists ENODATA pwm4/5; chromaflowd DISPLAY=:0 | Empty SYS_FAN* stay 0 RPM; pump tach is FAN7 |
| 2026-09-13 | Live Balanced take-over | Watchdog owns hwmon3/hwmon4 enable=1 | Cooling copy is conflict-only; tiles 22rem |
| 2026-09-12 | Sprint 20 extras green + temp cards | PWM extra stays green; temp CPU/GPU cards; optional liquidctl apt | No NVML; USB AIO still needs liquidctl CLI |
| 2026-09-12 | Sprint 19 it87-dkms first + pwm_acl smoke | Extra `it87-dkms` is first; present is DKMS srcversion; live pwm* 0660 plugdev | Never add CoolerControl apt; never ship `.ko`; HUMAN take-over smoke still open |
| 2026-09-12 | Sprint 18 GPU fans + ITE names | nvidia-settings fan:0/1; FAN4/FAN5_PUMP/FAN6_PUMP; AIO unit; `ite_primary_missing` | IT8689 PWM stays HUMAN DKMS; never silent 0%; no `.ko` |
| 2026-09-12 | Sprint 17 cooling board | Fans/Pumps/Temps/Curves; schema 2 recipes + mix; auto-calibrate 20–100% duty↔RPM | HUMAN RPM smoke still open; never silent 0%; no Fan Control source |
| 2026-09-12 | PWM watchdog + Report dim | ADR-0018; `--watchdog` + enable=2 failsafe; Report confirm then dim | Live take-over still blocked while fancontrol/coolercontrold show as conflicts |
| 2026-09-12 | Support extra-kernel one-stop | `i2c-nct6775` → `nct6775-i2c`; NVIDIA GPU I2C marks both RGB extras present; fan extras on Support | DIMM SPD / `linux-modules-extra` stay honest; still no PWM; helper rebuild for live `--only it87` |
| 2026-09-12 | Compact hardware cube | Picker-sized bars; Temp/Usage LED toggle; CPU/GPU load | Still no PWM writes |
| 2026-09-12 | Archive Sprint 9–11 | Gauges + hitch rows moved to COMPLETED_TASKS; board origin Sprint 0–1 remains | AGENT board empty; still no PWM writes |
| 2026-09-12 | Per-metric gauges + Lighting hitch | Five graphs; nvidia-smi 2 s GPU; Aorus=CPU, AIO=GPU, Keychron/Arena=combined; skip Lighting inventory poll | Still no PWM writes |
| 2026-09-12 | Calm host USB + live gauge graph | 10 Hz motion; skip unchanged gauge USB; 1 Hz temps-only meter + spectrum/sparkline | GPU still missing from hwmon; still no PWM writes |
| 2026-09-12 | Smooth host Direct + hardware gauges | 33 Hz local paint; OpenRGB probe cache/try_lock; CPU+GPU/RAM/disk meters (ADR-0017) | This host has no nvidia hwmon (GPU meter is CPU-only + note); still no PWM writes |
| 2026-09-12 | Picker wrap + window size + Prime Neo | Cards wrap; `window.json`; hidraw `0x62`/`0x59` | Remaining unknown HID stays research-only; still no PWM writes |
| 2026-09-12 | Bundled OpenRGB engine + Mint .deb | Sibling spawn, research HID, GitHub device form | Engine user-unit follow-up; Prime Neo still no protocol; still no PWM writes |
| 2026-09-12 | Kelvin + OpenRGB modes + per-LED | CCT slider; `UPDATE_MODE`/`UPDATE_SINGLE_LED`; Keychron matrix from SDK | Arena has no SDK modes; Prime Neo still no protocol; still no PWM writes |
| 2026-09-11 | Arena 7 HID + Fusion D_LED | Arena vendor report 0x06; D_LED1/2 resized to 32 | Prime Neo still no protocol; still no PWM writes |
| 2026-09-11 | Unblock hidraw + SDK write | Per-VID udev; helper `--apply`; native OpenRGB AppImage; ADR-0011 color apply | Next: Arena 7 hidraw follow-up; Prime Neo upstream; still no PWM writes |
| 2026-09-11 | Fan Control UX + HUMAN live | OpenRGB server live; PWM take-over declined; Dependabot #1 merged; Cooling is Controls+Curves cards | Next: HUMAN pkexec helper `.deb`; do not pull Svelte 5 main; still no PWM writes |
| 2026-09-11 | Original-brief gap pivot | Browser first-run was Vite fallback; product is `chromaflow-gui`. Catalog `docs/PRODUCT_GAPS.md`; Sprint 2+ on BUILD_PLAN | Do not treat `xdg-open http://127.0.0.1` as product launch; install WebKit GTK **dev** before native smoke |
| 2026-09-11 | Sprint 0 AUTO sign-off | validate-bootstrap --quick, feature-gate web, GitHub CI/Security Scan/CodeQL green on 2a6b118 | Do not wait on ChromaFlow Tauri job for Sprint 0 AUTO; required checks are CI + Security Scan + CodeQL |
| 2026-09-11 | v1.3.0 /push | RP #106 admin-merged; tag+release live; CI green after TBT + instrumented soft skips | Lightroom (#29) stays HUMAN; do not use JUnit Assume on connectedAndroidTest |
| 2026-09-10 | M58–M61 /build | Espresso 3.7 + agent DX; M58–M61 archived; KB-023 path spaces; Release Please #106 open | Do not fold Unreleased until /push+/ship; merge RP is HUMAN |
| 2026-09-10 | M58 Espresso + Android 16 | Pin Espresso 3.7; nav Back smoke on phone; Release checkout order; agent-run PATH/micromamba | Do not empty Unreleased mid-sprint; `/push` then `/ship` owns fold; no `/dev/kvm` → use physical device |
| 2026-09-10 | BUILD_PLAN declutter | Recurring AUTO/AGENT chores left the board; Monday cron owns them | Do not put weekly/monthly 🔲 rows back on BUILD_PLAN or the child template |
| 2026-09-10 | OpenSSF passing | Project 14564 passing; README badge live; Ollama leftover rejected | Do not require Ollama on this template; ADB leftovers need the host with the phones |
| 2026-09-10 | M57 Cursor + docs | Grok Bots, marketplace, skills, registry, Automations, Cloud hooks, Canvas, CLI loop, Settings-only tour/print, optional-stack gaps, ADR-0001 gate, ci-gap registry | Do not treat `/tour` backticks as file paths; do not pre-select ADR-0001 |
| 2026-09-10 | #95/#96 on main | R8 + memory (#95); Settings-only chrome (#96) | Keep minify/shrink + Settings-only chrome; no header ThemeToggle/About/donate |
| 2026-09-09 | M54 Catalog + Lightroom | Lua lint, tagset factory, SDK bump playbook; MODULE sync; catalog navigation + lightroom-plugin | Do not treat bare `feature-catalog.json` as a repo-root path in smoke |
| 2026-09-09 | M53 Android distribution | Landed PR #95 R8; F-Droid/Fastlane/AntiFeatures; UnifiedPush sample; signing runbook | Do not commit keystores or add FCM on the FOSS path |
| 2026-09-09 | Child BUILD_PLAN template | `BUILD_PLAN_TEMPLATE.md` is the product board model; tallies on both plans | Do not put a child playbook back inside this repo’s BUILD_PLAN.md |
| 2026-09-09 | Sprint smoke + board | Slim BUILD_PLAN; M51–M57 = allideas 1–55; `smoke-sprint --require` before next sprint | Do not chain sprints until every ✅ row is smoked (startup + load order) |
| 2026-09-09 | M49 Settings chrome | Home chrome is Settings-only; sectioned menus + dropdowns; ThemeToggle removed | Do not put theme/About/donate back in the header; chips are not settings enums |
| 2026-09-09 | M48 Android runtime budget | Release R8 on; memory limiter/trim Application; Grok Bots optional commercial | Do not add broad keep rules or Credential Manager on FOSS path |
| 2026-09-05 | v1.1.0 /ship | Merged #90/#92/#94; tagged v1.1.0 + GitHub Release; SBOM/OpenVEX on tag; #86 RP blocked on workflow approve; #93 Linux DX pending | Prefer agent release PR when RP workflows need [HUMAN] approve; Unreleased empty before tag |
| 2026-09-01 | M47 Cline-first + GP nav | Cline first-run (no keys); web History + Android BackHandler pop one route; persist gp.nav.v1 | Do not put Codex on /tour /prerelease /ship; device Back smoke is [ADB] |
| 2026-08-28 | v1.0.0 /ship | Cloud agent #81 reviewed+merged; RP #82 cut first stable; Unreleased empty; SBOM+OpenVEX on the tag | Do not merge RP while upgrade-sim still fails on pruned stacks; `Release-As: 1.0.0` beats 0.26.0 |
| 2026-08-28 | /cleanup HUMAN leftovers | Archived 5 script-closed HUMAN rows; CII, Ollama, Android SDK stay 🔲 | Recurring weekly AUTO stays 🔲 |
| 2026-08-28 | HUMAN leftover automation | Scripts close Scorecard, crash-proxy-off, mcp.json copy, weekly Dependabot, CODEOWNERS | CII login, Ollama install, and Android licenses stay HUMAN/ADB |
| 2026-08-28 | /cleanup M46 | Archived M46/M45/M44 AGENT rows; HUMAN leftovers (Scorecard, CII) stay on the board | Recurring weekly AUTO stays 🔲; do not archive Child Playbook templates |
| 2026-08-28 | /build Slack | `--lane auto` on this template; next AGENT is M46-44+ after merge with PC M46 board | Do not run child Sprint 0 init-project on this repo |
| 2026-08-27 | /build scoped gates | Per-row `--scope auto`; failed-stack `--skip-preamble` retry; `/gates` wrap-up stays full | Do not treat docs-only as a skip of Sprint wrap-up `/gates` |
| 2026-08-27 | /build M46 P0 | Force-push deny; go/cargo PATH; System32 bash; Sacred upgrade sim; plugin version; GP JSON schemas | Next: Node/Python About+crash; do not treat /push as force-push approval |
| 2026-08-27 | M46 /allideas board | Uncapped dump command + 75 BUILD_PLAN rows; `/build --lane auto` reads maintainer board | `/ideas` stays the short ranked menu; HUMAN leftovers (Ollama, DPIA, CII) stay off the AGENT queue |
| 2026-08-27 | M45 /ideas round 2 | Health CI filter; init hooks; gates status script; Gradle pins apply; SBOM wait; plugin pack; Rust/Go About+crash; F-Droid+Lightroom gates | Crash-proxy stays off until DPIA; `--force` still matches `git push` approval |
| 2026-08-27 | M44 /ideas ship hygiene | docs/chore no RP bump; Unreleased-first; worktree skip; session-state.json ignore; RP Dependency Review check; Gradle pins; /gates canvas; commit-msg hook | Closed leftover 0.25.1 (#80); local `/gates` needs `pre-commit install --hook-type commit-msg` |
| 2026-08-27 | M43 local resource packing | RAM-capped parallel feature-gate; `/best-of-n` + `/emulator`; Ollama docs no keys | Do not require Ollama/emulator on `/ship`; CI slot cap 2; dummy GUI string never in git |
| 2026-08-27 | M42 local-first deps | `/update-deps` + `upd-cli==0.6.2`; `/ship` uses `--local` gate; Dependabot weekly backup; RP dry-run preview | Do not wait on Dependabot PRs before push; full GH gate stays on `/regress` |
| 2026-08-23 | v0.24.0 /ship | Privacy-feedback feat + TEMPLATE_INDEX fix; RP #72 admin-merge; emptied Unreleased before merge | About-without forbids imports of `about/`; never run that gate via WSL1 `bash` |
| 2026-08-21 | v0.23.0 /ship | Strict pre-release + about-without Windows write retry; RP #71 admin-merge after e2e seed fix | Playwright `addInitScript` re-runs on reload — only seed lastSeen when unset |
| 2026-08-20 | v0.22.0 /ship | Emptied Unreleased before push; RP #70 admin-merge after onStart display-mode fix | `decorView.display` is null in onCreate on the CI emulator — apply `preferredDisplayModeId` in onStart |
| 2026-08-18 | v0.21.0 /ship | CI + Windows upgrade-sim green on feat and fix; RP #69 admin-merge; fold comments leftover notes | Fold is local-only — commit empty Unreleased before push or RP leaves leftovers under the version heading |
| 2026-08-17 | M39 /ideas Windows PATH + ship hygiene | Shared PATH resolver; agent-run drops PYTHONPATH; fold Unreleased onto RP; Q&A GraphQL + HUMAN line | Do not attach Environments to required-check workflows; keep Unreleased empty only after fold+comment |
| 2026-08-17 | M38 /ideas ship-hardening | Branch protection now includes Windows upgrade-sim; Python TEMPLATE_INDEX; RP wait skip; lib files ≤150 | `gh` is not on Git Bash PATH unless Program Files is exported |
| 2026-08-17 | v0.20.0 /ship | Three /ideas rounds + Windows upgrade-sim required; RP #68 admin-merge after CI green on 812a2db | Empty Unreleased before RP; jq.exe CRLF breaks template-index; wait for `release` SBOM |
| 2026-08-17 | /ideas pass 3 | Windows required check; COACH.md; dirty Unreleased notes; weekly AUTO skip; Codespaces verify; citation date; setup-python pin; build_sprint split | Allowlist leftover oversized lib modules; do not pretend they are under 150 |
| 2026-08-17 | /ideas pass 2 | Health template-vs-child; pwsh skip; Windows upgrade-sim CI; UTF-8 health; hint JSON split; root md links; Q&A category; pre-commit | Recurring 🔲 maintenance rows are the honest template next-row |
| 2026-08-17 | /ideas implement-all | Eight ranked items: Windows REPL hang, citation sync, glossary, portable stamp, verify hints, welcome hook, docs links, Discussions | Keep welcome/Discussions opt-in or best-effort; do not fail init when `gh` is missing |
| 2026-08-16 | v0.19.0 /ship | Tour + portable adapters; CI green after push of unpushed feat; RP #67 admin-merge | Hard gate cannot see CI until HEAD is on origin; wait for `release` published SBOM |
| 2026-08-16 | Portable first-run | AGENTS.md SoT + thin pointers; GEMINI.md pointer-only; /tour twin in docs/help | Do not add `.agents/agents.md` (second SoT) |
| 2026-08-16 | Coach layer | BEST_PRACTICES + FIRST_30_DAYS + /coach; justfiles optional | Keep just out of CI |
| 2026-08-16 | M37 gap close | verify.sh + env schema + commit-msg + Dockerfile; post hooks implemented but opt-in | Keep `.agent/` as indexes only |
| 2026-08-16 | M36 bootstrap standards | Extended init-project instead of a second generator; 11 engine unit tests; validate-bootstrap --quick green | Full simulate-template-upgrade still the heavy init dry-run |
| 2026-08-16 | v0.18.3 /ship | Autofix + pre-release green; Codex skip; RP #66 admin-merge; Compose BOM 2026.08.00 | Release assets start empty — wait for `release` published SBOM job |
| 2026-08-16 | v0.18.2 /push | RP #63 admin-merge after maintainer gates; HEAD CI already green; no extra prepare commit | Keep Unreleased empty before RP or notes land under `chore` |
| 2026-08-15 | M35 HUMAN open items | Job-scoped workflow tokens; dismissed 65 PinnedDependencies; merged Dependabot #58–#61; radar max 6 | Rebase Dependabot before Feature Gate on stale lockfiles; Scorecard VulnerabilitiesID lags patched HEAD |
| 2026-08-15 | v0.18.1 /push | `resolve-python.sh` now sets a single executable path so `"$PY"` works; RP #62 admin-merge after CI green | Do not set `PY="py -3"` (quoted invoke fails); keep Unreleased empty before RP or notes land under `chore` |
| 2026-08-15 | M35 /audit | Shared `resolve-python.sh` skips Store stub; About gate restores from HEAD; slim Unreleased; UTF-8 LF rules | Do not run `python3` on Windows PATH; leave Scorecard + Dependabot PRs to HUMAN |
| 2026-08-15 | v0.18.0 /ship | M34 thin steals + extract-zip High cleared via `@puppeteer/browsers` 3.2.0; lockfile needed `proxy-agent` 8 for `npm ci`; RP #56 admin-merge | Generate lockfile with Node 22 / `npm ci` locally after overrides; Windows Store `python3` hangs autofix |
| 2026-08-14 | M34 prior-art thin steals | Honesty labels + handoff + Sacred upgrade column without vendoring cousin repos | Keep fail-open hooks labeled; do not claim `/push` blocks `--force` |
| 2026-08-12 | v0.17.0 /ship | Branding kit + pitch README generator; RP #55 admin-merge; CI green on feat commit | Trigger Release workflow for SBOM if assets empty after tag |
| 2026-08-10 | v0.16.0 /ship | Codex + multi-stack autofix in `/prerelease`; fixed About-without Biome stubs; undici/ip-address/nanoid overrides cleared High alerts after push; RP #51 admin-merge | Prefer Git Bash via agent-run on Windows (System32 bash = WSL1 breaks npm); push security lockfile before expecting Dependabot zero |
| 2026-08-01 | v0.15.2 /ship | Cleared High Dependabot mid-ship (js-yaml, brace-expansion, postcss); RP #50 admin-merge after auto-merge wait | Re-check Dependabot after each push before merge-release-please |
| 2026-07-22 | v0.15.0 /ship | RP #37 merged; fixed duplicate CHANGELOG Unreleased + Node 25 vitest localStorage before CI green | Confirm single Unreleased before push; watch GH Dependabot banner vs triage script |
| 2026-07-21 | M33 Cursor feature integration | Native worktrees + permissions + 7 skills + plugin pack + CLI example; commercial docs deepened | Keep pack script globs wholesale when adding skills; residual Auto-review classifier drift |
| 2026-07-12 | v0.14.1 release | /push merged RP #36; fixed Dependabot alert API + FOSS mcp.json gate | Prefer AUTOMERGE_TOKEN over admin merge fallback for RP |
| 2026-07-12 | M32 audit | Caught GITHUB_TOKEN automerge skipping push CI; Git Bash preference for Windows agent-run | Completed via HUMAN automation; GitHub MCP enabled locally |
| 2026-06-13 | v0.6.0 design system | Cross-stack tokens + i18n scaffold | Restore optional-stack CI jobs after large merge |
| 2026-06-30 | Autonomous /build + HUMAN automation | Grouped human section keeps board readable; automation router backlogs failures only | Release Please PR #20 for 0.12.0 needs human merge |
## Template Provenance

- **Source template:** `edwardlthompson/agent-project-bootstrap` (self-maintained)
- **Template version:** `1.9.0` (see `.template-version`)
- **Last update check:** See `.template-update.json`

### Retrospective — 2026-09-14 (Sprint 39)

- AGENT rows (unused Lighting copy, type/space/hit scale, Support toolbox, Apply verb, Lighting toast/busy, chrome motion, first-run Quiet, in-app alertdialog, About/version, calibrate progress) ✅ and smoked. Sequential because `app.css` / `en.json` overlap. PWM confirms and failsafe unchanged. No Golden Path teal.

### Retrospective — 2026-09-14 (Sprint 38)

- AGENT rows (copy/a11y, picker width, names-first Lighting, Profiles Apply, `--fc-*` tokens) ✅ and smoked. Advanced I2C stays off until checked. Apply uses existing `pwm_takeover` + `lighting_broadcast`. Product CSS aliases Fan Control navy/yellow; Golden Path teal is not imported.

### Retrospective — 2026-09-14 (Sprint 35)

- AGENT rows (Fusion HID apply without hidraw/liquidctl; CHANGELOG product SBOM tag) ✅ and smoked. HUMAN land Unreleased stays on the board (`HUMAN_BACKLOG.md`). CPU usage on motherboard/AIO is host-ticked; Cycle All no longer overwrites Fusion.

### Retrospective — 2026-09-13 (Sprint 32)

- AGENT row (OpenRGB identity CSV + native Keychron VIA SET `0x0A`; OpenRGB stays) ✅ and smoked. Fusion/GPU RGB remain sdk-fallback.

### Retrospective — 2026-09-13 (Sprint 26)

- AGENT row (one `chromaflow-gui` via session socket; product launch is `chromaflow_*.deb`) ✅ and smoked.

### Retrospective — 2026-09-13 (Sprint 25)

- AGENT row (toolbar Auto-calibrate fans then pumps; 15.5rem fan/temp cards) ✅ and smoked. NVIDIA GPU fans are not swept. Recipe Tune fields are load/store only.

### Retrospective — 2026-09-13 (Sprint 24)

- AGENT row (fit curve SVG labels, unmount idle tabs, `collect_cooling`) ✅ and smoked. Host lighting still freezes on Cooling. Software WebKit (`WEBKIT_DISABLE_COMPOSITING_MODE=1`) on the running `tauri dev` still paints slowly until relaunch without it.

### Retrospective — 2026-09-12 (Sprint 21)

- AGENT row (skip present extras, liquidctl, 60s graphs, locked chrome, Balanced take-over) ✅ and smoked. `chromaflowd` is writing duty. Failsafe is `pwm*_enable=2` when the user unit stops.

### Retrospective — 2026-09-12 (Sprint 18)

- AGENT row (NVIDIA fans + ITE names + AIO unit + `ite_primary_missing`) ✅ and smoked. IT8689 PWM stays HUMAN (no shipped `.ko`).

### Retrospective — 2026-09-12 (Sprint 16)

- AGENT rows (live conflicts + PWM ACL/`chromaflowd`) ✅ and smoked. HUMAN live take-over smoke is done (RPM moved; watchdog active).

### Retrospective — 2026-09-10 (M61)

- M61 allideas 161–200 AGENT rows ✅; smoke-sprint passed; archived @ `ca0edfb`. Open PR #106 merge stays HUMAN.
## Milestone 2026-09-11 — /push toward 1.3.0

- Folded Unreleased; pushing main for Release Please #106.
- UnifiedPush ntfy E2E + BroadcastReceiver discovery; HUMAN/ADB waiting automation.
- About lego: Rust CARGO_PKG_VERSION; Python test_about_parity split.
