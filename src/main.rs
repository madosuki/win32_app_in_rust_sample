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

/*
fn convert_to_pwstr(src: &str) -> PWSTR {
    PWSTR(convert_u8_to_u16(src).as_mut_ptr())
}
*/

/*
fn convert_to_pcwstr(src: &str) -> PCWSTR {
    PCWSTR(convert_u8_to_u16(src).as_mut_ptr())
}
*/

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

#[derive(Default)]
struct MainWindow {
    window_name: PCWSTR,
    class_name: PCWSTR,
}

impl MainWindow {
    pub fn init(mut self) {
        unsafe {
            let instance: HINSTANCE = GetModuleHandleW(None).expect("").into();
    
            // let class_name_str = "class_name";
            self.class_name = windows::core::w!("class_name");
    
            let mut wnd = WNDCLASSW::default();
            wnd.lpfnWndProc = Some(wnd_proc);
            wnd.hInstance = instance;
            wnd.lpszClassName = self.class_name;
    
            let result = RegisterClassW(&wnd);
            if result == 0 {
                println!("{:?}", GetLastError());
                return;
            }
    
            self.window_name = windows::core::w!("window name");
            let hwnd: HWND = CreateWindowExW(
                WS_EX_OVERLAPPEDWINDOW,
                self.class_name,
                self.window_name,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                0,
                0,
                1024,
                768,
                None,
                None,
                None,
                None
            ).expect("failed create window");
    
            let show_window_result = ShowWindow(hwnd, SW_SHOW);
            if !show_window_result.as_bool() {
                println!("{:?}", GetLastError());
                return;
            }
    
            let mut msg = MSG::default();
            loop {
                let get_messeage_result = GetMessageW(&mut msg, Some(HWND::default()), 0, 0);
                if !get_messeage_result.as_bool() {
                    return;
                }
    
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        } 
    }
}

fn main() {
    let main_window = MainWindow::default();
    main_window.init();
}

