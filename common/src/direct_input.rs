use windows::{
    core::*, Win32::Devices::HumanInterfaceDevice::*, Win32::Foundation::*,
};

use winapi::um::dinput::{c_dfDIKeyboard, c_dfDIMouse2};

use crate::*;

pub struct DirectInput {
    keyboard: Option<IDirectInputDevice8A>,
    mouse: Option<IDirectInputDevice8A>,
    keyboard_state: [u8; 256],
    mouse_state: DIMOUSESTATE2,
    dinput: Option<IDirectInput8A>,
}

impl DirectInput {
    pub fn new(app_inst: HINSTANCE, main_wnd: HWND, keyboard_coop_flags: u32, mouse_coop_flags: u32) -> Option<DirectInput> {
        unsafe {
            let mut direct_input = DirectInput {
                keyboard: None,
                mouse: None,
                keyboard_state: [0; 256],
                mouse_state: std::mem::zeroed(),
                dinput: None,
            };

            HR!(DirectInput8Create(
                app_inst,
                DIRECTINPUT_VERSION,
                &IDirectInput8A::IID,
                &mut direct_input.dinput as *mut _ as _,
                None
            ));

            direct_input.dinput.clone().map_or_else(|| None, |dinput: IDirectInput8A| {
                HR!(dinput.CreateDevice(&GUID_SysKeyboard, &mut direct_input.keyboard, None));

                if let Some(keyboard) = direct_input.keyboard {
                    let mut df_keyboard: DIDATAFORMAT = std::mem::transmute(c_dfDIKeyboard);
                    HR!(keyboard.SetDataFormat(&mut df_keyboard));
                    HR!(keyboard.SetCooperativeLevel(main_wnd, keyboard_coop_flags));
                    HR!(keyboard.Acquire());

                    direct_input.keyboard = Some(keyboard);
                }

                HR!(dinput.CreateDevice(&GUID_SysMouse, &mut direct_input.mouse, None));
                if let Some(mouse) = direct_input.mouse {
                    let mut df_mouse: DIDATAFORMAT = std::mem::transmute(c_dfDIMouse2);
                    HR!(mouse.SetDataFormat(&mut df_mouse));
                    HR!(mouse.SetCooperativeLevel(main_wnd, mouse_coop_flags));
                    HR!(mouse.Acquire());

                    direct_input.mouse = Some(mouse);
                }
                Some(direct_input)
            })
        }
    }

    pub fn release_com_objects(&self) {
        // windows crate should drop automatically those objects when they fall out of scope
        // drop(&self.dinput);

        if let Some(keyboard) = &self.keyboard {
            unsafe { let _result = keyboard.Unacquire(); }
            // drop(&self.keyboard);

        }
        if let Some(mouse) = &self.mouse {
            unsafe { let _result = mouse.Unacquire(); }
            // drop(&mouse);
        }
    }

    pub fn poll(&mut self) {
        unsafe {
            // Poll keyboard.
            if let Some(keyboard) = &self.keyboard {
                let hr = keyboard.GetDeviceState(self.keyboard_state.len() as u32,
                                                 self.keyboard_state.as_mut_ptr().cast());
                if FAILED!(hr) {
                        // Keyboard lost, zero out keyboard data structure.
                        self.keyboard_state = std::mem::zeroed();

                    // Try to acquire for next time we poll.
                    let _hr = keyboard.Acquire();
                }
            }

            if let Some(mouse) = &self.mouse {
                // Poll mouse.
                let hr = mouse.GetDeviceState(std::mem::size_of::<DIMOUSESTATE2>() as u32,
                                                  &mut self.mouse_state as *mut _ as _);
                if FAILED!(hr) {
                    // Mouse lost, zero out mouse data structure.
                    self.mouse_state = std::mem::zeroed();

                    // Try to acquire for next time we poll.
                    let _hr = mouse.Acquire();
                }
            }
        }
    }

    pub fn key_down(&mut self, key: usize) -> bool {
        self.keyboard_state[key] & 0x80 != 0
    }

    pub fn mouse_dx(&mut self) -> f32 {
        self.mouse_state.lX as f32
    }

    pub fn mouse_dy(&mut self) -> f32 {
        self.mouse_state.lY as f32
    }

    pub fn mouse_dz(&mut self) -> f32 {
        self.mouse_state.lZ as f32
    }
}