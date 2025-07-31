use std::cell::RefCell;
use std::sync::{LazyLock, Mutex};

use std::{os::windows::ffi::OsStrExt};

use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::{self, Gdi::{BeginPaint, BitBlt, CreateCompatibleDC, SelectObject, DT_WORDBREAK, HBITMAP, HGDIOBJ, SRCCOPY}, GdiPlus::{GdipCreateFromHDC, GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, GpGraphics, GpImage, Image}}, 
    System::LibraryLoader::{GetModuleHandleA, GetModuleHandleExW}, 
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetClientRect, GetMessageW, PostQuitMessage, RegisterClassW, ShowWindow, CW_USEDEFAULT, GDI_IMAGE_TYPE, IMAGE_BITMAP, LR_CREATEDIBSECTION, LR_LOADFROMFILE, SW_SHOW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSA, WNDCLASSW, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_VISIBLE
    }};
use windows::core::w;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, GetWindowLongPtrW, SetWindowLongPtrW, GWLP_USERDATA};

fn convert_u8_to_u16(src: &str) -> Vec<u16> {
    // ref from https://teratail.com/questions/lcimq2rocy2hyu
    let a: Vec<u16> = std::ffi::OsString::from(src).encode_wide().collect();
    a
}

struct ImageContainer {
    img_ptr: *mut GpImage,
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let img_path = w!("C:\\Users\\user\\Pictures\\example.bmp");
                let mut img = windows::Win32::Graphics::GdiPlus::GpImage::default();
                let mut img_ptr: *mut GpImage = &mut img;
                let s = windows::Win32::Graphics::GdiPlus::GdipLoadImageFromFile(img_path, &mut img_ptr);
                println!("GdipLoadImageFromFile: {}", s.0);
                if s.0 == 0 {
                    let img_container = Box::new(ImageContainer { img_ptr: img_ptr });
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(img_container) as isize);
                }
            },
            WM_PAINT => {
                let mut rect = windows::Win32::Foundation::RECT::default();
                rect.top = 0;
                rect.bottom = 100;
                rect.left = 50;
                rect.right = 200;
                // let _ = GetClientRect(hwnd, &mut rect);

                let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut paint);
                let mut text = convert_u8_to_u16("猫 wake up!\r\nHero!");
                windows::Win32::Graphics::Gdi::DrawTextW(
                    hdc, 
                    &mut text,
                    &mut rect,
                    DT_WORDBREAK);
                let mut graphics = windows::Win32::Graphics::GdiPlus::GpGraphics::default();
                let mut graphics_ptr: *mut GpGraphics = &mut graphics;
                let graphics_ptr_ptr: *mut *mut GpGraphics = &mut graphics_ptr;
                let graphics_status = GdipCreateFromHDC(hdc, graphics_ptr_ptr);
                if graphics_status.0 != 0 {
                    println!("gprahics_status: {}", graphics_status.0);
                }

                let img_container_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ImageContainer;
                if !img_container_data.is_null() {
                    let img_container_ptr = &*img_container_data;
                    let gdip_draw_image_status = windows::Win32::Graphics::GdiPlus::GdipDrawImage(graphics_ptr, img_container_ptr.img_ptr, 0.0, 50.0);
                    if gdip_draw_image_status.0 != 0 {
                        println!("GdipDrawImage status: {}", gdip_draw_image_status.0);
                    }
                }
            },
            WM_DESTROY => {
                PostQuitMessage(0);
            },
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}

#[derive(Default)]
struct MainWindow {
    window_title: windows::core::PCWSTR,
    class_name: windows::core::PCWSTR,
}

impl MainWindow  {
    pub fn start(mut self) -> windows::core::Result<()> {

        self.class_name = w!("Example App");

        let window_title = windows::core::w!("Example App Window");
        self.window_title = window_title;

        let module = unsafe { 
            GetModuleHandleW(None).expect("failed get module handle")
        };
        let h_instance: HINSTANCE = module.into();

        let wnd = WNDCLASSW { 
            hInstance: h_instance,
            lpszClassName: self.class_name,
            lpfnWndProc: Some(wnd_proc),
            ..Default::default()
        };

        unsafe {
            let r = windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&wnd);
            if r == 0 {
                println!("{:?}", GetLastError());
                panic!("");
            }
        }

        let mut gdip_token = 0usize;
        let mut gdip_input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let gdi_status = unsafe {
            GdiplusStartup(&mut gdip_token, &mut gdip_input, std::ptr::null_mut())
        };
        println!("GdiplusStartup Status: {}", gdi_status.0);

        let hwnd = unsafe {
            windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
            WS_EX_OVERLAPPEDWINDOW,
            self.class_name,            
            self.window_title,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT,
            CW_USEDEFAULT, CW_USEDEFAULT,
            None,
            None,
            None,
            None).expect("failed createwindow.")
        };

        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            loop {
                let get_message_result = GetMessageW(&mut msg, None, 0, 0);
                if !get_message_result.as_bool() {
                    break;
                }

                DispatchMessageW(&msg);
            }
            GdiplusShutdown(gdip_token);
            return Ok(())
        }
    }

}


fn main() {
  let w = MainWindow::default();
  let _ = w.start();
}


