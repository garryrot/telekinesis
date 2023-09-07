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
            If StringUtil.Find(evt, "connected") != -1
                LogConnection(evt)
                ; Event Connected
            ElseIf StringUtil.Find(evt, "removed") != -1
                LogConnection(evt)
                ; Event Removed
            ElseIf StringUtil.Find( evt, "Vibrated") != -1
                If StringUtil.Find( evt, "0%") != -1
                    ; Stop Vibrate
                Else
                    ; Start vibrate
                    LogEvent(evt)
                EndIf
            Else
                ; Other Event
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
    If Connects()
        Tele_Api.Close()
        ScanningForDevices = false
    EndIf
EndFunction

Int Function Vibrate(Int speed, Float duration_sec = -1.0)
    { Vibrate all specified devices for the given duration
        - speed (Percentage from 0=off to 100=full power)
        - duration_sec (Duratation in seconds. You can specify split seconds) 
      Returns an Int handle to stop the vibration early, see StopHandle(Int) }
    If Connects()
        String[] events = new String[1]
        return Tele_Api.Vibrate(InRange(speed, 0, 100), duration_sec, events)
    EndIf
    Trace("(Vibrate) speed='" + speed + "' duration='" + duration_sec + "' all")
    return -1
EndFunction

Int Function VibrateEvents(Int speed, Float duration_sec = -1.0, String[] events)
    { See vibrate(speed, duration_sec), but additionally filters for events
        - events (Vibrate devices that match the specified events)
      Returns an Int handle to stop the vibration early, see StopHandle(Int) }
    If Connects()
        return Tele_Api.Vibrate(InRange(speed, 0, 100), duration_sec, events)
    EndIf
    Trace("(Vibrate) speed='" + speed + " duration=" + duration_sec + " events=" + events)
    return -1
EndFunction

Function VibratePattern(String pattern, Float duration_sec = -1.0, String[] events)
    { Like VibrateEvents(speed, duration_sec, events) but instead of a speed,
        the vibration strength is regulated by the given funscript pattern
      Returns an Int handle to stop the vibration early, see StopHandle(Int) }
    If Connects()
        Tele_Api.VibratePattern(pattern, duration_sec, events)
    EndIf
    Trace("(Vibrate) pattern='" + pattern + " duration=" + duration_sec + " events=" + events)
EndFunction

Function StopHandle(Int handle)
    { Stops the vibration with the given handle early
      If you start an action with an infinite duration (<= 0), storing this handle
      and calling StopHandle at some point is a hard requirement.
      
      Note: Handles lose validity on each game restart, a call with a
      stale handle has no effect }
    If Connects()
        Tele_Api.Stop(handle)
    EndIf
    Trace("(Stop) stop handle=" + handle)
EndFunction

Function EmergencyStop()
    { Executes a global stop routine that will cause every single device to be
      stopped, and also abort all currently running patterns/vibrations. After 
      this call all existing handles are considered stale }
    If Connects()
        LogError("Emergency stop")
        Tele_Api.StopAll()
    EndIf
    Trace("(Stop) emergency stop")
EndFunction

; TODO Move StopEmergency here

Bool Function Connects()
    { Returns if the module connects at all (Connection is not Disable and the DLL was loaded) }
    return Tele_Api.Loaded() && ConnectionType != 2
EndFunction

String[] Function GetPatternNames(Bool vibrator)
    If Tele_Api.Loaded()
        return Tele_Api.GetPatternNames(vibrator)
    EndIf
    
    String[] defaultPatterns = new String[4]
    defaultPatterns[0] = "Tease-30s"
    defaultPatterns[1] = "Slow-Tease-30s"
    defaultPatterns[2] = "Sine"
    defaultPatterns[3] = "On-Off"
    return defaultPatterns
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
