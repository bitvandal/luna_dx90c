# Topic

Rain Demo.

To compile, requires to place these static library files at the ``dependencies`` directory at the root level of the
project:
* `DxErr.lib`, `d3dx9.lib` from latest legacy DX SDK Jun 2010
* `legacy_stdio_definitions.lib` from VS 2015 or later (can also be gotten from Microsoft C++ Build Tools).
  * See info here:
    * https://docs.microsoft.com/en-us/windows/win32/directx-sdk--august-2009-
    * https://docs.microsoft.com/en-us/cpp/porting/overview-of-potential-upgrade-issues-visual-cpp?view=msvc-170

Also, requires DX SDK Jun 2010 files to be put also in ``dependencies`` directory. See `build.rs` file for more details
on how the path is created.

Copy from Book resources the resource files (DDS files, FX files, RAW file) into package directory.
