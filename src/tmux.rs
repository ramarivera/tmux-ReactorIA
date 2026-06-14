use crate::config::Target;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub id: String,
    pub window_id: String,
    pub pid: u32,
    pub command: String,
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneCapture {
    pub head: String,
    pub tail: String,
}

pub trait Tmux {
    fn panes(&self, target: &str) -> anyhow::Result<Vec<Pane>>;
    fn capture(&self, pane_id: &str, head: usize, tail: usize) -> anyhow::Result<PaneCapture>;
    fn write_target(&self, target: &Target, tmux_target: &str, value: &str) -> anyhow::Result<()>;
}

#[derive(Debug, Default)]
pub struct CliTmux;

impl Tmux for CliTmux {
    fn panes(&self, target: &str) -> anyhow::Result<Vec<Pane>> {
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t",
                target,
                "-F",
                "#{pane_id}\t#{window_id}\t#{pane_pid}\t#{pane_current_command}\t#{pane_current_path}\t#{pane_title}",
            ])
            .output()?;
        parse_panes(&String::from_utf8_lossy(&output.stdout))
    }

    fn capture(&self, pane_id: &str, head: usize, tail: usize) -> anyhow::Result<PaneCapture> {
        let full = Command::new("tmux")
            .args(["capture-pane", "-p", "-t", pane_id, "-S", "-"])
            .output()?;
        let raw = String::from_utf8_lossy(&full.stdout);
        Ok(capture_head_tail(&raw, head, tail))
    }

    fn write_target(&self, target: &Target, tmux_target: &str, value: &str) -> anyhow::Result<()> {
        let mut cmd = Command::new("tmux");
        match target {
            Target::WindowName => {
                cmd.args(["rename-window", "-t", tmux_target, value]);
            }
            Target::PaneTitle => {
                cmd.args(["select-pane", "-t", tmux_target, "-T", value]);
            }
            Target::GlobalOption { name } => {
                cmd.args(["set-option", "-g", name, value]);
            }
            Target::WindowOption { name } => {
                cmd.args(["set-window-option", "-t", tmux_target, name, value]);
            }
        }
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr))
        }
    }
}

pub fn parse_panes(raw: &str) -> anyhow::Result<Vec<Pane>> {
    raw.lines()
        .map(|line| {
            let mut parts = line.splitn(6, '\t');
            Ok(Pane {
                id: parts.next().unwrap_or_default().to_string(),
                window_id: parts.next().unwrap_or_default().to_string(),
                pid: parts.next().unwrap_or_default().parse().unwrap_or_default(),
                command: parts.next().unwrap_or_default().to_string(),
                path: parts.next().unwrap_or_default().to_string(),
                title: parts.next().unwrap_or_default().to_string(),
            })
        })
        .collect()
}

pub fn capture_head_tail(raw: &str, head: usize, tail: usize) -> PaneCapture {
    let lines = raw.lines().collect::<Vec<_>>();
    PaneCapture {
        head: lines
            .iter()
            .take(head)
            .copied()
            .collect::<Vec<_>>()
            .join("\n"),
        tail: lines
            .iter()
            .rev()
            .take(tail)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pane_rows() {
        let panes = parse_panes("%1\t@1\t42\tvolta-shim\t/tmp/project\t⠋ title\n").unwrap();
        assert_eq!(panes[0].id, "%1");
        assert_eq!(panes[0].pid, 42);
    }

    #[test]
    fn captures_head_and_tail() {
        let capture = capture_head_tail("a\nb\nc\nd", 2, 2);
        assert_eq!(capture.head, "a\nb");
        assert_eq!(capture.tail, "c\nd");
    }
}
