#include "Papyrus.h"

#include <../plug/include/telekinesis_plug.h>

#define DllExport __declspec(dllexport)

using namespace RE;
using namespace RE::BSScript;
using namespace REL;
using namespace SKSE;

namespace Telekinesis {
    constexpr std::string_view PapyrusClass = "Tele";
    DllExport bool ConnectAndScanForDevices(StaticFunctionTag*) {
        return tk_connect_and_scan(); 
    }
    DllExport bool VibrateAll(StaticFunctionTag*, int speed) { 
        return tk_vibrate_all((float_t)speed / 100.0); 
    }
    DllExport bool VibrateAllFor(StaticFunctionTag*, int speed, float_t time_sec) {
        return tk_vibrate_all_for((float_t)speed / 100.0, time_sec); 
    }
    DllExport bool StopAll(StaticFunctionTag*) {
        return tk_stop_all(); 
    }
    DllExport bool Close(StaticFunctionTag*) {
        tk_close();
        return true;
    }
    DllExport std::vector<std::string> PollEventsStdString() {
        std::vector<std::string> output;
        log::debug("Telekinesis::PollEvents");
        int8_t* evt;
        int i = 0;
        while (i++ < 128 && (evt = tk_try_get_next_event()) != NULL) {
            std::string evtstr((char*)evt);
            output.push_back(evtstr);
            tk_free_event(evt);
            log::info("Received Event: {}.", evtstr);
            spdlog::info("Received Event: {}.", evtstr);
        }
        return output;
    }
    DllExport std::vector<RE::BSFixedString> PollEvents(StaticFunctionTag*) {
        std::vector<RE::BSFixedString> output;
        auto evts = PollEventsStdString();
        for (size_t i = 0; i < evts.size(); i++) {
            output.push_back(evts[i]);
        }
        return output;
    }
    bool RegisterPapyrusCalls(IVirtualMachine* vm) {
        vm->RegisterFunction("ScanForDevices", PapyrusClass, ConnectAndScanForDevices); 
        vm->RegisterFunction("VibrateAll", PapyrusClass, VibrateAll);
        vm->RegisterFunction("VibrateAllFor", PapyrusClass, VibrateAllFor);
        vm->RegisterFunction("StopAll", PapyrusClass, StopAll);
        vm->RegisterFunction("PollEvents", PapyrusClass, PollEvents);
        vm->RegisterFunction("Close", PapyrusClass, Close);
        return true;
    }
}