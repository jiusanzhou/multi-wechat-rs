use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::patcher::{self, WeChatApp};

#[derive(Parser)]
#[command(name = "multi-wechat-rs", about = "WeChat multi-instance & anti-revoke tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to WeChat.app
    #[arg(short, long, default_value = "/Applications/WeChat.app")]
    app: PathBuf,

    /// Path to external config.json (uses embedded config if not specified)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Patch WeChat binary (multi-instance + anti-revoke)
    Patch,
    /// List current and supported WeChat versions
    Versions,
}

pub fn run() {
    let cli = Cli::parse();
    let app = WeChatApp::new(&cli.app);
    let config_path = cli.config.as_deref();

    match cli.command.unwrap_or(Commands::Patch) {
        Commands::Patch => {
            if let Err(e) = patcher::patch(&app, config_path) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Versions => match patcher::list_versions(&app, config_path) {
            Ok((current, supported)) => {
                println!("------ Current version ------");
                println!("{}", current.as_deref().unwrap_or("unknown"));
                println!("------ Supported versions ------");
                for v in &supported {
                    println!("{}", v);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
