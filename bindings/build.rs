fn main() {
    windows::build!(
        Windows::Win32::UI::WindowsAndMessaging::{
            HMENU, 
            CreateWindowExW, 
            ShowWindow, 
            WINDOW_EX_STYLE, 
            MessageBoxA, 
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
         },
        Windows::Win32::Graphics::Gdi::{ UpdateWindow, HBRUSH, HDC },
        Windows::Win32::Foundation::{ HWND, HINSTANCE, PWSTR, LPARAM, WPARAM, LRESULT }
    );
}