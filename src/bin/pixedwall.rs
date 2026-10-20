#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
use std::{path::PathBuf, time::Duration};

#[path = "../utils.rs"]
pub mod utils;

#[path = "../windows.rs"]
mod windows;

fn main() {
    let mut args = std::env::args().skip(1);
    let arg = args.next();
    let mut path = if let Some(arg) = arg {
        PathBuf::from(arg)
    } else {
        PathBuf::from("data/images/TESTRAMP.LBM.json")
    };

    let exe_path = std::env::current_exe().unwrap();
    let parent = exe_path.parent().unwrap();
    let renderer = parent.join("renderer.exe");
    if !path.exists() && parent.join(&path).exists() {
        path = parent.join(path);
    }

    let monitors = windows::get_monitors();
    println!("{} monitors found.", monitors.len());

    let mut handles = Vec::new();
    for i in 0..monitors.len() {
        let handle = std::process::Command::new(&renderer)
            .arg(&path)
            .arg("-w")
            .arg(i.to_string())
            .spawn()
            .unwrap();
        handles.push(handle);
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("Wallpapers loaded!");
    for mut handle in handles {
        handle.wait().unwrap();
    }
}
