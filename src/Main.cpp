#include "Config.h"
#include "Papyrus.h"
#include "Logging.h"

#include <stddef.h>

using namespace RE::BSScript;
using namespace Telekinesis;
using namespace SKSE;

namespace Telekinesis {
    void InitializePapyrus() {
        log::trace("Initializing Papyrus binding...");
        if (GetPapyrusInterface()->Register(Telekinesis::RegisterPapyrusCalls)) {
            log::debug("Papyrus functions bound.");
        } else {
            stl::report_and_fail("Failure to register Papyrus bindings.");
        }
    }
}

SKSEPluginLoad(const LoadInterface* skse) {
    InitializeLogging();

    auto* plugin = PluginDeclaration::GetSingleton();
    auto version = plugin->GetVersion();
    log::info("{} {} is loading...", plugin->GetName(), version);

    Init(skse);
    InitializePapyrus();

    log::info("{} has finished loading.", plugin->GetName());
    return true;
}
