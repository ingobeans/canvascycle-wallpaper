use winapi::{
    DEFINE_GUID,
    shared::windef::{HDC, HMONITOR, HWND, LPRECT, RECT},
    um::{
        shellapi::{
            NIF_INFO, NIF_TIP, NIM_ADD, NOTIFYICONDATAA, NOTIFYICONDATAA_u, Shell_NotifyIconA,
        },
        shellscalingapi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness},
        winuser::{
            EnumDisplayMonitors, FindWindowA, FindWindowExA, GWL_EXSTYLE, GWL_STYLE,
            GetWindowLongPtrW, HWND_BOTTOM, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
            SWP_NOSIZE, SendMessageA, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow,
            WS_BORDER, WS_CAPTION, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_MAXIMIZEBOX,
            WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
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
    let mut data = NOTIFYICONDATAA {
        cbSize: size_of::<NOTIFYICONDATAA>() as _,
        hWnd: hwnd,
        dwInfoFlags: 0,
        uCallbackMessage: 0,
        hIcon: std::ptr::null_mut(),
        dwState: 0,
        dwStateMask: 0,
        szInfo: sz_info,
        szInfoTitle: sz_info_title,
        uID: 0,
        uFlags: NIF_INFO | NIF_TIP,
        szTip: sz_tip,
        u: unsafe { *(&[0] as *const _ as *const NOTIFYICONDATAA_u) },
        guidItem: GUID_NULL,
        hBalloonIcon: std::ptr::null_mut(),
    };
    let result = unsafe { Shell_NotifyIconA(NIM_ADD, &mut data) };
    if result != 1 {
        eprintln!("Couldn't add tray icon");
    }
}

fn get_monitors() -> Vec<RECT> {
    unsafe extern "system" fn enum_callback(
        hmon: HMONITOR,
        _hdc: HDC,
        rect: LPRECT,
        param: isize,
    ) -> i32 {
        unsafe {
            println!(
                "monitor found: hmon:{hmon:?}, rect:({},{}),({},{})",
                (*rect).left,
                (*rect).right,
                (*rect).top,
                (*rect).bottom
            );
        }
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

/// Holy vibecoded
///
/// Even god's mightiest soldiers have their limits. (mine are debugging undocumented windows APIs for over 3 hours and its already 2 am)
pub fn style_window_as_wallpaper(hwnd: HWND) {
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

    let rect = monitors[0];

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
    println!("{width}x{height}");
    if result {
        eprintln!("SetWindowPos failed");
        return;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
    }
}
