#include "Config.h"
#include "Papyrus.h"
#include "../plug/include/telekinesis_plug.h"
#include "../plug/target/cxxbridge/plug/src/logging.rs.h"

using namespace RE::BSScript;
using namespace SKSE::log;
using namespace SKSE::stl;
using namespace SKSE;

namespace Telekinesis {
    std::string GetLogFile(std::string context) {
        auto path = log_directory();
        if (!path) {
            report_and_fail("Unable to lookup SKSE logs directory.");
        }
        return std::format("{}\\{}.{}.log", path->string(), PluginDeclaration::GetSingleton()->GetName(), context);
    } 

    void InitializeLogging() {
        std::shared_ptr<spdlog::logger> log;
        if (IsDebuggerPresent()) {
            log = std::make_shared<spdlog::logger>("Telekinesis", std::make_shared<spdlog::sinks::msvc_sink_mt>());
        } else {
            auto skseLog = GetLogFile("Skse");
            log = std::make_shared<spdlog::logger>(
                "Global", std::make_shared<spdlog::sinks::basic_file_sink_mt>(skseLog, true));
        }
        const auto& debugConfig = Telekinesis::Config::GetSingleton().GetDebug();
        log->set_level(debugConfig.GetLogLevel());
        log->flush_on(debugConfig.GetFlushLevel());

        spdlog::set_default_logger(std::move(log));
        spdlog::set_pattern("%Y-%m-%d %H:%M:%S.%e %5l [%t] [%s:%#] %v");

        const auto logLevel = debugConfig.GetLogLevel();
        auto logLevelStr = spdlog::level::to_string_view(logLevel);
        log::info("Log started. Logging level: '{}' ({}).", logLevel, logLevelStr);

        auto rustLog = GetLogFile("Plug");
        tk_init_logging(::rust::String(rustLog));
    }
}
