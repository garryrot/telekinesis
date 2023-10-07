ScriptName Tele_Devices extends Quest
{
    Main control script for devices connected via Telekinesis
    ~ Use this API to control devices ~
}

Spell Property Tele_VibrateSpellStrong Auto
Spell Property Tele_VibrateSpellMedium Auto
Spell Property Tele_VibrateSpellWeak Auto
Spell Property Tele_Stop Auto

Int Property MajorVersion = 1 AutoReadOnly
Int Property MinorVersion = 2 AutoReadOnly
Int Property PatchVersion = 0 AutoReadOnly
String Property Revision = "" AutoReadOnly

String Property Version
    String Function Get()
        return MajorVersion + "." + MinorVersion + "." + PatchVersion + " " + Revision
    EndFunction
EndProperty

Bool Property LogDeviceConnects = true Auto
Bool Property LogDeviceEvents = false Auto
Bool Property LogDebugEvents = false Auto

Bool Property ScanningForDevices = false Auto
Int Property ConnectionType = 0 Auto
String _ErrorText
String Property ConnectionErrorDetails
    String Function Get()
        If GetConnectionStatus() == "Failed"
            return _ErrorText
        EndIf
        return ""
    EndFunction
EndProperty

String Property WsPort = "12345" Auto
String Property WsHost = "127.0.0.1" Auto

Event OnInit()
    RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    If Connects()
        String[] evts = Tele_Api.PollEvents()
        Int i = 0
        While (i < evts.Length)
            String[] evt = StringUtil.Split(evts[i], "|")
            String type = evt[0]
            If type == "DeviceAdded"
                LogConnection("Device '" + evt[1] + "' connected")
            ElseIf type == "DeviceRemoved"
                LogConnection("Device '" + evt[1] + "' disconnected")
            ElseIf type == "Connected"
                LogDebug("Connection success")
            ElseIf type == "ConnectionFailure"
                If ConnectionType == 0
                    _ErrorText = "In-Process Failure"
                ElseIf ConnectionType == 1
                    _ErrorText = "Intiface Connection Failure. Port: " + WsPort + " Host: " + WsHost
                Else
                    _ErrorText = ""
                EndIf
                LogError(_ErrorText)
            ElseIf type == "DeviceEvent"
                LogEvent(evt)
                ; LogEvent(evt[1] + " for " + evt[2] + "s at " + evt[3] + " (" +  evt[4] + ") on " + evt[5] + " events: " + evt[6])
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

Int Function VibratePattern(String pattern, Float duration_sec = -1.0, String[] events)
    { Like VibrateEvents(speed, duration_sec, events) but instead of a speed,
        the vibration strength is regulated by the given funscript pattern
      Returns an Int handle to stop the vibration early, see StopHandle(Int) }
    If Connects()
        return Tele_Api.VibratePattern(pattern, duration_sec, events)
    EndIf
    return -1
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

Function Reconnect()
    { Stops the current connection, resets the entire backend state and 
      restarts with the configured connection settings.
      
      NOTE: Handles will lose validity }
    If ConnectionType == 0
        Tele_Api.SettingsSet("connection.inprocess", "")
    ElseIf ConnectionType == 1
        Tele_Api.SettingsSet("connection.websocket", WsHost + ":" + WsPort)
    EndIf
    Tele_Api.SettingsStore()
    Utility.Wait(0.5)
    Disconnect()
    Utility.Wait(3)
    If (ConnectionType != 2)
        ConnectAndScanForDevices()
    EndIf
EndFunction

Bool Function Connects()
    { Returns if the plugin connects to a backend
      (true if dll is loadable AND backen is configure to connect) }
    return Tele_Api.Loaded() && ConnectionType != 2
EndFunction

String Function GetConnectionStatus()
    If ! Tele_Api.Loaded()
        return "Not Connected"
    EndIf
    return Tele_Api.GetConnectionStatus()
EndFunction

String[] Function GetPatternNames(Bool vibrator)
    If Tele_Api.Loaded()
        return Tele_Api.GetPatternNames(vibrator)
    EndIf
    
    String[] defaultPatterns = new String[4]
    defaultPatterns[0] = "01_Tease"
    defaultPatterns[1] = "02_Cruel-Tease"
    defaultPatterns[2] = "03_Wub-Wub-Wub"
    defaultPatterns[3] = "30_Sawtooth"
    defaultPatterns[4] = "30_Sawtooth-Fast"
    defaultPatterns[5] = "31_Sawtooth-Fast"
    defaultPatterns[6] = "35_On-Off"
    defaultPatterns[7] = "36_On-Off-Fast"
    defaultPatterns[8] = "40_Square"
    defaultPatterns[9] = "50_Sine"
    return defaultPatterns
EndFunction

String Function GetRandomPattern(Bool vibrator)
    String[] patterns = GetPatternNames(vibrator)
    return patterns[Utility.RandomInt(0, patterns.Length - 1)]
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
