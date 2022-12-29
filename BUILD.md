# Telekinesis (Bluetooth Toy Control for Papyrus Scripts)

Telekinesis provides various papyrus functions to control Buttplug.IO, allowing you to use bluetooth toys from within Skyrim SE. This is a modder resource/plugin for SKSE64 (Skyrim Script Extender) and wont do anything for you if you do not intent to create Mods.

## Modder Guide

If you just intent to create mods with the papyrus functions, you can ignore this page and go straight to the [User Documentation](README.md)

## Building from Source

This project builds a SKSE64 plugin `Telekinesis.dll` (using CommonLibSSE NG), a Plugin file (.esp) and some compiled pyparus files (.psx), which are required to use this in a skyrim.

I do not provide detailed build instructions (and won't promise that I ever will), but building this project should be fairly similar to the [CommonLibSSE NG Sample Plugin](https://gitlab.com/colorglass/commonlibsse-sample-plugin), which has amazingly well documented build instructions, so you probably want to do the following:

 1. Install the [Rust Compiler] https://www.rust-lang.org/tools/install

 2. See if you can get the [CommonLibSSE NG Sample Plugin](https://gitlab.com/colorglass/commonlibsse-sample-plugin) up and running

 3. Try to build this project in the same way as the Sample Plugin

### Dependencies
 
 * Rust
    * Buttplug.io (Rust Crate)
    * All rust crates listed in the toml
 * CommonLibSSE NG and all of its dependencies
 * CMake/Visual Studio 2022/See Sample Plugin dock


