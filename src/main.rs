use clap::{Parser, Subcommand};
use tmux_reactor_ai::{
    ai::RigAiProvider,
    config::ReactorConfig,
    logging,
    process::{PsProcessInspector, label_for_process_tree},
    reactor,
    tmux::CliTmux,
};

#[derive(Debug, Parser)]
#[command(version, about = "AI-powered tmux event reactor")]
struct Cli {
    #[arg(long, env = "TMUX_REACTOR_AI_CONFIG")]
    config: Option<std::path::PathBuf>,

    #[arg(long, env = "TMUX_REACTOR_AI_LOG", default_value = "info")]
    log: String,

    #[arg(long, env = "TMUX_REACTOR_AI_LOG_JSON")]
    json_logs: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run enabled rules once against a tmux target.
    RunOnce {
        #[arg(long, default_value = "#{window_id}")]
        target: String,
    },
    /// Print the deterministic label for a pane/process tuple.
    Label {
        #[arg(long)]
        pane_pid: u32,
        #[arg(long)]
        pane_command: String,
        #[arg(long)]
        pane_path: String,
    },
    /// Print an example config.
    InitConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    logging::init(cli.json_logs, &cli.log);

    match cli.command {
        Command::RunOnce { target } => {
            let config = match cli.config {
                Some(path) => ReactorConfig::from_path(path)?,
                None => ReactorConfig::default(),
            };
            reactor::run_once(
                &config,
                &CliTmux,
                &PsProcessInspector,
                &RigAiProvider,
                &target,
            )
            .await?;
        }
        Command::Label {
            pane_pid,
            pane_command,
            pane_path,
        } => {
            println!(
                "{}",
                label_for_process_tree(pane_pid, &pane_command, &pane_path, &PsProcessInspector)
            );
        }
        Command::InitConfig => {
            println!("{}", serde_yaml::to_string(&ReactorConfig::default())?);
        }
    }

    Ok(())
}
