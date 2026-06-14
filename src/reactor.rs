use crate::{
    ai::AiProvider,
    config::{InputScope, PaneSelector, ReactorConfig, Rule},
    process::{ProcessInspector, label_for_process_tree},
    redaction::Redactor,
    tmux::Tmux,
};
use tracing::info;

pub async fn run_once(
    config: &ReactorConfig,
    tmux: &dyn Tmux,
    process: &dyn ProcessInspector,
    ai: &dyn AiProvider,
    target: &str,
) -> anyhow::Result<()> {
    let redactor = Redactor::new(&config.redaction)?;
    for rule in config.rules.iter().filter(|rule| rule.enabled) {
        let prompt = build_rule_prompt(rule, tmux, process, &redactor, target)?;
        crate::lazy_trace!(|| format!("prompt for rule {}:\n{}", rule.name, prompt));
        let raw = ai.complete(&rule.model, &prompt).await?;
        let title = sanitize_title(&raw);
        tmux.write_target(&rule.target, target, &title)?;
        info!(rule = %rule.name, target = %target, value = %title, "updated tmux target");
    }
    Ok(())
}

pub fn build_rule_prompt(
    rule: &Rule,
    tmux: &dyn Tmux,
    process: &dyn ProcessInspector,
    redactor: &Redactor,
    target: &str,
) -> anyhow::Result<String> {
    let panes = match rule.input.panes {
        PaneSelector::CurrentPane => tmux.panes(target)?.into_iter().take(1).collect(),
        PaneSelector::CurrentWindow => tmux.panes(target)?,
        PaneSelector::AllWindows => tmux.panes(":")?,
    };

    let mut prompt = String::new();
    prompt.push_str(&rule.prompt);
    prompt.push_str("\n\n<context>\n");

    for pane in panes {
        let label = label_for_process_tree(pane.pid, &pane.command, &pane.path, process);
        let capture = tmux.capture(&pane.id, rule.input.capture_head, rule.input.capture_tail)?;
        prompt.push_str(&format!("<pane id=\"{}\">\n", pane.id));
        write_optional_context(&mut prompt, &rule.input, "cwd", &pane.path);
        write_optional_context(&mut prompt, &rule.input, "process", &label);
        prompt.push_str("<head>\n");
        prompt.push_str(&redactor.redact(&capture.head));
        prompt.push_str("\n</head>\n<tail>\n");
        prompt.push_str(&redactor.redact(&capture.tail));
        prompt.push_str("\n</tail>\n</pane>\n");
    }

    prompt.push_str("</context>\n");
    Ok(prompt)
}

fn write_optional_context(prompt: &mut String, input: &InputScope, name: &str, value: &str) {
    match name {
        "cwd" if !input.include_cwd => {}
        "process" if !input.include_process_tree => {}
        _ => prompt.push_str(&format!("<{name}>{value}</{name}>\n")),
    }
}

pub fn sanitize_title(raw: &str) -> String {
    raw.lines()
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .trim_matches('`')
        .chars()
        .filter(|c| !c.is_control())
        .take(80)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ai::AiProvider,
        config::{ProviderKind, Target},
        process::ProcessInfo,
        redaction::Redactor,
        tmux::{Pane, PaneCapture},
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct FakeTmux {
        writes: Mutex<Vec<String>>,
    }

    impl Tmux for FakeTmux {
        fn panes(&self, _target: &str) -> anyhow::Result<Vec<Pane>> {
            Ok(vec![Pane {
                id: "%1".to_string(),
                window_id: "@1".to_string(),
                pid: 10,
                command: "volta-shim".to_string(),
                path: "/tmp/toolbox".to_string(),
                title: String::new(),
            }])
        }

        fn capture(
            &self,
            _pane_id: &str,
            _head: usize,
            _tail: usize,
        ) -> anyhow::Result<PaneCapture> {
            Ok(PaneCapture {
                head: "OPENAI_API_KEY=sk-123456789012345678901234".to_string(),
                tail: "working on tmux".to_string(),
            })
        }

        fn write_target(
            &self,
            _target: &Target,
            _tmux_target: &str,
            value: &str,
        ) -> anyhow::Result<()> {
            self.writes.lock().unwrap().push(value.to_string());
            Ok(())
        }
    }

    struct FakeProcess;

    impl ProcessInspector for FakeProcess {
        fn children(&self, _pid: u32) -> anyhow::Result<Vec<ProcessInfo>> {
            Ok(vec![ProcessInfo {
                pid: 11,
                ppid: 10,
                command: "node".to_string(),
                args: "node /x/codex".to_string(),
            }])
        }
    }

    struct FakeAi;

    #[async_trait]
    impl AiProvider for FakeAi {
        async fn complete(
            &self,
            _model: &crate::config::ModelConfig,
            _prompt: &str,
        ) -> anyhow::Result<String> {
            Ok("Toolbox Tmux\nextra".to_string())
        }
    }

    #[test]
    fn prompt_includes_unwrapped_process_and_redacts() {
        let mut config = ReactorConfig::default();
        config.rules[0].model.provider = ProviderKind::Mock;
        let redactor = Redactor::new(&config.redaction).unwrap();
        let prompt = build_rule_prompt(
            &config.rules[0],
            &FakeTmux {
                writes: Mutex::new(vec![]),
            },
            &FakeProcess,
            &redactor,
            "%1",
        )
        .unwrap();
        assert!(prompt.contains("<process>codex</process>"));
        assert!(!prompt.contains("sk-123456789012345678901234"));
    }

    #[tokio::test]
    async fn run_once_writes_sanitized_title() {
        let tmux = FakeTmux {
            writes: Mutex::new(vec![]),
        };
        let config = ReactorConfig::default();
        run_once(&config, &tmux, &FakeProcess, &FakeAi, "@1")
            .await
            .unwrap();
        assert_eq!(tmux.writes.lock().unwrap()[0], "Toolbox Tmux");
    }
}
