#include <catch.hpp>
#include <windows.h>
#include <iostream>
#include <shellapi.h>

#include <../test/Telekinesis.h>
#include <../plug/include/telekinesis_plug.h>

using namespace Telekinesis;
using namespace Catch::Matchers;

std::string _wait_for_next_event(void *tk) {
    int8_t *evt = NULL;
    do {
        std::cout << ".";
        Sleep(1000);
        evt = tk_try_get_next_event(tk);
    } while (evt == NULL);
    std::cout << "Got it!";
    std::string message((char *)evt);
    tk_free_event(tk, evt);
    return message;
}

// E2E tests block until a device is connected
TEST_CASE("telekinesis_plug/cbindings_vibrates_the_device_E2E") {
    // arrange
    void *tk = tk_connect();
    REQUIRE(tk != NULL);
    tk_scan_for_devices(tk);
    REQUIRE_THAT(_wait_for_next_event(tk), ContainsSubstring("Device"));

    // act
    REQUIRE(tk_vibrate_all(tk, 0.1) == true);
    Sleep(2000);
    REQUIRE(tk_stop_all(tk) == true);
    Sleep(1000);
    REQUIRE(tk_vibrate_all_for(tk, 0.1, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(tk, 0.37, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(tk, 0.66, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(tk, 0.9, (float_t)1.5) == true);
    Sleep(2000);
    tk_close(tk); // device must be stopped now
}