namespace Tk
{
    bool Connect(void*);
    bool ScanForDevices(void*);
    bool StopScan(void*);
    bool Close(void*);
    std::vector<std::string> GetDevices(void*);
    std::vector<std::string> GetDeviceCapabilities(void*, std::string name);
    bool GetDeviceConnected(void*, std::string name);
    int Vibrate(void*, int speed, float time_sec, std::vector<std::string> events);
    int VibratePattern(void*, std::string pattern_name, float time_sec, std::vector<std::string> events);
    bool Stop(void*, int handle);
    bool StopAll(void*);
    std::vector<std::string> PollEvents(void*);
    bool SettingsSet(void*, std::string key, std::string value);
    bool GetEnabled(void*, std::string name);
    void SetEnabled(void*, std::string name, bool enabled);
    std::vector<std::string> GetEvents(void*, std::string device_name);
    void SetEvents(void*, std::string device_name, std::vector<std::string> events);
    bool SettingsStore(void*);
    std::vector<std::string> GetPatternNames(void*, bool vibration_patterns);
}