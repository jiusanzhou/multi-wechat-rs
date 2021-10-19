#![windows_subsystem = "windows"]

mod winapi;
mod process;
mod system;
mod utils;

fn main() {

    println!("Hello, WeChat & Rust!");

    // should we??? elevate privileges
    let _ = utils::evelate_privileges();

    // get the wechat process
    match process::Process::find_first_by_name("WeChat.exe") {
        None => {},
        Some(p) => {
            // get handles of those process
            let mutants = system::get_system_handles(p.pid()).unwrap()
                .iter()
                // match which one is mutex handle
                .filter(|x| x.type_name == "Mutant" && x.name.contains("_WeChat_App_Instance_Identity_Mutex_Name"))
                .cloned()
                .collect::<Vec<_>>();
            
            for m in mutants {
                // and close the handle
                println!("clean mutant: {}", m.name);
                let _ = m.close_handle();
            }
        }
    }

    // get wechat start exe location
    let wechat_key = "Software\\Tencent\\WeChat";
    match utils::get_install_path(wechat_key) {
        Some(p) => {
            // start wehat process
            // WeChat.exe
            println!("start wechat process => {}", p);
            let exe = format!("{}\\WeChat.exe", p);
            if utils::create_process(exe.as_str(), "").is_err() {
                println!("Error: {}", utils::get_last_error());
             }
        },
        None => {
            println!("get wechat install failed, you can still open multi wechat");
            utils::show_message_box("已关闭多开限制", "无法自动启动微信，仍可手动打开微信。");
        }
    }
}
