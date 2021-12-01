# Topic

Colored Cube Demo.

To compile, requires to place these static library files at same level of Cargo.toml file:
* `DxErr.lib`, `d3dx9.lib` from latest legacy DX SDK Jun 2010
* `legacy_stdio_definitions.lib` from VS 2015 or later (can also be gotten from Microsoft C++ Build Tools).
  * See info here:
    * https://docs.microsoft.com/en-us/windows/win32/directx-sdk--august-2009-
    * https://docs.microsoft.com/en-us/cpp/porting/overview-of-potential-upgrade-issues-visual-cpp?view=msvc-170

Edit in `build.rs` file the location of the legacy DXSDK.

Copy from Book resources the resource files (FX file) into project root directory.
