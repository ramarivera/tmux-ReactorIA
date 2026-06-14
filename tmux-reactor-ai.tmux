set -goq @tmux-reactor-ai-config "$HOME/.config/tmux-reactor-ai/config.yml"
set -goq @tmux-reactor-ai-log "info"

bind-key R run-shell "tmux-reactor-ai --config \"#{@tmux-reactor-ai-config}\" --log \"#{@tmux-reactor-ai-log}\" run-once --target \"#{window_id}\""
bind-key C-r run-shell "tmux-reactor-ai init-config > \"#{@tmux-reactor-ai-config}\""

# Hook entrypoints are intentionally narrow. The daemon mode will graduate these
# into debounced background work; run-once keeps the initial plugin easy to test.
set-hook -g after-new-window "run-shell -b 'tmux-reactor-ai --config \"#{@tmux-reactor-ai-config}\" --log \"#{@tmux-reactor-ai-log}\" run-once --target \"#{window_id}\" >/dev/null 2>&1 || true'"
set-hook -g window-layout-changed "run-shell -b 'tmux-reactor-ai --config \"#{@tmux-reactor-ai-config}\" --log \"#{@tmux-reactor-ai-log}\" run-once --target \"#{window_id}\" >/dev/null 2>&1 || true'"
