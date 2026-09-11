# Support script

Polkit action: `org.chromaflow.install-support`  
Pinned Exec: `/usr/libexec/chromaflow/install-support.sh`  
Repo copy: [`scripts/install-support.sh`](../scripts/install-support.sh)

## Dry-run (this milestone, CI, default)

```bash
bash scripts/install-support.sh --dry-run
# or
chromaflow support --dry-run
```

The Support tab in the Tauri GUI calls the same dry-run via `support_dry_run` (never `--apply`). Vite preview uses a fixture so the browser path stays unprivileged.

Must not call `apt-get`, `modprobe`, `usermod`, `udevadm`, or write under `/etc`.

JSON fields (stable):

- `would_install` — apt package names from YAML
- `would_load` — safe allowlist module names selected for this machine
- `skipped_experimental` — experimental rows omitted unless `--advanced`
- `skipped_not_installed` — e.g. `zenpower` when `dpkg -s` would fail
- `udev_path` — would write `/etc/udev/rules.d/60-chromaflow.rules` (does not delete OpenRGB rules)
- `logout_required` — true when groups `i2c`/`plugdev` would be added
- `reboot_required` — true only for rows marked `reboot-needed`
- `warnings` — large packages such as `linux-modules-extra-$(uname -r)`

## Apply (stubbed until a .deb installs the pinned path)

`--apply` exits non-zero unless all of:

1. `PKEXEC_UID` is set (pkexec)
2. `CHROMAFLOW_POLKIT=1` is set by the policy
3. `realpath` of `$0` is exactly `/usr/libexec/chromaflow/install-support.sh`

The policy file is [`packaging/polkit/org.chromaflow.install-support.policy`](../packaging/polkit/org.chromaflow.install-support.policy). CI must not install it.

When apply is enabled later on Mint/Ubuntu it will:

1. Select YAML rows from lspci/lsusb/DMI (never invent module names)
2. `apt-get install -y --` packages from YAML argv only (`^[a-zA-Z0-9._+-]+$`)
3. Write `60-chromaflow.rules`; leave existing OpenRGB udev files
4. `usermod -aG i2c,plugdev` for the pkexec user; JSON `logout_required`
5. `modprobe` **safe** names independently; ignore failures; `--advanced` for experimental
6. **Not** run `sensors-detect`
7. `udevadm control --reload-rules && udevadm trigger`
8. Optional `--load-at-boot` → `/etc/modules-load.d/chromaflow.conf` (allowlist names only)

`zenpower` is skipped unless already a dpkg. Never uninstall or blacklist NVIDIA proprietary drivers. `nouveau` is only recommended if it is already the running GPU driver.

## Allowlist files

Under [`data/`](../data/): `packages.yaml`, `modules-safe.yaml`, `modules-experimental.yaml`, `udev.yaml`. Names must match `^[a-zA-Z0-9_-]+$` for modules and `^[a-zA-Z0-9._+-]+$` for apt packages.
