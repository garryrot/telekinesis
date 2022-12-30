#pragma once

using namespace RE;

// Only for testing...
namespace Telekinesis { 
    __declspec(dllexport) bool Tk_ConnectAndScanForDevices(StaticFunctionTag*);
    __declspec(dllexport) int Tk_StartVibrateAll(StaticFunctionTag*, float_t speed);
    __declspec(dllexport) int Tk_StopAll(StaticFunctionTag*);
    __declspec(dllexport) BSFixedString Tk_PollEvents(StaticFunctionTag*);
    __declspec(dllexport) bool Tk_Close(StaticFunctionTag*);
}
