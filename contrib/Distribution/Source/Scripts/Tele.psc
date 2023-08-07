ScriptName Tele hidden

; Sets up a new connection and starts scanning for devices. This will 
; automatically connect to every single bluetooth toy Buttplug.io knows about.
; This will find any device that is in-reach and coupled with your PCs bluetooth 
; adapater. Right now the scanning will continue indefinitely, so new
; devices might be added at any point in time automatically
Bool function ScanForDevices() global native

; Close the connection and dispose all structures. Telekinesis will not be
; usable from this point on. However, you may run ScanForDevices to create
; a new connection and start over again.
Bool function Close() global native

; Return a list of all connected device names
; - These names can be used to call specific devices
; - The list will include devices that have been previously and are now disconnected
String[] function GetDeviceNames() global native

; Return a list of all device capabilities
; Only the capability "Vibrate" is avaiable right now
String[] function GetDeviceCapabilities(String name) global native

; Returns whether the device with the given name is connected.
; Will also return false when the device does not exist
bool function GetDeviceConnected(String name) global native

; Vibrate all specified devices for the given duration
; - speed (Percentage from 0=off to 100=full power)
; - suration_sec (Duratation in seconds. You can specify split seconds)
; - devices (A list of device names, as returned by `GetDeviceNames`)
; To stop the vibration early, call this method with the same device list, specify speed=0
; and any duration
Bool function Vibrate(Int speed, Float duration_sec, String[] devices) global native

; DEPRECATED - FOR SAFETY REASONS, ONLY CONTROL DEVICES THAT ARE MANUALLY ACTIVATED
; Vibrate all devices that are currently connected (until stopped manually).
Bool function VibrateAll(Int speed) global native

; DEPRECATED - FOR SAFETY REASONS, ONLY CONTROL DEVICES THAT ARE MANUALLY ACTIVATED
; Calls to `TK_VibrateAll` or `VibrateAllFor` that happen before `duration_sec` 
; has ended will owerwrite `speed` and `duration_sec` to the new calls value.
Bool function VibrateAllFor(Int speed, Float duration_sec) global native

; Immediately stops all connected devices. This can be used for
; shutdown of ALL device actions before calling `Close` to assure that
; everything stopped.
Bool function StopAll() global native

; Returns a stream of messages that describe events in Tk
; - RETURN a string describing the Event or an empty Array if nothing happened
; Examples messages:
;  * "Device XY connected" (This device is connected and will be controlled)
;  * "Device XY disconnected" (This device is no longer connected and will be ignored)
;  * "Vibrating X devices..." (A vibrate command was successful and vibrated X devices)
; When multiple Mods consume this, they will steal each others events
String[] function PollEvents() global native


; Enable device by `name` in settings
; This settings is permanently stored
Bool function GetEnabled(String device_name) global native

; Enable device by `name` in settings
; This setting is permanently stored
function SetEnabled(String device_name, Bool enabled) global native

; Persists settings in Telekinesis.json
Bool function SettingsStore() global native
