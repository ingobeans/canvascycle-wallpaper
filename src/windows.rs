use winapi::{
    shared::{minwindef::WPARAM, windef::HWND},
    um::winuser::{
        EnumWindows, FindWindowA, FindWindowExA, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SW_SHOW,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SendMessageA, SetParent,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, WNDENUMPROC, WS_BORDER, WS_CAPTION,
        WS_EX_APPWINDOW, WS_EX_CLIENTEDGE, WS_EX_COMPOSITED, WS_EX_DLGMODALFRAME, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_STATICEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_MAXIMIZEBOX,
        WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
    },
};

pub fn find_miniquad_window() -> Option<HWND> {
    let classname = "MINIQUADAPP\x00";
    let result = unsafe { FindWindowA(classname as *const str as _, std::ptr::null()) };
    if result as usize == 0 {
        return None;
    }
    Some(result)
}

pub fn style_window_as_wallpaper(hwnd: HWND) {
    // the result of SetWindowLongPtrW is the previous style,
    // which is how i got this value.
    let default_style = 382664704;
    let result = unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            (default_style
                & !WS_BORDER
                & !WS_THICKFRAME
                & !WS_CAPTION
                & !WS_MINIMIZEBOX
                & !WS_MAXIMIZEBOX
                & !WS_SYSMENU) as _,
        )
    };
    if result == 0 {
        eprintln!("Failed setting window style");
    }
    let result = unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            (262160
                | &WS_EX_NOACTIVATE
                    & !WS_EX_DLGMODALFRAME
                    & !WS_EX_COMPOSITED
                    & !WS_EX_WINDOWEDGE
                    & !WS_EX_CLIENTEDGE
                    & !WS_EX_LAYERED
                    & !WS_EX_STATICEDGE
                    & !WS_EX_TOOLWINDOW
                    & !WS_EX_APPWINDOW) as _,
        )
    };
    if result == 0 {
        eprintln!("Failed setting window style");
    }
    let result = unsafe {
        SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0,
            0,
            1920,
            1080,
            SWP_FRAMECHANGED | SWP_NOACTIVATE,
        )
    };
    if result == 0 {
        eprintln!("Failed setting window as always on bottom");
    }

    let progman_hwnd = unsafe { FindWindowA("Progman\x00" as *const str as _, std::ptr::null()) };
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

    let mut worker_w_hwnd: HWND = 0 as _;

    unsafe extern "system" fn enum_func(hwnd: HWND, pointer: isize) -> i32 {
        let def_view = unsafe {
            FindWindowExA(
                hwnd,
                std::ptr::null_mut(),
                "SHELLDLL_DefView\0" as *const str as _,
                std::ptr::null_mut(),
            )
        };

        if !def_view.is_null() {
            let worker = unsafe {
                FindWindowExA(
                    std::ptr::null_mut(),
                    hwnd,
                    "WorkerW\0" as *const str as _,
                    std::ptr::null_mut(),
                )
            };

            if !worker.is_null() {
                unsafe { *(pointer as *mut HWND) = worker };
                return 0;
            }
        }

        1
    }
    let result = unsafe {
        EnumWindows(
            WNDENUMPROC::Some(enum_func),
            &mut worker_w_hwnd as *mut _ as isize,
        )
    };
    if result == 0 {
        eprintln!("Failed to enum windows.");
    }
    println!("WorkerW hwnd: {worker_w_hwnd:?}");

    let result = unsafe { SetParent(hwnd, worker_w_hwnd) };
    if result as usize == 0 {
        eprintln!("Failed to set parent.");
    }
    unsafe { ShowWindow(hwnd, SW_SHOW) };
}
