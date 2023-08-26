# Build from Source

If you just intent to create mods with the papyrus functions, you can ignore this page and go straight to the [User Documentation](README.md)

## Building Dll

The build outputs `Telekinesis.dll` (An SKSE64 plugin using CommonLibSSE NG), a Skyrim Plugin file `.esp` and the compiled pyparus files `.psx` required to use the Telekinesis API from within other mods.

I do not provide detailed build instructions (and won't promise that I ever will), but building this project should be fairly similar to the [CommonLibSSE NG Sample Plugin](https://gitlab.com/colorglass/commonlibsse-sample-plugin), which has amazingly well documented build instructions. You probably want to do the following:

 1. [Install Rust](https://www.rust-lang.org/tools/install)
 2. See if you can get the [CommonLibSSE NG Sample Plugin](https://gitlab.com/colorglass/commonlibsse-sample-plugin) up and running
 3. Try to build this project in the same way as the Sample Plugin

### Dependencies
 
 * Rust
    * Buttplug.io (Rust Crate)
    * All rust crates listed in the toml
 * CommonLibSSE NG and all of its dependencies
 * CMake/Visual Studio 2022/See Sample Plugin dock

## Building Plugin Code

You need to include all dependency psc scripts that are not fetched with ninja in contrib/Dependencies

For example:
   - SkyUI
   - Devious Devices [SE][AE][VR] 5.2 5.2
   - Sexlab Framework
   - MfgFix
   - And their respective dependencies (a lot)

