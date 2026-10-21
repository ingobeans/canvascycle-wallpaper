#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
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

fn run() -> bool {
    let mut args = std::env::args().skip(1);
    let arg = args.next();

    let exe_path = std::env::current_exe().unwrap();
    let parent = exe_path.parent().unwrap();
    let renderer = parent.join("renderer.exe");

    let path = if let Some(arg) = arg {
        let mut path = PathBuf::from(arg);

        if !path.exists() && parent.join(&path).exists() {
            path = parent.join(path);
        }
        path
    } else {
        let mut data_path = PathBuf::from("data/images/");
        if !data_path.exists() && parent.join(&data_path).exists() {
            data_path = parent.join(data_path);
        }
        if !data_path.exists() {
            eprintln!("Error: no data/images/ directory found in exe dir!");
            return false;
        }
        let a = 'item: {
            for item in std::fs::read_dir(data_path).unwrap().flatten() {
                if item.file_type().unwrap().is_file() {
                    break 'item item.path();
                }
            }
            eprintln!("Error: data/images/ directory is empty!");
            return false;
        };
        a
    };

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

        #[cfg(not(debug_assertions))]
        {
            use std::os::windows::process::CommandExt;

            use winapi::um::winbase::CREATE_NO_WINDOW;

            handle = handle.creation_flags(CREATE_NO_WINDOW);
        }

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
    let mut last_message = Vec::new();
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
            for stdin in &mut stdins {
                stdin.write_all(&current_message).unwrap();
                stdin.write_all(b"\n").unwrap();
            }

            last_message = current_message.clone();
            current_message.clear();
        } else {
            current_message.push(buf[0]);
        }
    }

    for mut handle in seconday_handles {
        let _ = handle.wait();
    }
    last_message == b"restart"
}

fn main() {
    #[cfg(not(debug_assertions))]
    unsafe {
        use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    loop {
        let restart = run();
        if !restart {
            break;
        }
        println!("Restarting...");
    }
}
