pub mod gate_demo;
pub mod vertex;

use windows::{
    Win32::Foundation::*, Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::Graphics::Direct3D9::*, Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*,
    Win32::System::SystemServices::*, Win32::UI::Input::KeyboardAndMouse::*,
    Win32::System::Threading::*, Win32::System::Performance::*,
    Win32::Devices::HumanInterfaceDevice::*
};

use common::*;
use d3dx::*;
use std::ffi::{CStr};

use crate::gate_demo::*;
use crate::vertex::*;

// D3D App
pub struct D3DApp {
    app_inst: HINSTANCE,
    main_wnd: HWND,
    d3d_object: Option<IDirect3D9>,
    device_type: D3DDEVTYPE,
    requested_vp: i32,
    app_paused: bool,
    d3d_pp: D3DPRESENT_PARAMETERS,
    gate_demo: Option<GateDemo>,
}

// Unsafe global state
static mut D3D_APP: Option<D3DApp> = None;
static mut D3D_DEVICE: Option<IDirect3DDevice9> = None;
static mut DIRECT_INPUT: Option<DirectInput> = None;

// Is the application in a minimized or maximized state?
static mut MIN_OR_MAXED: bool = false;

fn main() {
    unsafe {
        D3D_APP = D3DApp::new(D3DDEVTYPE_HAL, D3DCREATE_HARDWARE_VERTEXPROCESSING);

        if let Some(d3d_app) = &mut D3D_APP {
            if let Some(d3d_device) = D3D_DEVICE.clone() {
                let gate_demo = GateDemo::new(d3d_device.clone(), &d3d_app.d3d_pp);
                d3d_app.gate_demo = gate_demo;

                DIRECT_INPUT = DirectInput::new(d3d_app.app_inst,
                                                d3d_app.main_wnd,
                                                DISCL_NONEXCLUSIVE | DISCL_FOREGROUND,
                                                DISCL_NONEXCLUSIVE | DISCL_FOREGROUND);

                let exit_code = d3d_app.run();

                if let Some(gate_demo) = &d3d_app.gate_demo {
                    gate_demo.release_com_objects();
                }

                if let Some(dinput) = &DIRECT_INPUT {
                    dinput.release_com_objects();
                }

                std::process::exit(exit_code);
            }
        }
    }
}

impl D3DApp {
    fn new(device_type: D3DDEVTYPE, requested_vp: i32) -> Option<D3DApp> {
        unsafe {
            let mut d3d_app = D3DApp {
                app_inst: HINSTANCE(0),
                main_wnd: HWND(0),
                d3d_object: None,
                device_type,
                requested_vp,
                app_paused: false,
                d3d_pp: std::mem::zeroed(),
                gate_demo: None,
            };

            d3d_app.init_main_window();
            d3d_app.init_direct_3d();

            Some(d3d_app)
        }
    }

    fn on_lost_device(&self) {
        if let Some(gate_demo) = &self.gate_demo {
            gate_demo.on_lost_device();
        }
    }

    fn on_reset_device(&mut self) {
        if let Some(gate_demo) = &mut self.gate_demo {
            gate_demo.on_reset_device();
        }
    }

    fn init_main_window(&mut self) {
        unsafe {
            self.app_inst = GetModuleHandleA(PSTR(std::ptr::null_mut()));

            let class_name = PSTR(b"D3DWndClassName\0".as_ptr() as _);

            let wc = WNDCLASSA {
                style: WNDCLASS_STYLES(CS_HREDRAW.0 | CS_VREDRAW.0),
                lpfnWndProc: Some(main_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: self.app_inst,
                hIcon: LoadIconW(HINSTANCE(0), IDI_APPLICATION),
                hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW),
                hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
                lpszMenuName: PSTR(std::ptr::null_mut()),
                lpszClassName: class_name,
            };

            if RegisterClassA(&wc) == 0 {
                display_error_then_quit("RegisterClass FAILED");
            }

            // Default to a window with a client area rectangle of 800x600.

            let mut r = RECT { left: 0, top: 0, right: 800, bottom: 600 };

            AdjustWindowRect(&mut r, WS_OVERLAPPEDWINDOW, false);

            let main_wnd_caption = PSTR(b"Gate Demo\0".as_ptr() as _);

            self.main_wnd = CreateWindowExA(
                WINDOW_EX_STYLE(0),
                class_name,
                main_wnd_caption,
                WS_OVERLAPPEDWINDOW,
                100,
                100,
                r.right,
                r.bottom,
                HWND(0),
                HMENU(0),
                self.app_inst,
                std::ptr::null_mut(),
            );

            if self.main_wnd.0 == 0 {
                display_error_then_quit("CreateWindow FAILED");
            }

            ShowWindow(self.main_wnd, SW_SHOW);
            UpdateWindow(self.main_wnd);
        }
    }

