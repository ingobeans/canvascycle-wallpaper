#![allow(dead_code)]
use std::path::PathBuf;

use winapi::{
    DEFINE_GUID,
    shared::windef::{HDC, HMONITOR, HWND, LPRECT, POINT, RECT},
    um::{
        shellapi::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAA, Shell_NotifyIconA},
        shellscalingapi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness},
        winuser::{
            AppendMenuA, AppendMenuW, CreateIconFromResourceEx, CreatePopupMenu,
            EnumDisplayMonitors, FindWindowA, FindWindowExA, GWL_EXSTYLE, GWL_STYLE, GetCursorPos,
            GetWindowLongPtrW, HWND_BOTTOM, LR_DEFAULTCOLOR, LookupIconIdFromDirectoryEx, MF_POPUP,
            MF_STRING, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SendMessageA, SetForegroundWindow, SetParent, SetWindowLongPtrW, SetWindowPos,
            ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TrackPopupMenu, WM_MOUSEMOVE, WS_BORDER,
            WS_CAPTION, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
            WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
        },
    },
};

use super::utils::*;

pub fn find_miniquad_window() -> Option<HWND> {
    let result = unsafe { FindWindowA(c"MINIQUADAPP".as_ptr().cast(), std::ptr::null()) };
    if result as usize == 0 {
        return None;
    }
    Some(result)
}

pub fn set_window_scaling() {
    unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) };
}

/// Const function that pads a bytestring with zeroes to become a fixed length
const fn fixed_length_str<const N: usize, const A: usize>(text: &[u8; A]) -> [i8; N] {
    assert!(N > A);
    let mut a = [0; N];
    let ptr = &mut a as *mut _ as *mut [u8; A];
    unsafe { *ptr = *text };
    a
}

pub fn set_tray(hwnd: HWND) {
    DEFINE_GUID! {GUID_NULL, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};

    let sz_info = fixed_length_str(b"Pixedwall");
    let sz_info_title = fixed_length_str(b"Pixedwall");
    let sz_tip = fixed_length_str(b"Pixedwall");

    let mut icon = std::ptr::null_mut();
    let icon_data = include_bytes!("../assets/icon.ico");
    let offset = unsafe {
        LookupIconIdFromDirectoryEx(icon_data as *const _ as _, 1, 16, 16, LR_DEFAULTCOLOR)
    };
    if offset == 0 {
        eprintln!("Failed to load icon resource");
    } else {
        let result = unsafe {
            CreateIconFromResourceEx(
                ((icon_data as *const _ as usize) + offset as usize) as _,
                icon_data.len() as _,
                1,
                0x00030000,
                16,
                16,
                LR_DEFAULTCOLOR,
            )
        };
        if result as usize == 0 {
            eprintln!("Failed to load icon");
        } else {
            icon = result as usize as _;
        }
    }

    #[repr(C)]
    union NotifyIconDataUnion {
        u_timeout: u32,
        u_version: u32,
    }

    let u = NotifyIconDataUnion { u_version: 0 };
    let mut data = NOTIFYICONDATAA {
        cbSize: size_of::<NOTIFYICONDATAA>() as _,
        hWnd: hwnd,
        dwInfoFlags: 0,
        uCallbackMessage: WM_MOUSEMOVE,
        hIcon: icon,
        dwState: 0,
        dwStateMask: 0,
        szInfo: sz_info,
        szInfoTitle: sz_info_title,
        uID: 0,
        uFlags: NIF_TIP | NIF_ICON | NIF_MESSAGE,
        szTip: sz_tip,
        u: unsafe { std::mem::transmute(u) },
        guidItem: GUID_NULL,
        hBalloonIcon: std::ptr::null_mut(),
    };
    let result = unsafe { Shell_NotifyIconA(NIM_ADD, &mut data) };
    if result != 1 {
        eprintln!("Couldn't add tray icon");
    }
}

pub fn get_monitors() -> Vec<RECT> {
    unsafe extern "system" fn enum_callback(
        _hmon: HMONITOR,
        _hdc: HDC,
        rect: LPRECT,
        param: isize,
    ) -> i32 {
        let rects: *mut Vec<RECT> = param as _;
        unsafe {
            (*rects).push(*rect);
        }

        1
    }
    let mut rects: Vec<RECT> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(enum_callback),
            &mut rects as *mut _ as _,
        )
    };
    rects
}

