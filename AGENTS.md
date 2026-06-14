# tmux-ReactorAI Agent Instructions

This repository builds a standalone Rust tmux plugin. Keep the mixed-case repo
name, but keep Rust crate names, binaries, modules, and features lowercase
kebab/snake case.

## Rules

- Preserve the reactor model: trigger + wait/debounce + input collection +
  prompt/model config + tmux string target.
- Keep redaction on by default for all scrollback sent to AI providers.
- Do not add provider-specific logic outside the `ai` module unless the
  provider trait needs to grow.
- Keep tmux interactions behind the `tmux::Tmux` trait so logic stays unit
  testable.
- Keep process-tree inspection behind `process::ProcessInspector`.
- Use `tracing` for logs. Expensive log payloads must be lazy, using
  `lazy_trace!` or an equivalent guard.
- Live tmux tests must use an isolated socket via `tmux -L <name>` or
  `tmux -S <temp-socket>`. Never kill the default tmux server.

## Verification

Before claiming completion, run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

For tmux integration changes, also run an isolated socket smoke test that
sources `tmux-reactor-ai.tmux`.
