#include "Papyrus.h"

#include <../plug/include/telekinesis_plug.h>

#define DllExport __declspec(dllexport)

using namespace RE;
using namespace RE::BSScript;
using namespace REL;
using namespace SKSE;

namespace Telekinesis {
    constexpr std::string_view PapyrusClass = "Tele";

    bool RegisterPapyrusCalls(IVirtualMachine* vm) {
        vm->RegisterFunction("ScanForDevices", PapyrusClass, (bool (*)(StaticFunctionTag*)) ConnectAndScanForDevices);
        vm->RegisterFunction("VibrateAll", PapyrusClass, (bool (*)(StaticFunctionTag*, int)) VibrateAll);
        vm->RegisterFunction("VibrateAllFor", PapyrusClass, (bool (*)(StaticFunctionTag*, int, float)) VibrateAllFor);
        vm->RegisterFunction("StopAll", PapyrusClass, (bool (*)(StaticFunctionTag*)) StopAll);
        vm->RegisterFunction("PollEvents", PapyrusClass, (std::vector<std::string>(*) (StaticFunctionTag*))PollEvents);
        vm->RegisterFunction("Close", PapyrusClass, (bool (*)(StaticFunctionTag*)) Close);
        return true;
    }
}