# Desktop launcher and packages

Cinnamon launches **ChromaFlow** from the menu (`Name=ChromaFlow`, `Exec=chromaflow-gui`). On Windows, the NSIS installer places Start Menu / taskbar shortcuts using `icon.ico`. The private OpenRGB lighting engine is not a second Start-menu app on either OS (ADR-0023).

## Install (Windows x64)

```powershell
.\scripts\build-chromaflow-nsis.ps1
# Install the generated ChromaFlow_*_x64-setup.exe under bundle\nsis\

```

Config lives in `%APPDATA%\chromaflow`. PWM uses the Windows SuperIO backend + `chromaflowd` service (ADR-0022). Set `CHROMAFLOW_LPC_MOCK=1` for UI/CI without hardware IO.

## Install (amd64 Mint 22)

Build and install one package (GUI, CLI, icon, helper):

```bash
bash scripts/build-chromaflow-deb.sh
sudo dpkg -i target/deb/chromaflow_1.0.0_amd64.deb
sudo apt-get install -f

```

Then **Menu → search ChromaFlow** (Settings / Hardware). The menu and panel icon is the color hue-ring mark (`packaging/icons/chromaflow.png`, hicolor 16–512). The package enables a Cinnamon favorite, `/etc/xdg/autostart/chromaflow.desktop`, and `chromaflow-gui.service` / `chromaflow-sdk.service` on `graphical-session.target` so the GUI and OpenRGB sibling start after X11 (`DISPLAY=:0`). Close hides to a color tray icon. If Lighting is missing the GPU or parks the keyboard under Uncontrolled after reboot, run `systemctl --user restart chromaflow-sdk` (do not restart `chromaflowd`). `apps/desktop/src-tauri/icons/icon.ico` is the Windows taskbar/Start icon (NSIS). The `.deb` remains Linux-only. The package script runs `npm run build` and `cargo --features custom-protocol` so the GUI is the Vite UI, and fetches the hashed OpenRGB AppImage into `/usr/libexec/chromaflow/`. `chromaflow daemon --sdk` keeps that sibling on localhost (not a second menu app). Only one window: a second `chromaflow-gui` focuses the first. `TryExec=chromaflow-gui` keeps a broken PATH from showing a dead icon.

WebKit/GTK runtime comes from `Depends`. The packaged OpenRGB engine needs `libfuse2` and `libxcb-cursor0` (also `Depends`). The same `.deb` ships hidraw/i2c udev at `/usr/lib/udev/rules.d/60-chromaflow.rules` and Recommends `liquidctl`, `i2c-tools`, and `lm-sensors`. Kernel `.ko` / frankcrawford `it87-dkms` / `linux-modules-extra-$(uname -r)` cannot be bundled; **Install all** is one password for those remaining helpers. Do not copy an OpenRGB `.desktop`.

Do not commit built `.deb` files. A Mint PPA / Software Manager listing is a follow-up; local `dpkg -i` is enough to be installed and searchable.

Dev-only (no menu): `cargo run -p chromaflow-desktop` needs GTK/WebKit **dev** packages. Vite preview is layout-only.

`packaging/chromaflowd.service` is a user unit (`ExecStart=chromaflow daemon --watchdog`, `ExecStopPost` firmware failsafe). Duty writes need writable `pwm*` and no competing fan daemon.
