use std::cell::RefCell;
use std::sync::{LazyLock, Mutex};

use std::os::windows::ffi::OsStrExt;

use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, FillRect, InvalidateRect, DRAW_TEXT_FORMAT, WHITE_PEN
};
use windows::Win32::Graphics::GdiPlus::{
    Bitmap, GdipCreateSolidFill, GdipDeleteBrush, GdipDeleteGraphics, GdipGetPenBrushFill,
    GdipSetPenBrushFill,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateMenu, CreateWindowExA, CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, GetWindowRect, MessageBoxW, SetWindowLongPtrW, BS_DEFPUSHBUTTON, GWLP_HINSTANCE, GWLP_USERDATA, HMENU, MB_OK, MF_ENABLED, MINMAXINFO, WM_COMMAND, WM_ERASEBKGND, WM_GETMINMAXINFO, WM_SIZE, WS_CHILD, WS_TABSTOP
};
use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::{
        Gdi::{
            BeginPaint, BitBlt, CreateCompatibleDC, DT_WORDBREAK, HBITMAP, HGDIOBJ, SRCCOPY,
            SelectObject,
        },
        GdiPlus::{
            GdipCreateFromHDC, GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, GpGraphics,
            GpImage, Image,
        },
    },
    System::LibraryLoader::{GetModuleHandleA, GetModuleHandleExW},
    UI::WindowsAndMessaging::{
        CW_USEDEFAULT, DispatchMessageW, GDI_IMAGE_TYPE, GetClientRect, GetMessageW, IMAGE_BITMAP,
        LR_CREATEDIBSECTION, LR_LOADFROMFILE, PostQuitMessage, RegisterClassW, SW_SHOW, ShowWindow,
        WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSA, WNDCLASSW,
        WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};
use windows::core::w;

fn convert_u8_to_u16(src: &str) -> Vec<u16> {
    // ref from https://teratail.com/questions/lcimq2rocy2hyu
    let a: Vec<u16> = std::ffi::OsString::from(src).encode_wide().collect();
    a
}

struct ImageContainer {
    img_ptr: *mut GpImage,
}

