ScriptName Tele_Devices extends Quest

Int property MajorVersion = 0 autoReadOnly
Int property MinorVersion = 4 autoReadOnly

Int property ScanTime = 30 auto

Bool property LogDeviceConnects = true auto
Bool property LogDeviceEvents = false auto
Bool property LogDebugEvents = false auto

Bool property ScanningForDevices = false auto

Event OnInit()
    Notify("Telekinesis v" + MajorVersion + "." + MinorVersion + ": Enable connected devices in MCM for usage...")
    Connect()
	RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    String[] evts = Tele_Api.PollEvents()
	Int i = 0
	While (i < evts.Length)
        String evt = evts[i]
        If StringUtil.Find(evt, "connected") != -1 || StringUtil.Find(evt, "removed") != -1
            LogConnection(evt)
        ElseIf StringUtil.Find( evt, "Vibrated") != -1
            LogEvent(evt)
        Else
            LogDebug(evt)
        EndIf
		i += 1
	EndWhile
EndEvent

; Private

Function Connect()
    Tele_Api.Connect()
	Tele_Api.ScanForDevices()
    ScanningForDevices = true
EndFunction

Function Disconnect() 
    Tele_Api.Close()
    ScanningForDevices = false
EndFunction

; Public

; Vibrate all specified devices for the given duration
; - speed (Percentage from 0=off to 100=full power)
; - duration_sec (Duratation in seconds. You can specify split seconds)
; - events (Vibrate devices that match the specified events)
Function Vibrate(Int speed, Float duration_sec, String[] events = None)
    If events == None
        Tele_Api.Vibrate(speed, duration_sec)
        Trace("(Vibrate) speed='" + speed + "' duration='" + duration_sec + "' all")
    Else
        Tele_Api.VibrateEvents(speed, duration_sec, events)
        Trace("(Vibrate) events speed='" + speed + " duration=" + duration_sec + " events=" + events)
    EndIf
EndFunction

; Stop all vibrators.
; - events (If events are specified, stop vibrators associated with the given event)
Function StopVibrate(String[] events = None)
    If events == None
        Tele_Api.Vibrate(0, 0.1)
        Trace("(Vibrate) stop all")
    Else
        Tele_Api.VibrateEvents(0, 0.1, events)
        Trace("(Vibrate) stop events=" + events)
    EndIf
EndFunction

; Logging

Function Notify(string msg)
	Debug.Notification("[Tele] " + msg)
EndFunction

Function Trace(string msg, Int level = 0)
	Debug.Trace("[Tele] " + msg, level)
EndFunction

Function LogError(string msg)
    Notify(msg)
    Trace(msg, 2)
EndFunction

Function LogConnection(string msg)
    Trace(msg)
    If LogDeviceConnects
        Notify(msg)
    EndIf
EndFunction

Function LogEvent(string msg)
    Trace(msg + " LogDeviceEvents " + LogDeviceEvents)
    If LogDeviceEvents
        Notify(msg)
    EndIf
EndFunction

Function LogDebug(string msg)
    Trace(msg)
    If LogDebugEvents
        Notify(msg)
    EndIf
EndFunction
