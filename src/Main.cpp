#include <stddef.h>

#include "../plug/include/api.h"
#include "../plug/target/cxxbridge/plug/src/logging.rs.h"

using namespace RE;
using namespace RE::BSScript;
using namespace SKSE::log;
using namespace SKSE::stl;
using namespace SKSE;
using namespace REL;

#define DllExport __declspec(dllexport)
#define SFT StaticFunctionTag*
#define TkNativeCall 

constexpr std::string_view PapyrusClass = "Tele_Api";

bool ApiLoaded(SFT) { return true; }

bool RegisterPapyrusCalls(IVirtualMachine* vm) {
    vm->RegisterFunction("Loaded", PapyrusClass, ApiLoaded);
    vm->RegisterFunction("Connect", PapyrusClass, (bool (*)(SFT))Tk::Connect);
    vm->RegisterFunction("ScanForDevices", PapyrusClass, (bool (*)(SFT))Tk::ScanForDevices);
    vm->RegisterFunction("GetConnectionStatus", PapyrusClass, (std::string (*)(SFT))Tk::GetConnectionStatus);
    vm->RegisterFunction("StopScan", PapyrusClass, (bool (*)(SFT))Tk::StopScan);
    vm->RegisterFunction("Close", PapyrusClass, (bool (*)(SFT))Tk::Close);
    vm->RegisterFunction("GetDevices", PapyrusClass, (std::vector<std::string>(*)(SFT))Tk::GetDevices);
    vm->RegisterFunction("GetDeviceCapabilities", PapyrusClass, (std::vector<std::string>(*)(SFT, std::string))Tk::GetDeviceCapabilities);
    vm->RegisterFunction("GetDeviceConnectionStatus", PapyrusClass, (std::string (*)(SFT, std::string))Tk::GetDeviceConnectionStatus);
    vm->RegisterFunction("Vibrate", PapyrusClass, (int (*)(SFT, int, float, std::vector<std::string>))Tk::Vibrate);
    vm->RegisterFunction("VibratePattern", PapyrusClass, (int (*)(SFT, std::string, float, std::vector<std::string>))Tk::VibratePattern);
    vm->RegisterFunction("Stop", PapyrusClass, (bool (*)(SFT, int))Tk::Stop);
    vm->RegisterFunction("StopAll", PapyrusClass, (bool (*)(SFT))Tk::StopAll);
    vm->RegisterFunction("PollEvents", PapyrusClass, (std::vector<std::string>(*)(SFT))Tk::PollEvents);
    vm->RegisterFunction("SettingsSet", PapyrusClass, (bool (*)(SFT, std::string, std::string))Tk::SettingsSet);
    vm->RegisterFunction("GetEnabled", PapyrusClass, (bool (*)(SFT, std::string))Tk::GetEnabled);
    vm->RegisterFunction("SetEnabled", PapyrusClass, (void (*)(SFT, std::string, bool))Tk::SetEnabled);
    vm->RegisterFunction("GetEvents", PapyrusClass, (std::vector<std::string>(*)(SFT, std::string))Tk::GetEvents);
    vm->RegisterFunction("SetEvents", PapyrusClass, (void (*)(SFT, std::string, std::vector<std::string>))Tk::SetEvents);
    vm->RegisterFunction("SettingsStore", PapyrusClass, (bool (*)(SFT))Tk::SettingsStore);
    vm->RegisterFunction("GetPatternNames", PapyrusClass, (std::vector<std::string>(*)(SFT, bool))Tk::GetPatternNames);
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
