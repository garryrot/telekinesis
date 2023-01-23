#pragma once

using namespace RE;

// Only for testing...
namespace Telekinesis { 
    __declspec(dllexport) bool ConnectAndScanForDevices(StaticFunctionTag*);
    __declspec(dllexport) bool VibrateAll(StaticFunctionTag*, int speed);
    __declspec(dllexport) bool VibrateAllFor(StaticFunctionTag*, int speed, float_t time_sec);
    __declspec(dllexport) bool StopAll(StaticFunctionTag*);
    __declspec(dllexport) std::vector<std::string> PollEventsStdString();
    __declspec(dllexport) bool Close(StaticFunctionTag*);
}
