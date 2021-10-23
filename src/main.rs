use bindings::Windows::Win32::UI::WindowsAndMessaging::{
    HMENU, 
    CreateWindowExW, 
    ShowWindow, 
    WINDOW_EX_STYLE, 
    WS_VISIBLE, 
    WS_OVERLAPPEDWINDOW, 
    SW_SHOW, 
    MessageBoxA, 
    MB_OK,
    WNDCLASS_STYLES,
    WNDCLASSEXW,
    HICON,
    HCURSOR,
    RegisterClassExW,
    PostQuitMessage,
    DefWindowProcW,
    WM_DESTROY,
    WM_PAINT,
    MSG,
    TranslateMessage,
    DispatchMessageW,
    GetMessageW,
    SHOW_WINDOW_CMD
};
use bindings::Windows::Win32::Foundation::{HWND, HINSTANCE, PWSTR, LPARAM, WPARAM, LRESULT};
use bindings::Windows::Win32::Graphics::Gdi::{UpdateWindow, HBRUSH, HDC};

fn convert_u8_to_u16(src: &str) -> Vec<u16> {
    src.encode_utf16().chain(Some(0)).collect()
}

fn convert_to_pwstr(src: &str) -> PWSTR {
    PWSTR(convert_u8_to_u16(src).as_mut_ptr())
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    
    match msg {
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    
    LRESULT(0)
}

fn main() {
    let class_name = convert_to_pwstr("test_window");
    let menu_name = convert_to_pwstr("menu name");
    let window_name = convert_to_pwstr("Win32 app written in Rust");
    
    let wnd = WNDCLASSEXW {
             cbSize: 0,
             style: WNDCLASS_STYLES(0),
             lpfnWndProc: Some(wnd_proc),
             cbClsExtra: 0,
             cbWndExtra: 0,
             hInstance: HINSTANCE::default(),
             hIcon: HICON(0),
             hCursor: HCURSOR(0),
             hbrBackground: HBRUSH(0),
             lpszMenuName: menu_name,
             lpszClassName: class_name,
             hIconSm: HICON(0)
    };
    
    unsafe {
        RegisterClassExW(&wnd);
    }

    let hwnd: HWND = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0), 
            class_name,
            window_name,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            0,
            0,
            1024,
            768,
            HWND::default(),
            HMENU::default(),
            HINSTANCE::default(),
            std::ptr::null_mut()
        )
    };
    unsafe {
        ShowWindow(hwnd, SHOW_WINDOW_CMD(5));
        /*
        let update_window_result = UpdateWindow(hwnd);
        if !update_window_result.as_bool() {
            return
        }
        */

        let mut msg = MSG::default();
        loop {
            let get_messeage_result = GetMessageW(&mut msg, HWND(0), 0, 0);
            if !get_messeage_result.as_bool() {
                return
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
}