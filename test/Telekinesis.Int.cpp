#include <catch.hpp>
#include <windows.h>
#include <iostream>
#include <shellapi.h>

#include <../test/Telekinesis.h>
#include <../plug/include/telekinesis_plug.h>

using namespace Telekinesis;
using namespace Catch::Matchers;

std::string _wait_for_next_event() {
    int8_t *evt = NULL;
    do {
        std::cout << ".";
        Sleep(1000);
        evt = tk_try_get_next_event();
    } while (evt == NULL);
    std::cout << "Got it!";
    std::string message((char *)evt);
    tk_free_event(evt);
    return message;
}

// E2E tests block until a device is connected
TEST_CASE("telekinesis_plug/cbindings_vibrates_the_device_E2E") {
    // arrange
    tk_connect();
    tk_scan_for_devices();
    REQUIRE_THAT(_wait_for_next_event(), ContainsSubstring("Device"));

    // act
    REQUIRE(tk_vibrate_all(0.1) == true);
    Sleep(2000);
    REQUIRE(tk_stop_all() == true);
    Sleep(1000);
    REQUIRE(tk_vibrate_all_for(0.1, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(0.37, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(0.66, (float_t)1.5) == true);
    Sleep(750);
    REQUIRE(tk_vibrate_all_for(0.9, (float_t)1.5) == true);
    Sleep(2000);
    tk_close();  // device must be stopped now
}