    fn init_direct_3d(&mut self) {
        unsafe {
            // Step 1: Create the IDirect3D9 object.
            self.d3d_object = Direct3DCreate9(D3D_SDK_VERSION);

            match &self.d3d_object {
                None => {
                    display_error_then_quit("Direct3DCreate9 FAILED");
                }
                Some(d3d_object) => {
                    // Step 2: Verify hardware support for specified formats in windowed and full screen modes.
                    let mut mode: D3DDISPLAYMODE = std::mem::zeroed();

                    HR!(d3d_object.GetAdapterDisplayMode(D3DADAPTER_DEFAULT, &mut mode));

                    HR!(d3d_object.CheckDeviceType(
                        D3DADAPTER_DEFAULT,
                        self.device_type,
                        mode.Format,
                        mode.Format,
                        true));

                    HR!(d3d_object.CheckDeviceType(
                        D3DADAPTER_DEFAULT,
                        self.device_type,
                        D3DFMT_X8R8G8B8,
                        D3DFMT_X8R8G8B8,
                        false));

                    // Step 3: Check for requested vertex processing and pure device.
                    let mut caps: D3DCAPS9 = std::mem::zeroed();

                    HR!(d3d_object.GetDeviceCaps(D3DADAPTER_DEFAULT, self.device_type, &mut caps));

                    let mut dev_behavior_flags: u32 = 0;

                    if caps.DevCaps & (D3DDEVCAPS_HWTRANSFORMANDLIGHT as u32) != 0 {
                        dev_behavior_flags |= self.requested_vp as u32;
                    } else {
                        dev_behavior_flags |= D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32;
                    }

                    // If pure device and HW T&L supported
                    if (caps.DevCaps & (D3DDEVCAPS_PUREDEVICE as u32) != 0) &
                        (dev_behavior_flags & (D3DCREATE_HARDWARE_VERTEXPROCESSING as u32) != 0) {
                        dev_behavior_flags |= D3DCREATE_PUREDEVICE as u32;
                    }

                    // Step 4: Fill out the D3DPRESENT_PARAMETERS structure.
                    self.d3d_pp = D3DPRESENT_PARAMETERS {
                        BackBufferWidth: 0,
                        BackBufferHeight: 0,
                        BackBufferFormat: D3DFMT_UNKNOWN,
                        BackBufferCount: 1,
                        MultiSampleType: D3DMULTISAMPLE_NONE,
                        MultiSampleQuality: 0,
                        SwapEffect: D3DSWAPEFFECT_DISCARD,
                        hDeviceWindow: self.main_wnd,
                        Windowed: BOOL::from(true),
                        EnableAutoDepthStencil: BOOL::from(true),
                        AutoDepthStencilFormat: D3DFMT_D24S8,
                        Flags: 0,
                        FullScreen_RefreshRateInHz: D3DPRESENT_RATE_DEFAULT,
                        PresentationInterval: D3DPRESENT_INTERVAL_IMMEDIATE as u32,
                    };

                    HR!(d3d_object.CreateDevice(
                        D3DADAPTER_DEFAULT,
                        self.device_type,
                        self.main_wnd,
                        dev_behavior_flags,
                        &mut self.d3d_pp,
                        &mut D3D_DEVICE));
                }
            }
        }
    }

    fn run(&mut self) -> i32 {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();

            let mut cnts_per_sec: i64 = 0;
            QueryPerformanceFrequency(&mut cnts_per_sec);

            let secs_per_cnt: f32 = 1.0 / (cnts_per_sec as f32);

            let mut prev_timestamp: i64 = 0;
            QueryPerformanceCounter(&mut prev_timestamp);

            while msg.message != WM_QUIT {
                // If there are Window messages then process them.
                if PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).into() {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                } else {
                    // Otherwise, do animation/game stuff.

                    // If the application is paused then free some CPU
                    // cycles to other applications and then continue on
                    // to the next frame.
                    if self.app_paused {
                        Sleep(20);
                        continue;
                    }

                    if !self.device_is_lost() {
                        let mut curr_timestamp: i64 = 0;
                        QueryPerformanceCounter(&mut curr_timestamp);
                        let dt: f32 = ((curr_timestamp - prev_timestamp) as f32) * secs_per_cnt;

                        if let Some(gate_demo) = &mut self.gate_demo {
                            gate_demo.update_scene(dt);
                            gate_demo.draw_scene();
                        }

                        // Prepare for next iteration: The current time stamp becomes
                        // the previous time stamp for the next iteration.
                        prev_timestamp = curr_timestamp;
                    }
                }
            }

