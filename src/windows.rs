use winapi::{
    shared::windef::HWND,
    um::{
        shellscalingapi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness},
        winuser::{
            FindWindowA, FindWindowExA, GWL_EXSTYLE, GWL_STYLE, GetClientRect, GetWindowLongPtrW,
            HWND_BOTTOM, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SendMessageA, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_BORDER,
            WS_CAPTION, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
            WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

pub fn find_miniquad_window() -> Option<HWND> {
    let result = unsafe { FindWindowA(c"MINIQUADAPP".as_ptr().cast(), std::ptr::null()) };
    if result as usize == 0 {
        return None;
    }
    Some(result)
}

pub fn set_window_scaling() {
    let result = unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) };
    if result != 0 {
        eprintln!("Failed setting DPI awareness: {result}");
    }
}

/// Holy vibecoded
///
/// Even god's mightiest soldiers have their limits. (mine are debugging undocumented windows APIs for over 3 hours and its already 2 am)
pub fn style_window_as_wallpaper(hwnd: HWND) {
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
        | WS_SYSMENU);

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

    let mut rect = unsafe { std::mem::zeroed() };

    if unsafe { GetClientRect(progman_hwnd, &mut rect) == 0 } {
        eprintln!("GetClientRect failed");
        return;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    let result = unsafe {
        SetWindowPos(
            hwnd,
            defview,
            0,
            0,
            width,
            height,
            SWP_FRAMECHANGED | SWP_NOACTIVATE,
        ) == 0
    };
    println!("{width}x{height}");
    if result {
        eprintln!("SetWindowPos failed");
        return;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
    }
}
