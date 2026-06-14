use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub command: String,
    pub args: String,
}

pub trait ProcessInspector {
    fn children(&self, pid: u32) -> anyhow::Result<Vec<ProcessInfo>>;
}

#[derive(Debug, Default)]
pub struct PsProcessInspector;

impl ProcessInspector for PsProcessInspector {
    fn children(&self, pid: u32) -> anyhow::Result<Vec<ProcessInfo>> {
        let child_pids = Command::new("pgrep")
            .args(["-P", &pid.to_string()])
            .output()?;

        if !child_pids.status.success() || child_pids.stdout.is_empty() {
            return Ok(vec![]);
        }

        let pids = String::from_utf8_lossy(&child_pids.stdout)
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        if pids.is_empty() {
            return Ok(vec![]);
        }

        let output = Command::new("ps")
            .args(["-o", "pid=,ppid=,comm=,args=", "-p", &pids])
            .output()?;

        Ok(parse_ps_rows(&String::from_utf8_lossy(&output.stdout)))
    }
}

pub fn label_for_process_tree(
    pane_pid: u32,
    pane_command: &str,
    pane_path: &str,
    inspector: &dyn ProcessInspector,
) -> String {
    if pane_command != "volta-shim" {
        return pane_command.to_string();
    }

    let Ok(children) = inspector.children(pane_pid) else {
        return basename(pane_path);
    };

    children
        .iter()
        .find_map(|child| label_from_args(&child.args))
        .unwrap_or_else(|| basename(pane_path))
}

pub fn label_from_args(args: &str) -> Option<String> {
    let lower = args.to_ascii_lowercase();
    for known in ["codex", "claude", "pi", "opencode", "moshi"] {
        if lower
            .split(|c: char| c.is_whitespace() || c == '/' || c == '\\')
            .any(|part| part == known || part == format!("{known}.js"))
        {
            return Some(known.to_string());
        }
    }

    if let Some(bin) = args.split_whitespace().last()
        && let Some(name) = bin.rsplit('/').next()
        && !name.is_empty()
        && name != "node"
    {
        return Some(name.trim_end_matches(".js").to_string());
    }

    None
}

fn basename(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn parse_ps_rows(raw: &str) -> Vec<ProcessInfo> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pid = parts.next()?.parse().ok()?;
            let ppid = parts.next()?.parse().ok()?;
            let command = parts.next()?.to_string();
            let args = parts.collect::<Vec<_>>().join(" ");
            Some(ProcessInfo {
                pid,
                ppid,
                command,
                args,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeInspector(Vec<ProcessInfo>);

    impl ProcessInspector for FakeInspector {
        fn children(&self, _pid: u32) -> anyhow::Result<Vec<ProcessInfo>> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn unwraps_codex_from_node_args() {
        let inspector = FakeInspector(vec![ProcessInfo {
            pid: 2,
            ppid: 1,
            command: "node".to_string(),
            args: "node /x/npm-openai-codex/latest/bin/codex".to_string(),
        }]);
        assert_eq!(
            label_for_process_tree(1, "volta-shim", "/Users/ramiro/dev/toolbox", &inspector),
            "codex"
        );
    }

    #[test]
    fn non_shim_is_unchanged() {
        let inspector = FakeInspector(vec![]);
        assert_eq!(label_for_process_tree(1, "nu", "/tmp/x", &inspector), "nu");
    }
}
