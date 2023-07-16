#include <stddef.h>

#include "../plug/include/telekinesis_plug.h"
#include "../plug/target/cxxbridge/plug/src/logging.rs.h"

using namespace RE;
using namespace RE::BSScript;
using namespace SKSE::log;
using namespace SKSE::stl;
using namespace SKSE;
using namespace REL;

#define DllExport __declspec(dllexport)

constexpr std::string_view PapyrusClass = "Tele";

bool RegisterPapyrusCalls(IVirtualMachine* vm) {
    vm->RegisterFunction("ScanForDevices", PapyrusClass, (bool (*)(StaticFunctionTag*)) ConnectAndScanForDevices);
    vm->RegisterFunction("GetDeviceNames", PapyrusClass, (std::vector<std::string>(*)(StaticFunctionTag*)) GetDeviceNames);
    vm->RegisterFunction("GetDeviceCapabilities", PapyrusClass, (std::vector<std::string>(*)(StaticFunctionTag*, std::string)) GetDeviceCapabilities);
    vm->RegisterFunction("GetDeviceConnected", PapyrusClass, (bool (*)(StaticFunctionTag*, std::string)) GetDeviceConnected);
    vm->RegisterFunction("Vibrate", PapyrusClass, (bool (*)(StaticFunctionTag*, int, float, std::vector<std::string>))Vibrate);
    vm->RegisterFunction("VibrateAll", PapyrusClass, (bool (*)(StaticFunctionTag*, int)) VibrateAll);
    vm->RegisterFunction("VibrateAllFor", PapyrusClass, (bool (*)(StaticFunctionTag*, int, float)) VibrateAllFor);
    vm->RegisterFunction("StopAll", PapyrusClass, (bool (*)(StaticFunctionTag*)) StopAll);
    vm->RegisterFunction("PollEvents", PapyrusClass, (std::vector<std::string>(*)(StaticFunctionTag*)) PollEvents);
    vm->RegisterFunction("Close", PapyrusClass, (bool (*)(StaticFunctionTag*)) Close);
    return true;
}

void InitializePapyrus() {
    log::trace("Initializing Papyrus binding...");
    if (GetPapyrusInterface()->Register(RegisterPapyrusCalls)) {
        log::debug("Papyrus functions bound.");
    } else {
        stl::report_and_fail("Failure to register Papyrus bindings.");
    }
}

std::string GetLogFile() {
    auto path = log_directory();
    if (!path) {
        report_and_fail("Unable to lookup SKSE logs directory.");
    }
    return std::format("{}\\{}.log", path->string(), PluginDeclaration::GetSingleton()->GetName());
}

SKSEPluginLoad(const LoadInterface* skse) {
    tk_init_logging(::rust::String(GetLogFile())); 

    auto* plugin = PluginDeclaration::GetSingleton();
    auto version = plugin->GetVersion();
    tk_log_info(std::format("{} {} is loading...", plugin->GetName(), version));

    Init(skse);
    InitializePapyrus();

    tk_log_info(std::format("{} has finished loading.", plugin->GetName()));
    return true;
}
