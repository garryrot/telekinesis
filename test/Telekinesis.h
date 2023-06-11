#pragma once

using namespace RE;

// Only for testing...
namespace Telekinesis { 
    __declspec(dllexport) bool ConnectAndScanForDevices(StaticFunctionTag*);
    __declspec(dllexport) std::vector<std::string> PollEventsStdString();
    __declspec(dllexport) bool Close(StaticFunctionTag*);
}
