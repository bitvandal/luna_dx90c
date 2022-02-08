fn main() {
    cc::Build::new().file("d3dx9_bindings.cpp")
        .include("../dependencies/DXSDK_Jun10/DXSDK/Include")
        .compile("d3dx9_bindings");
}