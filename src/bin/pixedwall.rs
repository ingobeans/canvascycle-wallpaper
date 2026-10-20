#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::Stdio,
};

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

    #[allow(invalid_value)]
    // first window spawned is the primary window.
    // handles tray and messaging.
    // it can send messages by writing to stdout,
    // and data can be sent to it through stdin.
    let mut primary = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

    let mut stdins = Vec::new();

    let mut seconday_handles = Vec::new();
    for i in 0..monitors.len() {
        let mut command = std::process::Command::new(&renderer);
        let mut handle = command
            .arg(&path)
            .arg("-w")
            .arg(i.to_string())
            .stdin(Stdio::piped());

        if i == 0 {
            handle = handle.arg("--tray").stdout(Stdio::piped());
        }

        let mut handle = handle.spawn().unwrap();

        let stdin = handle.stdin.take().unwrap();
        stdins.push(stdin);
        if i == 0 {
            primary = handle;
        } else {
            seconday_handles.push(handle);
        }
    }
    println!("Wallpapers loaded!");

    let mut primary_stdout = primary.stdout.take().unwrap();
    let mut current_message = Vec::new();
    let mut buf = [0];
    loop {
        if primary_stdout.read_exact(&mut buf).is_err() {
            break;
        }
        if buf[0] == b'\n' {
            // pop current_message

            println!(
                "received message from primary: {:?}",
                String::from_utf8_lossy(&current_message)
            );
            match &current_message[..] {
                b"exit" => {
                    println!("Exit received!");
                    for stdin in &mut stdins {
                        stdin.write_all(b"exit\n").unwrap();
                    }
                }
                _ => {
                    eprintln!("Unknown message.");
                }
            }

            current_message.clear();
        } else {
            current_message.push(buf[0]);
        }
    }

    for mut handle in seconday_handles {
        let _ = handle.wait();
    }
}
