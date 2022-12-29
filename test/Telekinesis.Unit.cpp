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

// papyrus native function implementations
TEST_CASE("Connection/Connecting_Works") {
    Tk_ConnectAndScanForDevices(NULL);
    Sleep(10);
    Tk_Close(NULL);
}

TEST_CASE("Controlls/NotConnected_ReturnFalse") { 
    REQUIRE_FALSE(Tk_StartVibrateAll(NULL, 0.0));
    REQUIRE_FALSE(Tk_StopVibrateAll(NULL));
    REQUIRE_FALSE(Tk_Close(NULL));
}

TEST_CASE("Controlls/Connected_ReturnTrue") {
    Tk_ConnectAndScanForDevices(NULL);
    REQUIRE(Tk_StartVibrateAll(NULL, 0.0) >= 0);
    REQUIRE(Tk_StopVibrateAll(NULL) >= 0);
    Sleep(10);
    Tk_Close(NULL);
}

TEST_CASE("Controlls/ConnectAndDisconnect_ReturnsFalse") {
    Tk_ConnectAndScanForDevices(NULL);
    Sleep(10);
    Tk_Close(NULL);
    REQUIRE_FALSE(Tk_StartVibrateAll(NULL, 0.0));
    REQUIRE_FALSE(Tk_StopVibrateAll(NULL));
    REQUIRE_FALSE(Tk_Close(NULL));
}

TEST_CASE("telekinesis_plug/cbinding_returns_instance") {
    void *tk = tk_connect();
    REQUIRE(tk != NULL);
    tk_close(tk);
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

    Sleep(10);
    std::ifstream t(tmp);
    std::stringstream buffer;
    buffer << t.rdbuf();
    REQUIRE_THAT(buffer.str(), ContainsSubstring("Buttplug Server Operating System Info"));
}
