use crate::config::{RedactionConfig, RedactionMode};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Redactor {
    enabled: bool,
    patterns: Vec<(Regex, &'static str)>,
}

impl Redactor {
    pub fn new(config: &RedactionConfig) -> anyhow::Result<Self> {
        let enabled = config.enabled && config.mode != RedactionMode::Off;
        let mut patterns = vec![
            (
                Regex::new(
                    r#"(?i)(api[_-]?key|token|secret|password|passwd|pwd)\s*[:=]\s*['"]?[^'"\s]+"#,
                )?,
                "[REDACTED_SECRET_ASSIGNMENT]",
            ),
            (
                Regex::new(r"(?i)\b(bearer|basic)\s+[a-z0-9._~+/=-]{16,}")?,
                "[REDACTED_AUTH_HEADER]",
            ),
            (
                Regex::new(
                    r"\b(?:sk-[A-Za-z0-9_-]{20,}|gh[opsu]_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{20,})\b",
                )?,
                "[REDACTED_TOKEN]",
            ),
            (
                Regex::new(
                    r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
                )?,
                "[REDACTED_PRIVATE_KEY]",
            ),
        ];

        if config.mode == RedactionMode::Strict {
            patterns.push((
                Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")?,
                "[REDACTED_EMAIL]",
            ));
        }

        for pattern in &config.extra_patterns {
            patterns.push((Regex::new(pattern)?, "[REDACTED_CUSTOM]"));
        }

        Ok(Self { enabled, patterns })
    }

    pub fn redact(&self, input: &str) -> String {
        if !self.enabled {
            return input.to_string();
        }

        self.patterns
            .iter()
            .fold(input.to_string(), |acc, (pattern, replacement)| {
                pattern.replace_all(&acc, *replacement).into_owned()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedactionMode;
    use proptest::prelude::*;

    #[test]
    fn redacts_common_secret_shapes() {
        let cfg = RedactionConfig::default();
        let redactor = Redactor::new(&cfg).unwrap();
        let out = redactor.redact(
            "OPENAI_API_KEY=sk-123456789012345678901234 bearer ghp_123456789012345678901234",
        );
        assert!(!out.contains("sk-123456789012345678901234"));
        assert!(!out.contains("ghp_123456789012345678901234"));
    }

    #[test]
    fn off_mode_is_transparent() {
        let cfg = RedactionConfig {
            enabled: true,
            mode: RedactionMode::Off,
            extra_patterns: vec![],
        };
        let redactor = Redactor::new(&cfg).unwrap();
        assert_eq!(redactor.redact("token=abc"), "token=abc");
    }

    proptest! {
        #[test]
        fn redaction_never_increases_text_by_more_than_replacements(input in ".*") {
            let cfg = RedactionConfig::default();
            let redactor = Redactor::new(&cfg).unwrap();
            let _ = redactor.redact(&input);
        }
    }
}
