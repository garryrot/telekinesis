#include "Papyrus.h"

#include <../plug/include/telekinesis_plug.h>

using namespace RE;
using namespace RE::BSScript;
using namespace REL;
using namespace SKSE;

namespace Telekinesis {

    constexpr std::string_view PapyrusClass = "TK_Telekinesis";
    static void* _tk = NULL;

    __declspec(dllexport) bool Tk_ConnectAndScanForDevices(StaticFunctionTag*) {
        log::info("TK_ScanForDevices");
        if (_tk == NULL) {
            if ((_tk = tk_connect()) == NULL) {
                log::error("tk_connect returned null pointer. Server not created.");
                return false;
            }
        }
        tk_scan_for_devices(_tk);
        return true;
    }

    __declspec(dllexport) bool TK_VibrateAll(StaticFunctionTag*, float_t speed) {
        log::info("TK_VibrateAll ( speed={} ) ", speed );
        if (_tk == NULL)
            return false;

        tk_vibrate_all(_tk, speed);
        return true;
    }

    __declspec(dllexport) bool Tk_StopAll(StaticFunctionTag*) {
        log::info("Tk_StopAll");
        if (_tk == NULL)
            return false;

        tk_stop_all(_tk);
        return true;
    }

    __declspec(dllexport) std::vector<std::string> Tk_PollEventsStdString() {
        std::vector<std::string> output;
        log::info("Tk_PollEvents");
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
    
    __declspec(dllexport) std::vector<RE::BSFixedString> Tk_PollEvents(StaticFunctionTag*) {
        std::vector<RE::BSFixedString> output;
        auto evts = Tk_PollEventsStdString();
        for (size_t i = 0; i < evts.size(); i++) {
            output.push_back(evts[i]);
        }
        return output;
    } 

    __declspec(dllexport) bool Tk_Close(StaticFunctionTag*) {
        log::info("Tk_Close");
        if (_tk == NULL)
            return false;

        tk_close(_tk);
        _tk = NULL;
        return true;
    }

    bool RegisterPapyrusCalls(IVirtualMachine* vm) {
        vm->RegisterFunction("TK_ScanForDevices", PapyrusClass, Tk_ConnectAndScanForDevices);
        vm->RegisterFunction("TK_VibrateAll", PapyrusClass, TK_VibrateAll);
        vm->RegisterFunction("Tk_StopAll", PapyrusClass, Tk_StopAll);
        vm->RegisterFunction("Tk_PollEvents", PapyrusClass, Tk_PollEvents);
        vm->RegisterFunction("Tk_Close", PapyrusClass, Tk_Close);
        return true;
    }
}