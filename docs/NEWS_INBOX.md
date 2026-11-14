# News Inbox

Auto-collected inbox of news, upstream updates, security fixes, and ideas relevant to this repo, appended daily by News Bot.
A Cursor planning agent should review these items and fold the useful ones into BUILD_PLAN; entries are append-only, newest at the bottom.


## 2026-11-13

**News and updates**

- Rust 1.100.0: Cargo gains `cargo::unused_dependencies` lint and dependency cooldowns (minimum publish age); Windows `CommandExt::show_window` stabilized. [source](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)
- hwmon nct6683 patches: fan control for ASRock NCT6686D boards and writable fan curves (pwmN_auto_point*) on NCT6687D-R. [source](https://lkml.indiana.edu/2611.1/10463.html)
- hwmon dell-smm: Dell Pro Precision 7 16 added to the fan-control whitelist. [source](https://lkml.indiana.edu/2611.1/16001.html)

**Tasks for the planning agent**

- [ ] Bump toolchain to Rust 1.100 and enable dependency cooldown — Cooldowns and the unused-deps lint harden supply chain; show_window removes console flashes for Windows helpers. Label: `[AGENT]` · Priority: P2 · https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/
- [ ] Track NCT6686D/NCT6687D-R hwmon support in the device matrix — New writable fan curves on common Nuvoton chips enable hardware curves. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/2611.1/10463.html
- [ ] Add Dell Pro Precision 7 16 to dell-smm detection — Laptop gains manual PWM control in the kernel. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/2611.1/16001.html

## 2026-11-14

**News and updates**

- wfx-loot campaign pushed a CI secret stealer into 17,000+ public repos using stolen GitHub credentials since Nov 10; an npm postinstall hook slipped past GitHub's workflow hold. IOCs: wfx-docs branch, .ci-xxxxxx.sh files, build-audit-*.yml workflows. [source](https://blog.gitguardian.com/wfx-loot-a-ci-secret-stealer-pushed-with-stolen-github-credentials/)
- CoolerControl 5.1.2: stops overwriting other tools' fan settings, fixes RDNA3/4 Zero RPM fans staying stopped, rejects fixed speeds above 100%. [source](https://github.com/codifryed/coolercontrol/releases/tag/5.1.2)
- hwmon dell-smm patch: a pwm overflow could turn the fan OFF; values are now clamped before scaling. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21355.html)
- hwmon fixes for Linux 7.4-rc3 (nct6775 dual-use registers, gpio-fan, max31760) plus an ASRock X600DC-ITX nct6683 board ID. [source](https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21383.html)

**Tasks for the planning agent**

- [ ] Audit the repo for wfx-loot IOCs — Checks for the wfx-docs branch/PR, .ci-*.sh files, build-audit-*.yml workflows and new npm lifecycle hooks, since the campaign steals CI secrets and Cursor credentials. Label: `[AGENT]` · Priority: P1 · https://blog.gitguardian.com/wfx-loot-a-ci-secret-stealer-pushed-with-stolen-github-credentials/
- [ ] Port CoolerControl 5.1.2 safeguards — Hands-off unmanaged fans, RDNA3/4 Zero RPM handling and >100% rejection prevent real fan bugs. Label: `[AGENT]` · Priority: P2 · https://github.com/codifryed/coolercontrol/releases/tag/5.1.2
- [ ] Clamp and validate every pwm value before sysfs writes — Tests that out-of-range input can never become 0% fan speed. Label: `[AGENT]` · Priority: P2 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21355.html
- [ ] Add the X600DC-ITX nct6683 quirk and nct6775 7.4-rc3 note to the support matrix — Keeps the hardware matrix accurate for Nuvoton boards. Label: `[AGENT]` · Priority: P3 · https://lkml.indiana.edu/hypermail/linux/kernel/2611.1/21383.html
