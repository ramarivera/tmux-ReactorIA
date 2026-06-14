use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn init_config_outputs_yaml() {
    Command::cargo_bin("tmux-reactor-ai")
        .unwrap()
        .arg("init-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("ai-window-title"));
}

#[test]
fn label_non_shim_is_command() {
    Command::cargo_bin("tmux-reactor-ai")
        .unwrap()
        .args([
            "label",
            "--pane-pid",
            "1",
            "--pane-command",
            "nu",
            "--pane-path",
            "/tmp/toolbox",
        ])
        .assert()
        .success()
        .stdout("nu\n");
}
