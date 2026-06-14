# tmux-ReactorAI

`tmux-ReactorAI` is a Rust tmux plugin that reacts to tmux events, gathers pane
context, redacts sensitive scrollback, asks an AI provider for a string, and
writes that string back into tmux targets such as window names, pane titles, or
status options.

The repository name intentionally keeps Ramiro's mixed-case `tmux-ReactorAI`
spelling. The Rust crate follows Cargo conventions as `tmux-reactor-ai`.

## Prior Art

- `evanpurkhiser/tmux-ai-titles`: daemon, pane/window title generation,
  scrollback hashing, stability delays, and spinner UX.
- `alvinunreal/tmuxai`: tmux pane observation, process args, scrollback capture,
  and AI-context assembly.
- `accessd/tmux-agent-indicator`: agent state hooks, status icons, title styling,
  animation, and automation polish.

`tmux-ReactorAI` generalizes the title-specific idea into a rule engine:

```yaml
rules:
  - name: ai-window-title
    trigger:
      event: window-changed
      stable-ms: 30000
    wait-ms: 120000
    input:
      panes: current-window
      capture-head: 40
      capture-tail: 120
      include-process-tree: true
      include-cwd: true
    prompt: "Generate a short tmux window title. Output only the title."
    model:
      provider: open-ai-compatible
      endpoint-env: OPENAI_BASE_URL
      api-key-env: OPENAI_API_KEY
      model: gpt-5.4-mini
    target: window-name
```

## Architecture

```text
tmux hooks / CLI command
        |
        v
Rule trigger + debounce
        |
        v
Input collector -> process unraveller -> redactor
        |
        v
AI provider facade powered by rig-core
        |
        v
Target writer: rename-window, select-pane -T, set-option, set-window-option
```

The implementation is split into unit-testable seams:

- `config`: YAML schema and defaults.
- `tmux`: tmux CLI adapter plus pure parsers.
- `process`: `volta-shim`/shell child-process label unravelling.
- `redaction`: on-by-default sensitive text filtering before model calls.
- `ai`: provider trait with a `rig-core` OpenAI-compatible implementation.
- `reactor`: rule execution and prompt assembly.
- `logging`: `tracing` setup plus lazy logging helpers for expensive payloads.

## Safety Defaults

Redaction is on by default. The balanced mode removes common token assignments,
bearer/basic headers, GitHub/OpenAI/Slack-style tokens, and private key blocks
before any scrollback leaves the machine. Strict mode also redacts email-shaped
PII. Custom regex patterns are supported through `redaction.extra-patterns`.

## Logging

Structured logging uses `tracing`/`tracing-subscriber`.

```bash
TMUX_REACTOR_AI_LOG=trace tmux-reactor-ai --json-logs run-once --target @1
```

Expensive diagnostic payloads must use the lazy logging helpers:

```rust
tmux_reactor_ai::lazy_trace!(|| format!("prompt:\n{prompt}"));
```

The closure is not evaluated unless the `TRACE` level is enabled.

## tmux Plugin

With TPM:

```tmux
set -g @plugin 'ramarivera/tmux-ReactorAI'
set -g @tmux-reactor-ai-config '~/.config/tmux-reactor-ai/config.yml'
set -g @tmux-reactor-ai-log 'info'
```

Bindings:

- `prefix R`: run enabled rules once for the current window.
- `prefix C-r`: write the default config to `@tmux-reactor-ai-config`.

Hooks:

- `after-new-window`
- `window-layout-changed`

The first implementation keeps hooks as `run-once` calls for testability. A
daemon mode should own real debouncing and long-running event state.

## Tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

Safe tmux smoke tests must use an isolated socket:

```bash
tmux -L tmux-reactor-ai-test -f /dev/null new-session -d -s tri
tmux -L tmux-reactor-ai-test source-file ./tmux-reactor-ai.tmux
tmux -L tmux-reactor-ai-test kill-server
```
