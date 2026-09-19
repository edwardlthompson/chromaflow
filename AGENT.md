# AGENT.md — ChromaFlow

Linux Mint fan, pump, and RGB control. Bootstrap stamps `AGENTS.md` only. Do not replace this file from the template.

<!-- agent-brief:one-liner -->
Linux Mint app for fan and pump curves and RGB control.
<!-- /agent-brief:one-liner -->

<!-- agent-brief:keywords -->
hwmon, openrgb, polkit, pwm
<!-- /agent-brief:keywords -->

## Rules

- The GUI never runs as root. The polkit helper is the only root path.
- Never write PWM except through `pwm_apply` or `chromaflow daemon --watchdog`.
- Never stop `chromaflowd`. Failsafe is `pwm*_enable=2`. Never silent 0%.

## First milestone

Ship fan curves and lighting with no bundled kernel modules.
