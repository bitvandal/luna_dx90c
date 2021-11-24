use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D9::*,
};

unsafe fn enum_system_display_modes() -> Result<()> {
    let d3d9: Option<IDirect3D9> = Direct3DCreate9(D3D_SDK_VERSION);

    match d3d9 {
        Some(d3d_object) => {
            let adapter_count = d3d_object.GetAdapterCount();

            for i in 0..adapter_count {
                let mode_count = d3d_object.GetAdapterModeCount(i, D3DFMT_X8R8G8B8);
                println!("Listing {} display modes for adapter #{}:", mode_count, i);

                let mut display_mode: D3DDISPLAYMODE = D3DDISPLAYMODE::default();

                for j in 0..mode_count {
                    let result = d3d_object.EnumAdapterModes(
                        i,
                        D3DFMT_X8R8G8B8,
                        j,
                        &mut display_mode);

                    match result {
                        Ok(()) => println!("{:?}", display_mode),
                        Err(_) => return result,
                    }
                }
            }

            // This is done automatically when goes out of scope, here the implementation of drop
            // is a call to COM Release, using knowo position in vtable for IUnknown
            // drop(d3d_object);

            Ok(())
        }
        None => return Err(Error::new(E_FAIL, HSTRING::from("Could not obtain IDirect3D9 interface!"))),
    }
}

fn main() -> Result<()> {
    unsafe { return enum_system_display_modes(); };
}