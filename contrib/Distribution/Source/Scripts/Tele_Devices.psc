ScriptName Tele_Devices extends Quest  

Int property DevicesLength = 0 auto
String[] Devices

Event OnInit()
    DevicesInit()
	StartScanningForDevices()
EndEvent

Function StartScanningForDevices()
	Log("Scanning for devices...")
	Tele.ScanForDevices()
	RegisterForUpdate(5)
EndFunction

Event OnUpdate()
	String[] evts = Tele.PollEvents()

	Int i = 0
	While (i < evts.Length) 
        String evt = evts[i]

		If StringUtil.Find(evt, "connected.") != -1
			String deviceId = StringUtil.Split(evt, "'")[ 1 ]
			if ! DeviceExists(deviceId)
				DeviceNew(deviceId, true, false)
			EndIf
            Log("Connected " + deviceId)
			SetConnected(deviceId, true)
		EndIf

		If StringUtil.Find(evt, "Removed.") != -1
			String deviceId = StringUtil.Split(evt, "'")[1]
			SetConnected(deviceId, true)
            Log("Disconnected " + deviceId)
		EndIf

		i += 1
	EndWhile
EndEvent

; -------------- DEVICES ---------------

Bool[] CanVibrate
Bool[] Connected
Bool[] Used
; Bool[] CanStroke

Function DevicesInit()
    Devices = new String[32]
    CanVibrate = new Bool[32]
    Connected = new Bool[32]
    Used = new Bool[32]
    ; CanStroke = new Bool[32]
EndFunction

Function DeviceNew(String id, Bool vibrate, Bool stroke)
    if (DevicesLength < 32) 
        Devices[DevicesLength] = id
        CanVibrate[DevicesLength] = vibrate
        Connected[DevicesLength] = true
        Used[DevicesLength] = true
        ; CanStroke[DevicesLength] = stroke
    Else
        Log( "ERROR too many devices: " + DevicesLength )
    EndIf
    DevicesLength += 1
EndFunction

Bool Function DeviceExists(String id)
	return Devices.Find(id) >= 0
EndFunction

String[] Function GetDevices()
	return Devices
EndFunction

Bool Function CanVibrate(String id)
    Int i = Devices.Find(id)
    If (i >= 0)
        return CanVibrate[i]
    EndIf
    Log( "ERROR GetDeviceActive " + id )
    return false
EndFunction

Function SetConnected(String id, Bool value)
    Int i = Devices.Find(id)
    If (i >= 0)
        Connected[i] = value
        return
    EndIf
    Log( "ERROR SetDeviceConnected " + id )
    return
EndFunction

Bool Function GetConnected(String id)
    Int i = Devices.Find(id)
    If (i >= 0)
        return Connected[i]
    EndIf
    Log( "ERROR GetDeviceConnected " + id )
    return false
EndFunction

Bool Function GetUsed(String id)
    Int i = Devices.Find(id)
    If (i >= 0)
        return Used[i]
    EndIf
    Log( "ERROR GetDeviceActive " + id )
    return false
EndFunction

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction
