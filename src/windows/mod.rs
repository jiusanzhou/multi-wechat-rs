pub mod winapi;
pub mod process;
pub mod system;
pub mod utils;

pub fn run() {
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
