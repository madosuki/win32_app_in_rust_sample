use windows::{
    core::{PWSTR, PCWSTR, PCSTR, PSTR},
    Win32::Foundation::{HWND, HINSTANCE, LPARAM, WPARAM, LRESULT, GetLastError},
    Win32::Graphics::Gdi::{UpdateWindow, HBRUSH, HDC},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::{
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
        WNDCLASSW,
        HICON,
        HCURSOR,
        RegisterClassW,
        PostQuitMessage,
        DefWindowProcW,
        WM_DESTROY,
        WM_PAINT,
        MSG,
        TranslateMessage,
        DispatchMessageW,
        GetMessageW,
        SHOW_WINDOW_CMD,
        WS_EX_OVERLAPPEDWINDOW
    }
};

fn convert_u8_to_u16(src: &str) -> Vec<u16> {
    src.encode_utf16().chain(Some(0)).collect()
}

fn convert_to_pwstr(src: &str) -> PWSTR {
    PWSTR(convert_u8_to_u16(src).as_mut_ptr())
}

fn convert_to_pcwstr(src: &str) -> PCWSTR {
    PCWSTR(convert_u8_to_u16(src).as_ptr())
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    
    match msg {
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    
    LRESULT(0)
}

fn main() {
    let instance = unsafe {
        match GetModuleHandleW(None) {
            Ok(v) => v,
            Err(_) => panic!("failed instance")
        }
    };
    let sz_window_class = convert_to_pcwstr("class_name");

    let mut wnd = WNDCLASSW::default();
    wnd.lpfnWndProc = Some(wnd_proc);
    wnd.hInstance = instance;
    wnd.lpszClassName = sz_window_class;
    
    unsafe {
        let result = RegisterClassW(&wnd);
        if result == 0 {
            println!("{:?}", GetLastError());
            return
        }
    }

    let hwnd: HWND = unsafe {
        let window_name = convert_to_pcwstr("Win32 app written in Rust");
        CreateWindowExW(
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
            std::ptr::null_mut()
        )
    };

    unsafe {
        let show_window_result = ShowWindow(hwnd, SW_SHOW);
        if !show_window_result.as_bool() {
            println!("{:?}", GetLastError());
            return
        }

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