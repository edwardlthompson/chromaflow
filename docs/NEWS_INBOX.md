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
