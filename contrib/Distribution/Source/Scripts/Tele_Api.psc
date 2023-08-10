ScriptName Tele_Api hidden

; Communication API with Telekinesis.DLL
; DO NOT USE DIRECTLY, use Tele_Devices instead to respect user settings

; Sets up a new connection
Bool function Connect() global native

; This will automatically connect to every single bluetooth toy Buttplug.io knows
; about. This will find any device that is in-reach and coupled with your PCs
; bluetooth adapater. Continues until StopScan is called
Bool function ScanForDevices() global native

; Stops any ongoing device scans
Bool function StopScan() global native

; Close the connection and dispose all structures. Telekinesis will not be
; usable from this point on. However, you may run ScanForDevices to create
; a new connection and start over again.
Bool function Close() global native

; Return a list of all connected device names
; - These names can be used to call specific devices
; - The list includes devices that have been connected in previous sessions
String[] function GetDevices() global native

; Return a list of all device capabilities
; Only the capability "Vibrate" is avaiable right now
String[] function GetDeviceCapabilities(String name) global native

; Returns whether the device with the given name is connected.
; Will also return false when the device does not exist
bool function GetDeviceConnected(String name) global native

; Vibrate all enabled devices
Bool function Vibrate(Int speed, Float duration_sec) global native

; Vibrate all enabled devices by events
Bool function VibrateEvents(Int speed, Float duration_sec, String[] events) global native

; Immediately stops all connected devices. This can be used for
; shutdown of ALL device actions before calling `Close` to assure that
; everything stopped.
Bool function StopAll() global native

; Returns a stream of messages that describe events in Tk
; - RETURN a string describing the Event or an empty Array if nothing happened
; Examples messages:
;  * "Device XY connected" (This device is connected and will be controlled)
;  * "Device XY disconnected" (This device is no longer connected and will be ignored)
;  * "Vibrated X devices..." (A vibrate command was successful and vibrated X devices)
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
