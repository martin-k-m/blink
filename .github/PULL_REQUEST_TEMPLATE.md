## What changed and why

<!-- Summarize the change. Link any related issue. -->

## Testing

<!-- What did you run to verify this? Paste relevant command output if
useful. If you added tests, say what they cover; if you didn't, say why
none were needed. -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Docs updated (`README.md` and/or `docs/`) if this changes
      user-facing behavior
- [ ] No fabricated numbers/output — anything new that looks like a
      measurement or score is derived from a concrete, checkable fact
      (see [`docs/analysis.md`](../docs/analysis.md) for the standard)
