scriptName TK_Telekinesis hidden

; Sets up a new connection and starts scanning for devices. This will 
; automatically connect to every single bluetooth toy Buttplug.io knows about.
; This will find any device that is in-reach and coupled with your PCs bluetooth 
; adapater. Right now the scanning will continue indefinitely, so new
; devices might be added at any point in time automatically
Bool function TK_ScanForDevices() global native

; Returns a stream of messages that describe the status devices of devices
; - RETURN a string describing the Event or a default String ("") if nothing happened
; Examples:
;  * "" (Nothing happened)
;  * "Device XY connected" (This device is connected and will be controlled)
;  * "Device XY disconnected" (This device should no longer get vibrated)
;  
; DISCLAIMER: The implementation is a really shitty hackjob right now and will
; only return one event at a time (and even drop some). When multiple
; Mods consume this, they will steal each others events
string function Tk_PollEvents() global native

; Vibrate all devices that are currently connected.
; Speed is any float between 0.0(=off) and 1.0 (=full power)
; TK_StartVibrateAll( 0 ) should also be used for stopping the vibration,
; as it provides a smoother experience than TK_StopVibrateAll
; TODO: Rename to Tk_SetVibrationSpeed
Int function TK_StartVibrateAll(Float speed) global native

; Immediately stops all connected devices. This should be used for
; shutdown, before calling Tk_Close to assure that everything stopped.
;
; NOTE: You could also use it to stop device vibration manually, but I've
; experienced that it will cause weird behavior: Some devices still store
; the last vibration speed, so 
; TODO: Rename to Tk_StopAll
Int function TK_StopVibrateAll() global native

; Close the connection and dispose all structures. Telekinesis will not be
; usable from this point on. However, you may run TK_ScanForDevices to
; start over again.
Bool function Tk_Close() global native
