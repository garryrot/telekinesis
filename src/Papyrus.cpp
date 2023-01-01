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

    __declspec(dllexport) bool Tk_StartVibrateAll(StaticFunctionTag*, float_t speed) {
        log::info("TK_StartVibrateAll ( speed={} ) ", speed );
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

    __declspec(dllexport) RE::BSFixedString Tk_PollEvents(StaticFunctionTag*) {
        log::info("Tk_PollEvents");
        if (_tk == NULL) {
            log::error("event query while _tk does not exist");
            return BSFixedString("");
        }

        if (int8_t* evt = tk_await_next_event(_tk)) {
            std::string evtstr((char*)evt);
            log::info("Received event: {}.", evtstr);
            tk_free_event(_tk, evt);
            return BSFixedString(evtstr);
        } else {
            log::debug("no new event");
            return BSFixedString("");
        }
    } 

    __declspec(dllexport) bool Tk_Close(StaticFunctionTag*) {
        log::info("Tk_Close");
        if (_tk == NULL)
            return false;

        tk_close(_tk);
        _tk = NULL;
        return true;
    }
    

    //__declspec(dllexport) std::vector<RE::BSFixedString> Tk_Get_All_Devices(StaticFunctionTag*) {
    //    std::vector<RE::BSFixedString> output;
    //    if (_tk == NULL) return output;
    //    // TODO: Implement me
    //    for (size_t i = 0; i < len; i++) {
    //        output.push_back(BSFixedString());
    //    }
    //    return output;
    //}

    bool RegisterPapyrusCalls(IVirtualMachine* vm) {
        vm->RegisterFunction("TK_ScanForDevices", PapyrusClass, Tk_ConnectAndScanForDevices);
        vm->RegisterFunction("TK_StartVibrateAll", PapyrusClass, Tk_StartVibrateAll);
        vm->RegisterFunction("Tk_StopAll", PapyrusClass, Tk_StopAll);
        vm->RegisterFunction("Tk_PollEvents", PapyrusClass, Tk_PollEvents);
        vm->RegisterFunction("Tk_Close", PapyrusClass, Tk_Close);
        return true;
    }
}