use winapi::{
    shared::windef::HWND,
    um::winuser::{
        FindWindowA, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SetWindowLongPtrW, SetWindowPos, WS_BORDER, WS_CAPTION, WS_EX_NOACTIVATE, WS_MINIMIZEBOX,
        WS_THICKFRAME,
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
            (default_style & !WS_BORDER & !WS_THICKFRAME & !WS_CAPTION & !WS_MINIMIZEBOX) as _,
        )
    };
    if result == 0 {
        eprintln!("Failed setting window style");
    }
    let result = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (262160 | &WS_EX_NOACTIVATE) as _) };
    println!("ex: {result}");
    if result == 0 {
        eprintln!("Failed setting window style");
    }
    let result = unsafe { SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 1920, 1080, SWP_NOACTIVATE) };
    if result == 0 {
        eprintln!("Failed setting window as always on bottom");
    }
}
