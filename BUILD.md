# Building Telekinesis.dll

This build outputs `Telekinesis.dll`, a SKSE64 plugin based on CommonLibSSE NG.

There is no need for you to do this, unless you...

- want to port the native library to a skyrim version that I currently don't support
- want to fork this project and change the native library

## Build Requirements

 1. [Rust](https://www.rust-lang.org/tools/install) - executables like `cargo` should be present in your PATH
 2. [Visual Studio 2022](https://visualstudio.microsoft.com/de/) with a C++ compiler
 3. [CMake](https://cmake.org/download/) - make sure that its added to your PATH environment variable
 4. [VCPKG](https://github.com/microsoft/vcpkg) - set environment variable VCPKG_ROOT to the vcpkg installation folder

## Step-By-Step

1. Make sure the submodule of CommonLibSSE-NG is initialised:

```ps
git submodule update --init --recursive
```
2. Test VCPKG_ROOT is set in your build terminal. This should return the path:

```
echo %VCPKG_ROOT%
```

3. Build the project

```ps
cmake --preset build
cmake --build --preset build --config Release
```

# Building Papyrus Scripts 

*to be done*