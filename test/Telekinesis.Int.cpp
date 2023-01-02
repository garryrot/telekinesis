#include <catch.hpp>
#include <windows.h>
#include <iostream>
#include <shellapi.h>

#include <../test/Telekinesis.h>
#include <../plug/include/telekinesis_plug.h>

using namespace Telekinesis;
using namespace Catch::Matchers;

// Integration tests that require a device to be connected
TEST_CASE("telekinesis_plug/cbindings_vibrates_the_device_E2E") {
    void *tk = tk_connect();
    REQUIRE(tk != NULL);
    tk_scan_for_devices(tk);

    int8_t *evt = NULL;
    do {
        std::cout << ".";
        Sleep(1000);
        evt = tk_try_get_next_event(tk);
    } while (evt == NULL);
    std::cout << "Got it!";
    std::string message((char *)evt);
    REQUIRE_THAT(message, ContainsSubstring("Device"));

    tk_free_event(tk, evt);
    float_t speed = 0.1;
    REQUIRE(tk_vibrate_all(tk, speed) == true);
    Sleep(5000);
    REQUIRE(tk_stop_all(tk) == true);
    tk_close(tk);
}