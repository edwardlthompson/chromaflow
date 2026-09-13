# Feature: it87-dkms

> Support Extra kernel support lists frankcrawford `it87-dkms` first so dual-ITE boards bind IT8689. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Extra kernel support’s first row is `it87-dkms` (ITE Super I/O DKMS). Present means live `/sys/module/it87/srcversion` matches the DKMS `.ko`, not in-tree `it87`. Install uses apt `it87-dkms` if already in sources, writes `ignore_resource_conflict=1`, and reloads `it87`
- ✅ Offline/error behavior: missing package is an honest error (do not add CoolerControl apt). Never `modprobe it87-dkms`. Never ship `.ko`
- ✅ Accessibility: existing Extra kernel support list and Install buttons
- ✅ i18n: `lighting.extras.it87-dkms`

## Smoke scenario

1. _Given_ Extra kernel support on this Gigabyte X570S
2. _When_ the user opens Support
3. _Then_ ITE Super I/O DKMS is the first extra; after DKMS is loaded it is present, and in-tree `it87` alone is not enough

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/lib/chromaflow_it87.py` `chromaflow_extras.py` `chromaflow_apply.py` |
| View | Support Extra kernel support (`ExtrasList.svelte`) |
| Tests | `tests/test_chromaflow_support.py` `tests/test_chromaflow_lighting.py` |
| Wiring | `scripts/install-support.sh` `--only it87-dkms`; `crates/chromaflow-core/src/support.rs` |

## Tests

- Automated: yes — extra id is first; not in `KERNEL_EXTRAS`; present is srcversion match; apply refuses non-root; no CoolerControl apt source
- Coverage: never `set_pwm`; never `modprobe it8689`

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-SAFE: dual-ITE hosts can install frankcrawford `it87-dkms` from Support without a competitor repo being added by ChromaFlow.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