            return msg.wParam.0 as i32;
        }
    }

    fn device_is_lost(&mut self) -> bool {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Get the state of the graphics device.
                let result = d3d_device.TestCooperativeLevel();

                if let Err(hresult) = result {
                    // If the device is lost and cannot be reset yet then
                    // sleep for a bit and we'll try again on the next
                    // message loop cycle.
                    return if hresult.code() == D3DERR_DEVICELOST {
                        Sleep(20);
                        true
                    }
                    // Driver error, exit.
                    else if hresult.code() == D3DERR_DRIVERINTERNALERROR {
                        display_error_then_quit("Internal Driver Error...Exiting");
                        true
                    }
                    // The device is lost but we can reset and restore it.
                    else if hresult.code() == D3DERR_DEVICENOTRESET {
                        self.on_lost_device();
                        HR!(d3d_device.Reset(&mut self.d3d_pp));
                        self.on_reset_device();
                        false
                    } else {
                        false
                    };
                }
            }
        }
        false
    }

    fn enable_full_screen_mode(&mut self, enable: bool) {
        unsafe {
            // Switch to fullscreen mode.
            if enable {
                // Are we already in fullscreen mode?
                if !self.d3d_pp.Windowed.as_bool() {
                    return;
                }

                let width = GetSystemMetrics(SM_CXSCREEN);
                let height = GetSystemMetrics(SM_CYSCREEN);

                self.d3d_pp = D3DPRESENT_PARAMETERS {
                    BackBufferFormat: D3DFMT_X8R8G8B8,
                    BackBufferWidth: width as u32,
                    BackBufferHeight: height as u32,
                    Windowed: BOOL::from(false),
                    ..self.d3d_pp
                };

                // Change the window style to a more fullscreen friendly style.
                SetWindowLongPtrA(self.main_wnd, GWL_STYLE, WS_POPUP.0 as isize);

                // If we call SetWindowLongPtr, MSDN states that we need to call
                // SetWindowPos for the change to take effect.  In addition, we
                // need to call this function anyway to update the window dimensions.
                SetWindowPos(self.main_wnd, HWND_TOP, 0, 0,
                             width, height, SWP_NOZORDER | SWP_SHOWWINDOW);
            }
            // Switch to windowed mode.
            else {
                // Are we already in windowed mode?
                if self.d3d_pp.Windowed.as_bool() {
                    return;
                }

                let mut r: RECT = RECT {
                    left: 0,
                    top: 0,
                    right: 800,
                    bottom: 600,
                };

                AdjustWindowRect(&mut r, WS_OVERLAPPEDWINDOW, false);

                self.d3d_pp = D3DPRESENT_PARAMETERS {
                    BackBufferFormat: D3DFMT_UNKNOWN,
                    BackBufferWidth: 800,
                    BackBufferHeight: 600,
                    Windowed: BOOL::from(true),
                    ..self.d3d_pp
                };

                // Change the window style to a more windowed friendly style.
                SetWindowLongPtrA(self.main_wnd, GWL_STYLE, WS_OVERLAPPEDWINDOW.0 as isize);

                // If we call SetWindowLongPtr, MSDN states that we need to call
                // SetWindowPos for the change to take effect.  In addition, we
                // need to call this function anyway to update the window dimensions.
                SetWindowPos(self.main_wnd, HWND_TOP, 100, 100, r.right, r.bottom, SWP_NOZORDER | SWP_SHOWWINDOW);
            }

            // Reset the device with the changes.
            self.on_lost_device();
            if let Some(d3d_device) = &D3D_DEVICE {
                HR!(d3d_device.Reset(&mut self.d3d_pp));
            }
            self.on_reset_device();
        }
    }

    fn msg_proc(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            let mut client_rect = RECT::default();

            match message as u32 {
                // WM_ACTIVE is sent when the window is activated or deactivated.
                // We pause the game when the main window is deactivated and
                // unpause it when it becomes active.
                WM_ACTIVATE => {
                    self.app_paused = (wparam.0 & 0x0000FFFF) as u32 == WA_INACTIVE;
                    LRESULT(0)
                }

                // WM_SIZE is sent when the user resizes the window.
                WM_SIZE => {
                    if let Some(d3d_device) = &D3D_DEVICE {
                        self.d3d_pp = D3DPRESENT_PARAMETERS {
                            BackBufferWidth: (lparam.0 & 0xFFFF) as u32,
                            BackBufferHeight: (lparam.0 >> 16 & 0xFFFF) as u32,
                            ..self.d3d_pp
                        };

                        if wparam.0 as u32 == SIZE_MINIMIZED {
                            self.app_paused = true;
                            MIN_OR_MAXED = true;
                        } else if wparam.0 as u32 == SIZE_MAXIMIZED {
                            self.app_paused = false;
                            MIN_OR_MAXED = true;
                            self.on_lost_device();
                            HR!(d3d_device.Reset(&mut self.d3d_pp));
                            self.on_reset_device();
                        } else if wparam.0 as u32 == SIZE_RESTORED {
                            // Restored is any resize that is not a minimize or maximize.
                            // For example, restoring the window to its default size
                            // after a minimize or maximize, or from dragging the resize
                            // bars.
                            self.app_paused = false;

                            // Are we restoring from a mimimized or maximized state,
                            // and are in windowed mode?  Do not execute this code if
                            // we are restoring to full screen mode.
                            if MIN_OR_MAXED && self.d3d_pp.Windowed.as_bool() {
                                self.on_lost_device();
                                HR!(d3d_device.Reset(&mut self.d3d_pp));
                                self.on_reset_device();
                            } else {
                                // No, which implies the user is resizing by dragging
                                // the resize bars.  However, we do not reset the device
                                // here because as the user continuously drags the resize
                                // bars, a stream of WM_SIZE messages is sent to the window,
                                // and it would be pointless (and slow) to reset for each
                                // WM_SIZE message received from dragging the resize bars.
                                // So instead, we reset after the user is done resizing the
                                // window and releases the resize bars, which sends a
                                // WM_EXITSIZEMOVE message.
                            }

                            MIN_OR_MAXED = false;
                        }
                    }

                    LRESULT(0)
                }

                // WM_EXITSIZEMOVE is sent when the user releases the resize bars.
                // Here we reset everything based on the new window dimensions.
                WM_EXITSIZEMOVE => {
                    GetClientRect(self.main_wnd, &mut client_rect);
                    self.d3d_pp = D3DPRESENT_PARAMETERS {
                        BackBufferWidth: client_rect.right as u32,
                        BackBufferHeight: client_rect.bottom as u32,
                        ..self.d3d_pp
                    };
                    self.on_lost_device();
                    if let Some(d3d_device) = &D3D_DEVICE {
                        HR!(d3d_device.Reset(&mut self.d3d_pp));
                    }
                    self.on_reset_device();
                    LRESULT(0)
                }

                // WM_CLOSE is sent when the user presses the 'X' button in the
                // caption bar menu.
                WM_CLOSE => {
                    DestroyWindow(self.main_wnd);
                    LRESULT(0)
                }

                // WM_DESTROY is sent when the window is being destroyed.
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }

                WM_KEYDOWN => {
                    if wparam.0 == VK_ESCAPE.0 as _ {
                        self.enable_full_screen_mode(false);
                    } else if wparam.0 as usize == 'F' as _ {
                        self.enable_full_screen_mode(true);
                    }
                    LRESULT(0)
                }

                _ => DefWindowProcA(self.main_wnd, message, wparam, lparam),
            }
        }
    }
}

extern "system" fn main_wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        // Don't start processing messages until the application has been created.
        D3D_APP.as_mut()
            .map_or_else(|| DefWindowProcA(window, message, wparam, lparam),
                         |d3d_app| d3d_app.msg_proc(message, wparam, lparam))
    }
}