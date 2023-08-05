
#include "plug/src/lib.rs.h"

#include <vector>
#include <string>

#define DllExport __declspec(dllexport)

// SKSE plugin loading requires us to declare all papyrus calls as dllexports
// otherwise they won't be found

DllExport bool ConnectAndScanForDevices(void*) {
    return tk_connect_and_scan(); 
}
DllExport bool Close(void*) {
    tk_close();
    return true;
}
DllExport std::vector<std::string> GetDeviceNames(void*) {
    auto names = tk_get_device_names();
    return std::vector<std::string>(names.begin(), names.end());
}
DllExport std::vector<std::string> GetDeviceCapabilities(void*, std::string device_name) {
    auto names = tk_get_device_capabilities(device_name);
    return std::vector<std::string>(names.begin(), names.end());
}
DllExport bool GetDeviceConnected(void*, std::string device_name) {
    return tk_get_device_connected(device_name);
}
DllExport bool Vibrate(void*, int speed, float time_sec, std::vector<std::string> events  /* TODO: Add extra call that uses events */) {
    return tk_vibrate(speed, time_sec);
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
DllExport std::vector<std::string> PollEvents(void*) {
    auto events = tk_poll_events();
    return std::vector<std::string>(events.begin(), events.end());
}
DllExport bool GetEnabled(void*, std::string device_name) {
    return tk_settings_get_enabled(device_name);
}
DllExport void SetEnabled(void*, std::string device_name, bool enabled) {
    tk_settings_set_enabled(device_name, enabled);
}
