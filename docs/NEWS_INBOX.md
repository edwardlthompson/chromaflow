# News Inbox

Auto-collected inbox of news, upstream updates, security fixes, and ideas relevant to this repo, appended daily by News Bot.
A Cursor planning agent should review these items and fold the useful ones into BUILD_PLAN; entries are append-only, newest at the bottom.


## 2026-11-13

**News and updates**

- Rust 1.100.0: Cargo gains `cargo::unused_dependencies` lint and dependency cooldowns (minimum publish age); Windows `CommandExt::show_window` stabilized. [source](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)
- hwmon nct6683 patches: fan control for ASRock NCT6686D boards and writable fan curves (pwmN_auto_point*) on NCT6687D-R. [source](https://lkml.indiana.edu/2611.1/10463.html)
- hwmon dell-smm: Dell Pro Precision 7 16 added to the fan-control whitelist. [source](https://lkml.indiana.edu/2611.1/16001.html)

**Tasks for the planning agent**

- [x] Bump toolchain to Rust 1.100 and enable dependency cooldown — Cooldowns and the unused-deps lint harden supply chain; show_window removes console flashes for Windows helpers. Label: `[AGENT]` · Priority: P2 · https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/ (synced to BUILD_PLAN)
- [ ] Track NCT6686D/NCT6687D-R hwmon support in the device matrix — New writable fan curves on common Nuvoton chips enable hardware curves. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/2611.1/10463.html
- [ ] Add Dell Pro Precision 7 16 to dell-smm detection — Laptop gains manual PWM control in the kernel. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/2611.1/16001.html

## 2026-11-14

**News and updates**

- wfx-loot campaign pushed a CI secret stealer into 17,000+ public repos using stolen GitHub credentials since Nov 10; an npm postinstall hook slipped past GitHub's workflow hold. IOCs: wfx-docs branch, .ci-xxxxxx.sh files, build-audit-*.yml workflows. [source](https://blog.gitguardian.com/wfx-loot-a-ci-secret-stealer-pushed-with-stolen-github-credentials/)
- CoolerControl 5.1.2: stops overwriting other tools' fan settings, fixes RDNA3/4 Zero RPM fans staying stopped, rejects fixed speeds above 100%. [source](https://github.com/codifryed/coolercontrol/releases/tag/5.1.2)
- hwmon dell-smm patch: a pwm overflow could turn the fan OFF; values are now clamped before scaling. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21355.html)
- hwmon fixes for Linux 7.4-rc3 (nct6775 dual-use registers, gpio-fan, max31760) plus an ASRock X600DC-ITX nct6683 board ID. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21383.html)

**Tasks for the planning agent**

- [x] Audit the repo for wfx-loot IOCs — Checks for the wfx-docs branch/PR, .ci-*.sh files, build-audit-*.yml workflows and new npm lifecycle hooks, since the campaign steals CI secrets and Cursor credentials. Label: `[AGENT]` · Priority: P1 · https://blog.gitguardian.com/wfx-loot-a-ci-secret-stealer-pushed-with-stolen-github-credentials/ (synced to BUILD_PLAN)
- [x] Port CoolerControl 5.1.2 safeguards — Hands-off unmanaged fans, RDNA3/4 Zero RPM handling and >100% rejection prevent real fan bugs. Label: `[AGENT]` · Priority: P2 · https://github.com/codifryed/coolercontrol/releases/tag/5.1.2 (synced to BUILD_PLAN)
- [x] Clamp and validate every pwm value before sysfs writes — Tests that out-of-range input can never become 0% fan speed. Label: `[AGENT]` · Priority: P2 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21355.html (synced to BUILD_PLAN)
- [ ] Add the X600DC-ITX nct6683 quirk and nct6775 7.4-rc3 note to the support matrix — Keeps the hardware matrix accurate for Nuvoton boards. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21383.html

## 2026-11-15

**News and updates**

- dasharo-acpi patch series adds hwmon fan PWM control and four-point EC fan curves (pwmN_enable, pwmN_auto_pointM_*); the EC silently keeps the old curve if a new one is invalid. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/18113.html)
- acer-wmi v3 patch exposes the Acer Nitro AN515-58 4-zone RGB keyboard through the kernel LED multicolor interface. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21570.html)

**Tasks for the planning agent**

- [ ] Support the hwmon pwmN_enable + pwmN_auto_point* EC-curve pattern — Switch to manual before duty writes and read back after a curve write to catch a silent EC reject (dasharo-acpi now, nct6683 later). Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/18113.html
- [ ] Add a kernel LED-multicolor backend for laptop RGB keyboards — Driving /sys/class/leds/*/multi_intensity + brightness supports platform-driver keyboards without HID access. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21570.html

## 2026-11-16

**News and updates**

- nct6683 fan control for ASRock B850 Steel Legend WiFi / BC-250 (NCT6686D) and writable NCT6687D-R fan curves were applied to hwmon-next (likely Linux 7.5); curve points are writable only when the BIOS curve is non-decreasing with temps ≤127 °C. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/22148.html) · [B850](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/22173.html)
- asus-ec-sensors adds PRIME Z590-A, ROG CROSSHAIR X870E DARK HERO and ROG STRIX X670E-F GAMING WIFI (applied Nov 15). [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/24631.html)

**Tasks for the planning agent**

- [x] Mark nct6683 NCT6686D fan control and NCT6687D-R writable curves as kernel 7.5+/hwmon-next and validate curve writes — The EC rejects decreasing duty curves, so check non-decreasing points with temp ≤127 °C before writing and keep the support matrix honest. Label: `[AGENT]` · Priority: P2 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/22148.html (synced to BUILD_PLAN)
- [ ] Add the 3 new asus-ec-sensors boards to the hardware support matrix — Keeps EC temperature/fan readout support accurate for these ASUS boards. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/24631.html

## 2026-11-20

**News and updates**

- Tauri GHSA-w28w-mhc8-qvjv (High, Sep 26): in tauri ≤2.11.5 the internal channel-fetch IPC command skips the ACL check and serves queued large channel responses by a sequential id, so any WebView that can call invoke can read (and drain) payloads meant for another WebView. Fixed in 2.11.6 (latest 2.12.7). It's a repository-only advisory missing from the Global Advisory DB, so Dependabot raises no alert. This repo's Cargo.lock resolves tauri 2.11.5; exposure is low because the app loads only its own local UI in one webview. [advisory](https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv)

**Tasks for the planning agent**

- [x] Bump tauri to 2.11.6 or later (latest 2.12.7) in Cargo.toml/Cargo.lock and smoke the desktop app — the High-severity IPC ACL bypass affects the locked 2.11.5 and Dependabot can't see it. Label: `[AGENT][LOCAL]` · Priority: P2 · https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv (synced to BUILD_PLAN)

## 2026-11-25

**News and updates**

- Better Stack compared the same desktop chat app built in GPUI (the Rust, GPU-drawn UI framework Discord is moving its desktop app to), Tauri, and Electron, looking at memory, app size, frame rate and CPU. Useful background if this repo is choosing or revisiting a desktop shell; I did not run their comparison. [GPUI](https://gpui.rs) · [Better Stack](https://www.youtube.com/shorts/PbNB0MHO0eA)
