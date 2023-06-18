#include <stddef.h>

#include "Papyrus.h"
#include "../plug/include/telekinesis_plug.h"
#include "../plug/target/cxxbridge/plug/src/logging.rs.h"

using namespace RE::BSScript;
using namespace SKSE::log;
using namespace SKSE::stl;
using namespace SKSE;
using namespace Telekinesis;

namespace Telekinesis {
    void InitializePapyrus() {
        log::trace("Initializing Papyrus binding...");
        if (GetPapyrusInterface()->Register(Telekinesis::RegisterPapyrusCalls)) {
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
