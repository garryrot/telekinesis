#include "Papyrus.h"

#include <../plug/include/telekinesis_plug.h>

using namespace RE;
using namespace RE::BSScript;
using namespace REL;
using namespace SKSE;

namespace Telekinesis {

    constexpr std::string_view PapyrusClass = "Tele";
    static void* _tk = NULL;

    __declspec(dllexport) bool ConnectAndScanForDevices(StaticFunctionTag*) {
        log::info("Telekinesis::ScanForDevices");
        if (_tk == NULL) {
            if ((_tk = tk_connect()) == NULL) {
                log::error("tk_connect returned null pointer. Server not created.");
                return false;
            }
        }
        tk_scan_for_devices(_tk);
        return true;
    }

    __declspec(dllexport) bool VibrateAll(StaticFunctionTag*, float_t speed) {
        log::info("Telekinesis::VibrateAll ( speed={} ) ", speed );
        if (_tk == NULL)
            return false;

        tk_vibrate_all(_tk, speed);
        return true;
    }

    __declspec(dllexport) bool VibrateAllFor(StaticFunctionTag*, float_t speed, float_t time_sec) {
        log::info("Telekinesis::VibrateAllFor ( speed={}, time_sec={} ) ", speed, time_sec);
        if (_tk == NULL) return false;

        tk_vibrate_all_for(_tk, speed, time_sec);
        return true;
    }

    __declspec(dllexport) bool StopAll(StaticFunctionTag*) {
        log::info("Telekinesis::StopAll");
        if (_tk == NULL)
            return false;

        tk_stop_all(_tk);
        return true;
    }

    __declspec(dllexport) std::vector<std::string> PollEventsStdString() {
        std::vector<std::string> output;
        log::info("Telekinesis::PollEvents");
        if (_tk == NULL) {
            return output;
        }

        int8_t* evt;
        int i = 0;
        while (i++ < 128 && (evt = tk_try_get_next_event(_tk)) != NULL) {
            std::string evtstr((char*)evt);
            output.push_back(evtstr);
            tk_free_event(_tk, evt);
            log::info("Received event: {}.", evtstr);
        }
        return output; 
    }
    
    __declspec(dllexport) std::vector<RE::BSFixedString> PollEvents(StaticFunctionTag*) {
        std::vector<RE::BSFixedString> output;
        auto evts = PollEventsStdString();
        for (size_t i = 0; i < evts.size(); i++) {
            output.push_back(evts[i]);
        }
        return output;
    } 

    __declspec(dllexport) bool Close(StaticFunctionTag*) {
        log::info("Telekinesis::Close");
        if (_tk == NULL)
            return false;

        tk_close(_tk);
        _tk = NULL;
        return true;
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