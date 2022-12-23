use std::os::windows::prelude::OsStrExt;

use windows::{
    core::{PCSTR, PCWSTR, PSTR, PWSTR},
    Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Win32::Graphics::Gdi::{UpdateWindow, HBRUSH, HDC},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadStringW, MessageBoxA,
        PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, HCURSOR, HICON, HMENU,
        MB_OK, MSG, SHOW_WINDOW_CMD, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WM_PAINT, WNDCLASSW,
        WNDCLASS_STYLES, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};

fn convert_u8_to_u16(src: &str) -> Vec<u16> {
    // ref from https://teratail.com/questions/lcimq2rocy2hyu
    let mut a: Vec<u16> = std::ffi::OsString::from(src).encode_wide().collect();
    a.push(0);
    a
}

fn convert_to_pwstr(src: &str) -> PWSTR {
    PWSTR(convert_u8_to_u16(src).as_mut_ptr())
}

fn convert_to_pcwstr(src: &str) -> PCWSTR {
    PCWSTR(convert_u8_to_u16(src).as_mut_ptr())
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }

    LRESULT(0)
}

fn main() {
    unsafe {
        let instance = match GetModuleHandleW(None) {
            Ok(v) => v,
            Err(_) => panic!("failed instance"),
        };

        let sz_window_class = convert_to_pcwstr("class_name");

        let mut wnd = WNDCLASSW::default();
        wnd.lpfnWndProc = Some(wnd_proc);
        wnd.hInstance = instance;
        wnd.lpszClassName = sz_window_class;

        let result = RegisterClassW(&wnd);
        if result == 0 {
            println!("{:?}", GetLastError());
            return;
        }

        let s = "window name";
        let window_name = convert_to_pcwstr(s);
        let llparam = Some(std::ptr::null());
        let hwnd: HWND = CreateWindowExW(
            WS_EX_OVERLAPPEDWINDOW,
            sz_window_class,
            window_name,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            0,
            0,
            1024,
            768,
            HWND::default(),
            HMENU::default(),
            instance,
            llparam,
        );

        let show_window_result = ShowWindow(hwnd, SW_SHOW);
        if !show_window_result.as_bool() {
            println!("{:?}", GetLastError());
            return;
        }

        let mut msg = MSG::default();
        loop {
            let get_messeage_result = GetMessageW(&mut msg, HWND(0), 0, 0);
            if !get_messeage_result.as_bool() {
                return;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
