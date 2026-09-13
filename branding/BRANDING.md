# Branding kit

Replaceable product identity for projects bootstrapped from **agent-project-bootstrap**.

## Locked ChromaFlow mark (2026-09-13)

| Asset | Role |
|-------|------|
| [`assets/chromaflow-icon.png`](assets/chromaflow-icon.png) | App icon: navy rounded square, full hue ring, gold three-blade fan with clockwise motion trails |
| [`assets/chromaflow-icon-hero-glass.png`](assets/chromaflow-icon-hero-glass.png) | Brand shot: the mark as a physical plaque, neon glow on a glass floor, distant glass wall with glowing circuitry |
| [`../apps/desktop/src-tauri/icons/icon.ico`](../apps/desktop/src-tauri/icons/icon.ico) | Color Windows taskbar / Start / `.exe` icon (16–256px PNG frames) |
| [`../packaging/icons/chromaflow.png`](../packaging/icons/chromaflow.png) | Linux hicolor + Cinnamon menu |
Do not go back to the Golden Path red triangle. Raster PNG is canonical for the hue ring and trails; kit SVGs wrap that PNG. Windows PWM is not in this release — the `.ico` is so a later NSIS/MSI build already has a color taskbar icon.

## Single sources of truth

| Concern | Edit here | Then run |
|---------|-----------|----------|
| Colors, type, spacing | [`design-tokens/design-tokens.json`](../design-tokens/design-tokens.json) | `python3 scripts/sync-design-tokens.py` |
| Logos / favicon / heroes | [`branding/assets/`](assets/) | `python3 scripts/sync-design-tokens.py` |
| Name, pitch, README copy | [`branding/product.json`](product.json) | `python3 scripts/generate-project-readme.py` |
| Voice guidelines | [`branding/voice.md`](voice.md) | (docs only) |
Official color stylesheet (generated): [`official-colors.css`](official-colors.css).

## Asset inventory

| File | Use |
|------|-----|
| `assets/chromaflow-icon.png` | Locked app mark (README + Tauri + `.deb`) |
| `assets/chromaflow-icon-hero-glass.png` | README hero / brand shot |
| `assets/logo-mark.svg` | App mark wrapper; synced to web `icon.svg` / `logo.svg` |
| `assets/logo-mark-mono.svg` | Monochrome / print |
| `assets/logo-wordmark.svg` | Wordmark only |
| `assets/logo-lockup.svg` | Mark + wordmark |
| `assets/favicon.svg` | Browser tab |
| `assets/app-icon-512.svg` | Store-sized wrapper of the locked PNG |
| `assets/readme-hero.svg` | Fallback SVG banner (README uses the PNG hero) |
| `assets/social-preview.svg` | GitHub / OG 1280×640 |
## Clear space & contrast

- Keep at least 1/8 of the mark’s width as padding around the mark.
- Prefer mark-on-dark (`#070b14` / `#0b1220`) or mono mark on light surfaces.
- Check contrast for primary on surface in both light and dark themes after token edits.

## Rebrand checklist

1. Update `meta.name` and colors in `design-tokens/design-tokens.json`.
2. Replace the locked PNG under `branding/assets/` (keep filenames), then regenerate Tauri/packaging sizes.
3. Fill `branding/product.json` (set `"mode": "product"` in child repos).
4. Run `python3 scripts/sync-design-tokens.py`.
5. Run `python3 scripts/generate-project-readme.py`.
6. Align UI strings (`locales` / `strings.xml`), `manifest.webmanifest`, and GitHub About.
7. Keep `icon.ico` in color (never a template/symbolic gray) for Windows taskbar.

## Template vs product README

- `"mode": "template"` (default here) — generator writes only `generated/README.preview.md`; root `README.md` stays the template guide.
- `"mode": "product"` — generator overwrites root `README.md` with the pitch README. **Never** set this on the upstream template.

## Store listing sizes

See [`examples/android/metadata/en-US/images/README.md`](../examples/android/metadata/en-US/images/README.md) for `icon.png` and `featureGraphic.png`. Source art: `chromaflow-icon.png` and `chromaflow-icon-hero-glass.png`.
