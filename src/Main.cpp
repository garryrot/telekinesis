#include <stddef.h>

#include "../plug/target/cxxbridge/plug/src/lib.rs.h"
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

namespace Tele {
    static ::rust::Box tk = tk_new();
    bool ApiLoaded(SFT) { 
        return true;
    }
    bool Cmd(SFT, std::string cmd) { 
        return (*tk).tk_cmd(cmd); 
    }
    bool Cmd_1(SFT, std::string cmd, std::string arg0) { 
        return (*tk).tk_cmd_1(cmd, arg0);
    }
    bool Cmd_2(SFT, std::string cmd, std::string arg0, std::string arg1) { 
        return (*tk).tk_cmd_2(cmd, arg0, arg1);
    }
    std::string Qry_Str(SFT, std::string qry) { 
        return (std::string) (*tk).tk_qry_str(qry);
    }
    std::string Qry_Str_1(SFT, std::string qry, std::string arg0) { 
        return (std::string) (*tk).tk_qry_str_1(qry, arg0); 
    }
    std::vector<std::string> Qry_Lst(SFT, std::string qry) {
        auto list = (*tk).tk_qry_lst(qry);
        return std::vector<std::string>(list.begin(), list.end());
    }
    std::vector<std::string> Qry_Lst_1(SFT, std::string qry, std::string arg0) {
        auto list = (*tk).tk_qry_lst_1(qry, arg0);
        return std::vector<std::string>(list.begin(), list.end());
    }
    bool Qry_Bool(SFT, std::string qry) { 
        return (*tk).tk_qry_bool(qry);
    }
    bool Qry_Bool_1(SFT, std::string qry, std::string arg0) { 
        return (*tk).tk_qry_bool_1(qry, arg0); 
    }
    int Tele_Control(SFT, std::string actuator, int speed, float time_sec, std::string pattern,
                     std::vector<std::string> events) {
        return (*tk).tk_control(actuator, speed, time_sec, pattern, events);
    }
    bool Tele_Stop(SFT, int handle) { 
        return (*tk).tk_stop(handle); 
    }
}

bool RegisterPapyrusCalls(IVirtualMachine* vm) {
    vm->RegisterFunction("Loaded", PapyrusClass, Tele::ApiLoaded);
    vm->RegisterFunction("Cmd", PapyrusClass, Tele::Cmd);
    vm->RegisterFunction("Cmd_1", PapyrusClass, Tele::Cmd_1);
    vm->RegisterFunction("Cmd_2", PapyrusClass, Tele::Cmd_2);
    vm->RegisterFunction("Qry_Str", PapyrusClass, Tele::Qry_Str);
    vm->RegisterFunction("Qry_Str_1", PapyrusClass, Tele::Qry_Str_1);
    vm->RegisterFunction("Qry_Lst", PapyrusClass, Tele::Qry_Lst);
    vm->RegisterFunction("Qry_Lst_1", PapyrusClass, Tele::Qry_Lst_1);
    vm->RegisterFunction("Qry_Bool", PapyrusClass, Tele::Qry_Bool);
    vm->RegisterFunction("Qry_Bool_1", PapyrusClass, Tele::Qry_Bool_1);
    vm->RegisterFunction("Tele_Control", PapyrusClass, Tele::Tele_Control);
    vm->RegisterFunction("Tele_Stop", PapyrusClass, Tele::Tele_Stop);
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
