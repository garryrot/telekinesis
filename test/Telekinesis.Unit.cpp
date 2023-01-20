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
    ConnectAndScanForDevices(NULL);
    Sleep(10);
    Close(NULL);
}

TEST_CASE("Controlls/NotConnected_ReturnFalse") { 
    REQUIRE_FALSE(VibrateAll(NULL, 0.0));
    REQUIRE_FALSE(VibrateAllFor(NULL, 0.0, 0.0));
    REQUIRE_FALSE(StopAll(NULL));
    REQUIRE_FALSE(Close(NULL));
}

TEST_CASE("Controlls/Connected_ReturnTrue") {
    ConnectAndScanForDevices(NULL);
    REQUIRE(VibrateAll(NULL, 0.0) == true);
    REQUIRE(StopAll(NULL) == true);
    Sleep(10);
    Close(NULL);
}

TEST_CASE("Controlls/ConnectAndDisconnect_ReturnsFalse") {
    ConnectAndScanForDevices(NULL);
    Sleep(10);
    Close(NULL);
    REQUIRE_FALSE(VibrateAll(NULL, 0.0));
    REQUIRE_FALSE(StopAll(NULL));
    REQUIRE_FALSE(Close(NULL));
}

TEST_CASE("Papyrus/poll_events_nothing_happened_returns_empty_list") {
    ConnectAndScanForDevices(NULL);
    auto list = PollEventsStdString();
    Close(NULL);
}

TEST_CASE("Papyrus/poll_commands_produce_1_event") {
    ConnectAndScanForDevices(NULL);
    VibrateAll(NULL, 0.0);
    Sleep(1);
    auto list = PollEventsStdString();
    REQUIRE(list.size() == 1);
    Close(NULL);
}

TEST_CASE("Papyrus/poll_events_2_commands_produce_2_events") {
    ConnectAndScanForDevices(NULL);
    VibrateAll(NULL, 0.0);
    VibrateAllFor(NULL, 0.0, 0.0); // delay emits 2 events
    Sleep(10);
    auto list = PollEventsStdString();
    REQUIRE(list.size() == 3);
    Close(NULL);
}

TEST_CASE("Papyrus/poll_events_200_commands_produce_128_events") {
    ConnectAndScanForDevices(NULL);
    for (size_t i = 0; i < 200; i++) {
        StopAll(NULL);
    }
    Sleep(2);
    auto list = PollEventsStdString();
    REQUIRE(list.size() == 128);
    Close(NULL);
}

// rust ffi
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
