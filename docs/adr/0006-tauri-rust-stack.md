# ADR-0006: Tauri and Rust stack

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

The product brief allowed Avalonia + .NET 8 or Tauri + Svelte. Maintainability on Mint 21/22 with a cooling daemon and OpenRGB SDK was the picking rule.

## Decision

Tauri 2 + Svelte GUI, Rust workspace for `chromaflow` / future `chromaflowd`, bash+polkit for install-support.

Keep `examples/web` for bootstrap feature-gate. Product UI lives in `apps/desktop`.

## Alternatives considered

- Avalonia — UX kinship with Fan Control, but no shared code (proprietary app) and weaker sysfs story
- GTK/Qt Python — liquidctl-native, but we want a small failsafe daemon in Rust

### Critique

| Issue | Resolution |
|-------|------------|
| WebKit on Ubuntu 22.04 | `tauri-attempt` apt-installs `libwebkit2gtk-4.1-dev` on 22.04 (best-effort) and 24.04 (required). Product jobs exclude `chromaflow-desktop` so CLI tests stay green without GTK. |

## Consequences

Root `Cargo.toml` workspace members are `crates/*` plus `apps/desktop/src-tauri`. `default-members` are the CLI crates so `cargo test` does not need WebKit. `examples/rust` stays a separate Golden Path crate.