fn draw(hwnd: HWND) {
    let mut drawable_rect = windows::Win32::Foundation::RECT::default();
    let _ = unsafe { GetClientRect(hwnd, &mut drawable_rect) };
    let drawable_width = if drawable_rect.right > 100 {
        drawable_rect.right - drawable_rect.left
    } else {
        640
    };
    let drawable_height = if drawable_rect.bottom > 100 {
        drawable_rect.bottom - drawable_rect.top
    } else {
        480
    };

    let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    let mem_dc = unsafe { CreateCompatibleDC(Some(hdc)) };
    let h_bitmap = unsafe { CreateCompatibleBitmap(hdc, drawable_width, drawable_height) };
    let h_bitmap_obj = HGDIOBJ::from(h_bitmap);
    let old_bitmap = unsafe { SelectObject(mem_dc, h_bitmap_obj) };

    let mut graphics = windows::Win32::Graphics::GdiPlus::GpGraphics::default();
    let mut graphics_ptr: *mut GpGraphics = &mut graphics;
    let graphics_ptr_ptr: *mut *mut GpGraphics = &mut graphics_ptr;
    let graphics_status = unsafe { GdipCreateFromHDC(mem_dc, graphics_ptr_ptr) };
    if graphics_status.0 != 0 {
        println!("gprahics_status: {}", graphics_status.0);
    }
    unsafe {
        windows::Win32::Graphics::GdiPlus::GdipGraphicsClear(
            graphics_ptr,
            windows::Win32::Graphics::GdiPlus::Color::White as u32,
        )
    };

    let mut text_rect = windows::Win32::Foundation::RECT::default();
    text_rect.top = 100;
    text_rect.bottom = 200;
    text_rect.left = 0;
    text_rect.right = 100;
    let _ = unsafe {
        windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut convert_u8_to_u16("猫"), &mut text_rect, DT_WORDBREAK)
    };

    let img_container_data =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ImageContainer };
    if !img_container_data.is_null() {
        let img_container_ptr = unsafe { &*img_container_data };
        let mut width = 0u32;
        let mut height = 0u32;
        let _ = unsafe {
            windows::Win32::Graphics::GdiPlus::GdipGetImageWidth(
                img_container_ptr.img_ptr,
                &mut width,
            )
        };
        let _ = unsafe {
            windows::Win32::Graphics::GdiPlus::GdipGetImageHeight(
                img_container_ptr.img_ptr,
                &mut height,
            )
        };
        let is_height_higher_than_width = height > width;
        let ratio = if is_height_higher_than_width {
            (height as f32) / (width as f32)
        } else {
            (width as f32) / (height as f32)
        };

        let mut draw_img_rect = drawable_rect.clone();
        draw_img_rect.left = text_rect.right;

        let mut is_overflow = false;
        let mut new_height;
        let mut new_width;
        if is_height_higher_than_width {
            new_height = (draw_img_rect.bottom - draw_img_rect.top) as f32;
            new_width = new_height / ratio;
            
            let draw_img_width = (draw_img_rect.right - draw_img_rect.left) as f32;
            if new_width > draw_img_width {
                let new_draw_height = draw_img_width / ratio;

                new_height = new_draw_height;
                new_width = new_height / ratio;

                is_overflow = true;
            }
        } else {
            new_width = (draw_img_rect.right - draw_img_rect.left) as f32;
            new_height = new_width / ratio;

            let draw_img_height = (draw_img_rect.bottom - draw_img_rect.top) as f32;
            if new_height > draw_img_height {
                let new_draw_width = draw_img_height / ratio;

                new_width = new_draw_width;
                new_height = new_width / ratio;

                is_overflow = true;
            }
        }

        let x = if is_height_higher_than_width {
            ((draw_img_rect.right as f32) - new_width) * 0.5f32
        } else {
            if is_overflow {
                ((draw_img_rect.right as f32) - new_width) * 0.5f32
            } else {
                draw_img_rect.left as f32
            }
        };
        let y = if is_height_higher_than_width {
            if is_overflow {
                ((draw_img_rect.bottom as f32) - new_height)  * 0.5f32
            } else {
                draw_img_rect.top as f32
            }
        } else {
            ((draw_img_rect.bottom as f32) - new_height) * 0.5f32
        };
        let gdip_draw_image_status = unsafe {
            windows::Win32::Graphics::GdiPlus::GdipDrawImageRect(
                graphics_ptr,
                img_container_ptr.img_ptr,
                x,
                y,
                new_width,
                new_height,
            )
        };
        if gdip_draw_image_status.0 != 0 {
            println!("GdipDrawImage status: {}", gdip_draw_image_status.0);
        }
    }

    unsafe { GdipDeleteGraphics(graphics_ptr) };

    let _ = unsafe {
        BitBlt(
            hdc,
            0,
            0,
            drawable_width,
            drawable_height,
            Some(mem_dc),
            0,
            0,
            SRCCOPY,
        )
    };
    unsafe { SelectObject(mem_dc, old_bitmap) };
    let _ = unsafe { DeleteObject(h_bitmap_obj) };
    let _ = unsafe { DeleteDC(mem_dc) };
    let _ = unsafe { EndPaint(hwnd, &mut paint) };
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                // let img_path = w!("C:\\Users\\user\\Pictures\\example.bmp"); // for vertical test
                let img_path = w!("C:\\Users\\user\\Pictures\\example_horizonal.bmp"); // for horizonal test
                let mut img = windows::Win32::Graphics::GdiPlus::GpImage::default();
                let mut img_ptr: *mut GpImage = &mut img;
                let s = windows::Win32::Graphics::GdiPlus::GdipLoadImageFromFile(
                    img_path,
                    &mut img_ptr,
                );
                println!("GdipLoadImageFromFile: {}", s.0);
                if s.0 == 0 {
                    let img_container = Box::new(ImageContainer { img_ptr: img_ptr });
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(img_container) as isize);
                }

                let mut hinst_from_longptr = GetWindowLongPtrW(hwnd, GWLP_HINSTANCE);
                let hinst_ptr: *mut isize = &mut hinst_from_longptr;
                let hinst = HINSTANCE(hinst_ptr as *mut std::ffi::c_void);
                let button_num = 1001isize;
                let button_hmenu = HMENU(button_num as *mut _);
                let _ = 
                CreateWindowExW(WS_EX_OVERLAPPEDWINDOW,
                     w!("BUTTON"),
                     w!("PUSH ME"),
                     WS_TABSTOP | WS_VISIBLE | WS_CHILD,
                     0, 
                     0, 
                     100,
                     100,
                     Some(hwnd),
                     Some(button_hmenu),
                     Some(hinst),
                     None);
            }
            WM_GETMINMAXINFO => {
                let min_max_info = lparam.0 as *mut MINMAXINFO;
                if !min_max_info.is_null() {
                    (*min_max_info).ptMinTrackSize.x = 320;
                    (*min_max_info).ptMinTrackSize.y = 240;
                }
            }
            WM_SIZE => {
                let img_container_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ImageContainer;
                if !img_container_data.is_null() {
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
            },
            WM_PAINT => {
                draw(hwnd);
            }
            WM_COMMAND => {
                let command = wparam.0 & 0xFFFF;
                println!("get command {command}");
                match command {
                    1001 =>{
                        MessageBoxW(Some(hwnd), w!("Click!"), w!(""), MB_OK);
                    },
                    _ => {

                    }
                }
            },
            WM_ERASEBKGND => {
                return LRESULT(0);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
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

impl MainWindow {
    pub fn start(mut self) -> windows::core::Result<()> {
        self.class_name = w!("Example App");

        let window_title = windows::core::w!("Example App Window");
        self.window_title = window_title;

        let module = unsafe { GetModuleHandleW(None).expect("failed get module handle") };
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
        let gdi_status =
            unsafe { GdiplusStartup(&mut gdip_token, &mut gdip_input, std::ptr::null_mut()) };
        println!("GdiplusStartup Status: {}", gdi_status.0);

        let hwnd = unsafe {
            windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                WS_EX_OVERLAPPEDWINDOW,
                self.class_name,
                self.window_title,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                None,
                None,
            )
            .expect("failed createwindow.")
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
            return Ok(());
        }
    }
}

fn main() {
    let w = MainWindow::default();
    let _ = w.start();
}
