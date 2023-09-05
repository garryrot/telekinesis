ScriptName Tele_Devices extends Quest
{
    Main control script for devices connected via Telekinesis
    ~ Use this API to control devices ~
}

Spell Property Tele_VibrateSpellStrong auto
Spell Property Tele_VibrateSpellMedium auto
Spell Property Tele_VibrateSpellWeak auto
Spell Property Tele_Stop auto

Int Property MajorVersion = 1 autoReadOnly
Int Property MinorVersion = 1 autoReadOnly
Int Property PatchVersion = 0 autoReadOnly
String Property Revision = "" autoReadOnly

String Property Version
    String Function Get()
        return MajorVersion + "." + MinorVersion + "." + PatchVersion + " " + Revision
    EndFunction
EndProperty

Bool Property LogDeviceConnects = true auto
Bool Property LogDeviceEvents = false auto
Bool Property LogDebugEvents = false auto

Bool Property ScanningForDevices = false auto
Int Property ConnectionType = 0 auto ; In-Process

Event OnInit()
    RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    If Connects()
        String[] evts = Tele_Api.PollEvents()
        Int i = 0
        While (i < evts.Length)
            String evt = evts[i]
            If StringUtil.Find(evt, "connected") != -1 || StringUtil.Find(evt, "removed") != -1
                LogConnection(evt)
            ElseIf StringUtil.Find( evt, "Vibrated") != -1 && StringUtil.Find( evt, "0%") >= 0
                LogEvent(evt)
            Else
                LogDebug(evt)
            EndIf
            i += 1
        EndWhile
    EndIf
EndEvent

; Public

Function ConnectAndScanForDevices()
    { Starts a new conenction to the backend (if not disabled) }
    If Connects()
        Tele_Api.Connect()
        Tele_Api.ScanForDevices()
        ScanningForDevices = true
    EndIf
EndFunction

Function Disconnect()
    { Closes the connection to the backend (if not disabled) }
    Tele_Api.Close()
    ScanningForDevices = false
EndFunction

Function Vibrate(Int speed, Float duration_sec = -1.0)
    { Vibrate all specified devices for the given duration
        - speed (Percentage from 0=off to 100=full power)
        - duration_sec (Duratation in seconds. You can specify split seconds) }
    If Connects()
        String[] events = new String[1]
        Tele_Api.VibrateEvents(InRange(speed, 0, 100), duration_sec, events)
    EndIf
    Trace("(Vibrate) speed='" + speed + "' duration='" + duration_sec + "' all")
EndFunction

Function VibrateEvents(Int speed, Float duration_sec = -1.0, String[] events)
    { See vibrate(speed, duration_sec), but additionally filters for events
        - events (Vibrate devices that match the specified events) }
    If Connects()
        Tele_Api.VibrateEvents(InRange(speed, 0, 100), duration_sec, events)
    EndIf
    Trace("(Vibrate) events speed='" + speed + " duration=" + duration_sec + " events=" + events)
EndFunction

Function VibratePattern(String pattern, Float duration_sec = -1.0, String[] events)
    { Like VibrateEvents(speed, duration_sec, events) but instead of a speed,
      the vibration strength is regulated by the given funscript pattern }
    If Connects()
        Tele_Api.VibratePattern(pattern, duration_sec, events)
    EndIf
    Trace("(Vibrate) pattern pattern='" + pattern + " duration=" + duration_sec + " events=" + events)
EndFunction

Function VibrateStopAll()
    String[] events = new String[1]
    VibrateStop(events)
EndFunction

Function VibrateStop(String[] events)
    { Should be called in conjunction with infinite duration vibrations (duration -1)
      to stop the vibration. For simultaneous vibrations, this reduces the references 
      to the selected devices by one, and only stops when the last vibration ends }
    If Connects()
        Tele_Api.VibrateStop(events)
    EndIf
    Trace("(Vibrate) stop all")
EndFunction

; TODO Move StopEmergency here

Bool Function Connects()
    { Returns if the module connects at all (Connection is not Disable and the DLL was loaded) }
    return Tele_Api.Loaded() && ConnectionType != 2
EndFunction

; Utility

Int Function InRange(Int value, Int min, Int max)
    { Assures that value is within the given boundaries }
    If value > max
        value = max
    EndIf 
    If value < min
        value = min
    EndIf
    return value
EndFunction

; Logging

Function Notify(string msg)
    { Telekinesis Notification on top left }
    Debug.Notification("[Tele] " + msg)
EndFunction

Function Trace(string msg, Int level = 0)
    { Telekinesis log to papyrus log (with `level`) }
    Debug.Trace("[Tele] " + msg, level)
EndFunction

Function LogError(string msg)
    { Log Telekinesis Error }
    Notify(msg)
    Trace(msg, 2)
EndFunction

Function LogConnection(string msg)
    { Log Telekinesis Connection Event (connect/disconnect) }
    Trace(msg)
    If LogDeviceConnects
        Notify(msg)
    EndIf
EndFunction

Function LogEvent(string msg)
    { Log Telekinesis Event (N devices vibrated, etc.) }
    Trace(msg + " LogDeviceEvents " + LogDeviceEvents)
    If LogDeviceEvents
        Notify(msg)
    EndIf
EndFunction

Function LogDebug(string msg)
    { Log Telekinesis debug level event }
    Trace(msg)
    If LogDebugEvents
        Notify(msg)
    EndIf
EndFunction
