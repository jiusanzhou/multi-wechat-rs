#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod winapi;
#[cfg(target_os = "windows")]
mod process;
#[cfg(target_os = "windows")]
mod system;
#[cfg(target_os = "windows")]
mod utils;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
fn main() {
    println!("Hello, WeChat & Rust!");

    let _ = utils::evelate_privileges();

    match process::Process::find_first_by_name("WeChat.exe") {
        None => {}
        Some(p) => {
            let mutants = system::get_system_handles(p.pid())
                .unwrap()
                .iter()
                .filter(|x| {
                    x.type_name == "Mutant"
                        && x.name
                            .contains("_WeChat_App_Instance_Identity_Mutex_Name")
                })
                .cloned()
                .collect::<Vec<_>>();

            for m in mutants {
                println!("clean mutant: {}", m.name);
                let _ = m.close_handle();
            }
        }
    }

    let wechat_key = "Software\\Tencent\\WeChat";
    match utils::get_install_path(wechat_key) {
        Some(p) => {
            println!("start wechat process => {}", p);
            let exe = format!("{}\\WeChat.exe", p);
            if utils::create_process(exe.as_str(), "").is_err() {
                println!("Error: {}", utils::get_last_error());
            }
        }
        None => {
            println!("get wechat install failed, you can still open multi wechat");
            utils::show_message_box("已关闭多开限制", "无法自动启动微信，仍可手动打开微信。");
        }
    }
}

#[cfg(target_os = "macos")]
fn main() {
    use clap::{Parser, Subcommand};
    use macos::patcher::{self, WeChatApp};
    use std::path::PathBuf;

    #[derive(Parser)]
    #[command(name = "multi-wechat-rs", about = "WeChat multi-instance & anti-revoke tool")]
    struct Cli {
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

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn main() {
    eprintln!("This tool only supports Windows and macOS.");
    std::process::exit(1);
}
