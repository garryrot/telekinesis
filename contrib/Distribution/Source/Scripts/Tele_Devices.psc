ScriptName Tele_Devices extends Quest

Int property MajorVersion = 0 autoReadOnly
Int property MinorVersion = 4 autoReadOnly

Int property ScanTime = 30 auto
Int property DevicesLength = 0 auto
String[] property Devices auto

Bool property LogDeviceConnects = true auto
Bool property LogDeviceEvents = false auto
Bool property LogDebugEvents = false auto

Bool property ScanningForDevices = false auto

Event OnInit()
    Notify("Telekinesis v" + MajorVersion + "." + MinorVersion + ": Enable connected devices in MCM for usage...")
    Devices = new String[32]
    Connect()
	RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    String[] evts = Tele.PollEvents()
    
	Int i = 0
	While (i < evts.Length)
        String evt = evts[i]
        Debug.MessageBox("Parsing MsgBox " + evt)
        If StringUtil.Find(evt, "connected") != -1 || StringUtil.Find(evt, "removed") != -1
            LogConnection(evt)
            UpdateDevices()
        ElseIf StringUtil.Find( evt, "Vibrated") != -1
            LogEvent(evt)
        Else
            LogDebug(evt)
        EndIf
		i += 1
	EndWhile
EndEvent

Function Connect()
    Tele.Connect()
	Tele.ScanForDevices()
    ScanningForDevices = true
EndFunction

Function Disconnect() 
    Tele.Close()
    ScanningForDevices = false
EndFunction

; Privates

Function UpdateDevices()
    String[] names = Tele.GetDeviceNames()
    Int j = 0
    While (j < names.Length)
        String name = names[j]
        If Devices.Find(name) < 0
            DeviceNew(DevicesLength, name)
        EndIf
        j += 1
    EndWhile
EndFunction

Function DeviceNew(String id, String name)
    if (DevicesLength < 32) 
        Devices[DevicesLength] = name
    Else
        Notify("ERROR too many devices: " + DevicesLength)
    EndIf
    DevicesLength += 1
EndFunction

; Logging

Function Notify(string textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction

Function Trace(string textToTrace)
	Debug.Trace("[Tele] " + textToTrace)
EndFunction

Function LogConnection(string textToPrint)
    Trace(textToPrint)
    If LogDeviceConnects
        Notify(textToPrint)
    EndIf
EndFunction

Function LogEvent(string textToPrint)
    Trace(textToPrint + " LogDeviceEvents " + LogDeviceEvents)
    If LogDeviceEvents
        Notify(textToPrint)
    EndIf
EndFunction

Function LogDebug(string textToPrint)
    Trace(textToPrint)
    If LogDebugEvents
        Notify(textToPrint)
    EndIf
EndFunction
