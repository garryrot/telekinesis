#include <catch.hpp>
#include <filesystem>
#include <iostream>
#include <shellapi.h>
#include <windows.h>

#include <../test/Telekinesis.h>
#include <../plug/include/telekinesis_plug.h>


using namespace Catch::Matchers;
using namespace SKSE::log;
using namespace Telekinesis;

// rust ffi
TEST_CASE("telekinesis_plug/cbinding_returns_instance") {
    tk_connect();
    Sleep(50);
    tk_close();
}

TEST_CASE("telekinesis_plug/cbindings_enums_map_correctly") {
    REQUIRE(LogLevel::Debug == static_cast<LogLevel>(spdlog::level::debug));
    REQUIRE(LogLevel::Trace == static_cast<LogLevel>(spdlog::level::trace));
    REQUIRE(LogLevel::Info == static_cast<LogLevel>(spdlog::level::info));
    REQUIRE(LogLevel::Warn == static_cast<LogLevel>(spdlog::level::warn));
    REQUIRE(LogLevel::Error == static_cast<LogLevel>(spdlog::level::err));
}

TEST_CASE("telekinesis_plug/init_logger_writes_to_file_path") {
    auto tmp = std::format( "{}.log", std::tmpnam(NULL) );
    const char *cstr = tmp.c_str();
    REQUIRE(tk_init_logging(LogLevel::Trace, cstr));
    tk_connect();

    Sleep(50);
    std::ifstream t(tmp);
    std::stringstream buffer;
    buffer << t.rdbuf();
    REQUIRE_THAT(buffer.str(), ContainsSubstring("Buttplug Server Operating System Info"));
}