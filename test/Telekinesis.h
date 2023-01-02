#pragma once

using namespace RE;

// Only for testing...
namespace Telekinesis { 
    __declspec(dllexport) bool Tk_ConnectAndScanForDevices(StaticFunctionTag*);
    __declspec(dllexport) bool Tk_StartVibrateAll(StaticFunctionTag*, float_t speed);
    __declspec(dllexport) bool Tk_StopAll(StaticFunctionTag*);
    __declspec(dllexport) std::vector<std::string> Tk_PollEventsStdString();
    __declspec(dllexport) bool Tk_Close(StaticFunctionTag*);
}
