ScriptName Tele_Devices extends Quest

Int property DevicesLength = 0 auto
Int property ScanTime = 30 auto
Bool property Reconnect = false auto

String[] Devices
Bool[] Used

Event OnInit()
    InitDevices()
	ScanForDevices()
	RegisterForUpdate(5)
    Log("Please enable connected devices in MCM to use them...")
EndEvent

Event OnUpdate()
    String[] evts = Tele.PollEvents()

	Int i = 0
	While (i < evts.Length)
        String evt = evts[i]
        Log(evt)
		i += 1
	EndWhile

    String[] names = Tele.GetDeviceNames()

    Int j = 0;
    While (j < names.Length)
        String name = names[j]
        If ! DeviceExists(name)
            DeviceNew(DevicesLength, name)
        EndIf
        j += 1
    EndWhile
EndEvent

Function ScanForDevices()
	Log("Start scanning for devices (" + ScanTime + "s...)")
	Tele.ScanForDevices()
EndFunction

Function InitDevices()
    Devices = new String[32]
    Used = new Bool[32]
EndFunction

Function DeviceNew(String id, String name)
    if (DevicesLength < 32) 
        Devices[DevicesLength] = name
        Used[DevicesLength] = false
    Else
        Log("ERROR too many devices: " + DevicesLength)
    EndIf
    DevicesLength += 1
EndFunction

String[] Function GetUsedDevices()
    Int i = 0
    Int count = 0
    String[] UsedDevices = new String[32]
    While i < DevicesLength
        If Used[i]
            UsedDevices[count] = Devices[i]
            count += 1
        EndIf
        i += 1
    EndWhile
    return UsedDevices
EndFunction

String[] Function GetDevices()
	return Devices
EndFunction

Bool Function IsUsed(String name)
    Int i = Devices.Find(name)
    If (i >= 0)
        return Used[i]
    EndIf
    Log("ERROR GetDeviceActive " + name)
    return false
EndFunction

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction

Bool Function GetUsed(Int index)
    return Used[index]
EndFunction

Function SetUsed(Int index, Bool isUsed)
    Used[index] = isUsed
EndFunction

Bool Function DeviceExists(String id)
	return Devices.Find(id) >= 0
EndFunction
