use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ReactorConfig {
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub redaction: RedactionConfig,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: default_poll_interval_ms(),
            redaction: RedactionConfig::default(),
            rules: vec![Rule::default_window_title()],
        }
    }
}

impl ReactorConfig {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&raw)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Rule {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default = "default_wait_ms")]
    pub wait_ms: u64,
    #[serde(default)]
    pub input: InputScope,
    pub prompt: String,
    #[serde(default)]
    pub model: ModelConfig,
    pub target: Target,
}

impl Rule {
    pub fn default_window_title() -> Self {
        Self {
            name: "ai-window-title".to_string(),
            enabled: true,
            trigger: Trigger::default(),
            wait_ms: 120_000,
            input: InputScope::default(),
            prompt: "Generate a short 1-3 word tmux window title. Output only the title."
                .to_string(),
            model: ModelConfig::default(),
            target: Target::WindowName,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Trigger {
    #[serde(default)]
    pub event: EventKind,
    #[serde(default = "default_stable_ms")]
    pub stable_ms: u64,
}

impl Default for Trigger {
    fn default() -> Self {
        Self {
            event: EventKind::WindowChanged,
            stable_ms: default_stable_ms(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    #[default]
    WindowChanged,
    LayoutChanged,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct InputScope {
    #[serde(default)]
    pub panes: PaneSelector,
    #[serde(default = "default_capture_head")]
    pub capture_head: usize,
    #[serde(default = "default_capture_tail")]
    pub capture_tail: usize,
    #[serde(default = "default_include_process_tree")]
    pub include_process_tree: bool,
    #[serde(default = "default_include_cwd")]
    pub include_cwd: bool,
}

impl Default for InputScope {
    fn default() -> Self {
        Self {
            panes: PaneSelector::CurrentWindow,
            capture_head: default_capture_head(),
            capture_tail: default_capture_tail(),
            include_process_tree: true,
            include_cwd: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PaneSelector {
    CurrentPane,
    #[default]
    CurrentWindow,
    AllWindows,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ModelConfig {
    #[serde(default)]
    pub provider: ProviderKind,
    #[serde(default = "default_endpoint_env")]
    pub endpoint_env: String,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::OpenAiCompatible,
            endpoint_env: default_endpoint_env(),
            api_key_env: default_api_key_env(),
            model: default_model(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    #[default]
    OpenAiCompatible,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    WindowName,
    PaneTitle,
    GlobalOption { name: String },
    WindowOption { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RedactionConfig {
    #[serde(default = "default_redaction_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub mode: RedactionMode,
    #[serde(default)]
    pub extra_patterns: Vec<String>,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            enabled: default_redaction_enabled(),
            mode: RedactionMode::Balanced,
            extra_patterns: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RedactionMode {
    Off,
    #[default]
    Balanced,
    Strict,
}

fn default_poll_interval_ms() -> u64 {
    5_000
}
fn default_stable_ms() -> u64 {
    30_000
}
fn default_wait_ms() -> u64 {
    120_000
}
fn default_capture_head() -> usize {
    40
}
fn default_capture_tail() -> usize {
    120
}
fn default_include_process_tree() -> bool {
    true
}
fn default_include_cwd() -> bool {
    true
}
fn default_endpoint_env() -> String {
    "OPENAI_BASE_URL".to_string()
}
fn default_api_key_env() -> String {
    "OPENAI_API_KEY".to_string()
}
fn default_model() -> String {
    "gpt-5.4-mini".to_string()
}
fn default_redaction_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_rule_config() {
        let raw = r#"
rules:
  - name: title
    enabled: true
    prompt: "Name this"
    target: window-name
"#;
        let cfg: ReactorConfig = serde_yaml::from_str(raw).unwrap();
        assert_eq!(cfg.rules[0].wait_ms, 120_000);
        assert_eq!(cfg.rules[0].input.capture_tail, 120);
        assert!(cfg.redaction.enabled);
    }
}
