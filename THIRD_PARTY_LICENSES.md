# Third-Party Licenses

> Generated and maintained per release. See pre-release gate in `docs/INITIALIZATION_PROMPT.md` Section 7a.

## Project License

This project is licensed under the MIT License. See [`LICENSE`](LICENSE).

## Dependencies

Run license audits for active stacks:

```bash
# Web (npm)
cd examples/web && npx license-checker --production --summary

# Product Rust workspace
grep 'license' Cargo.toml crates/*/Cargo.toml

# Optional Golden Path rust stub
grep 'license' examples/rust/Cargo.toml
```

`[AUTO]` CI runs `scripts/check-license-compliance.sh` on each push.

## Attribution

When bundling dependencies in releases (desktop binary, etc.), include
this file or [`NOTICE`](NOTICE) in the distribution artifact.

Fan Control, OpenRGB, and liquidctl are attributed in NOTICE as inspiration
or optional separate programs — they are not linked into ChromaFlow.

## Incompatible Licenses

`[HUMAN]` must approve any dependency with copyleft licenses (GPL, AGPL) that
may affect distribution. Document exceptions in `DECISION_LOG.md`.
