bool ConnectAndScanForDevices(void*);
bool VibrateAll(void*, int speed);
bool VibrateAllFor(void*, int speed, float time_sec);
bool StopAll(void*);
bool Close(void*);
std::vector<std::string> PollEvents(void*);
