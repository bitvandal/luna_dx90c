fn main() {
    cc::Build::new().file("d3dx9_bindings.cpp")
        .include("C:\\Users\\jose\\Downloads\\DXSDK_Jun10\\DXSDK\\Include")
        // .static_flag(true)
        .compile("d3dx9_bindings");
}