pub fn show_context_menu(hwnd: HWND, items: &[(String, PathBuf)]) {
    let mut cursor_pos = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut cursor_pos) };

    const EXIT: usize = 1;
    const RESTART: usize = 2;
    const CHANGE_WALLPAPER_OFFSET: usize = 100;

    let submenu = unsafe { CreatePopupMenu() };
    for (index, (name, _)) in items.iter().enumerate() {
        let utf16: Vec<u16> = name.encode_utf16().collect();
        unsafe {
            AppendMenuW(
                submenu,
                MF_STRING,
                CHANGE_WALLPAPER_OFFSET + index,
                utf16.as_ptr() as _,
            )
        };
    }

    let menu = unsafe { CreatePopupMenu() };

    unsafe { AppendMenuA(menu, MF_STRING, EXIT, c"Exit".as_ptr()) };
    unsafe { AppendMenuA(menu, MF_STRING, RESTART, c"Restart".as_ptr()) };
    unsafe {
        AppendMenuA(
            menu,
            MF_STRING | MF_POPUP,
            submenu as usize,
            c"Change Wallpaper".as_ptr(),
        )
    };

    unsafe { SetForegroundWindow(hwnd) };

    let cmd = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY,
            cursor_pos.x,
            cursor_pos.y,
            0,
            hwnd,
            std::ptr::null(),
        )
    };
    match cmd as usize {
        EXIT => {
            println!("exit");
        }
        CHANGE_WALLPAPER_OFFSET.. => {
            let index = cmd as usize - CHANGE_WALLPAPER_OFFSET;
            let path = items[index].1.to_string_lossy();
            println!("change-wallpaper{path}");
        }
        _ => {}
    }
}

/// Holy vibecoded
///
/// Even god's mightiest soldiers have their limits. (mine are debugging undocumented windows APIs for over 3 hours and its already 2 am)
pub fn style_window_as_wallpaper(hwnd: HWND, wallpaper: &WallpaperOptions) {
    let monitors = get_monitors();
    let result = unsafe {
        SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        )
    };
    if result == 0 {
        eprintln!("Failed setting window as always on bottom");
    }
    let progman_hwnd = unsafe { FindWindowA(c"Progman".as_ptr().cast(), std::ptr::null()) };
    if progman_hwnd as usize == 0 {
        eprintln!("Couldn't find Program Manager window");
    }

    // https://dynamicwallpaper.readthedocs.io/en/docs/dev/make-wallpaper.html
    let result = unsafe { SendMessageA(progman_hwnd, 0x052C, 0xD as _, 0 as _) };
    if result != 0 {
        eprintln!("Failed sending message 1: {result}")
    }
    let result = unsafe { SendMessageA(progman_hwnd, 0x052C, 0xD as _, 1 as _) };
    if result != 0 {
        eprintln!("Failed sending message 2: {result}")
    }

    let defview = unsafe {
        FindWindowExA(
            progman_hwnd,
            std::ptr::null_mut(),
            c"SHELLDLL_DefView".as_ptr().cast(),
            std::ptr::null(),
        )
    };

    let mut style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 };

    style |= WS_CHILD;
    style &= !(WS_POPUP
        | WS_BORDER
        | WS_CAPTION
        | WS_THICKFRAME
        | WS_MINIMIZEBOX
        | WS_MAXIMIZEBOX
        | WS_SYSMENU
        | WS_VISIBLE);

    let result = unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize) };
    if result == 0 {
        eprintln!("Failed setting window style");
    }

    let mut exstyle = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 };

    exstyle |= WS_EX_LAYERED | WS_EX_NOACTIVATE;

    let result = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, exstyle as isize) };
    if result == 0 {
        eprintln!("Failed setting window style");
    }

    let result = unsafe { SetParent(hwnd, progman_hwnd) };
    if result as usize == 0 {
        eprintln!("Failed to set parent.");
    }

    let rect = wallpaper;

    let mut smallest_left = monitors[0].left;
    let mut smallest_top = monitors[0].top;
    for rect in &monitors {
        if rect.left < smallest_left {
            smallest_left = rect.left;
        }
        if rect.top < smallest_top {
            smallest_top = rect.top;
        }
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    // need to be offset, since the client area of the desktop has 0,0 as the top left position,
    // while monitor rects i believe are relative to the primary monitor,
    // so a monitor to the left of the primary would for instance have the position -1920, 0,
    // which needs to be 0,0
    let left = rect.left - smallest_left;
    let top = rect.top - smallest_top;

    let result = unsafe {
        SetWindowPos(
            hwnd,
            defview,
            left,
            top,
            width,
            height,
            SWP_FRAMECHANGED | SWP_NOACTIVATE,
        ) == 0
    };
    if result {
        eprintln!("SetWindowPos failed");
        return;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
    }
}
