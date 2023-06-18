
#include "plug/src/lib.rs.h"

#include <vector>
#include <string>

#define DllExport __declspec(dllexport)

// SKSE plugin loading requires us to declare all papyrus calls as
// dllexports otherwise they won't be found

DllExport bool ConnectAndScanForDevices(void*) {
    return tk_connect_and_scan(); 
}
DllExport bool VibrateAll(void*, int speed) { 
    return tk_vibrate_all(speed);
}
DllExport bool VibrateAllFor(void*, int speed, float time_sec) {
    return tk_vibrate_all_for(speed, time_sec);
}
DllExport bool StopAll(void*) {
    return tk_stop_all(); 
}
DllExport bool Close(void*) {
    tk_close();
    return true;
}
DllExport std::vector<std::string> PollEvents(void*) {
    auto events = tk_poll_events();
    return std::vector<std::string>(events.begin(), events.end());
}
