fn main() {
    println!(r"cargo:rustc-link-lib=dylib=dxguid");
    println!(r"cargo:rustc-link-search=native");
}