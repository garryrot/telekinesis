ScriptName Tele_Devices extends Quest

Float property Version = 0.4 auto
Int property DevicesLength = 0 auto
Int property ScanTime = 30 auto
Bool property Reconnect = false auto

String[] Devices

Event OnInit()
    Log("Init Telekinesis v." + Version)
    Log("Enable devices in MCM to use them...")
    Devices = new String[32]
    Tele.Connect()
	Tele.ScanForDevices()
	RegisterForUpdate(5)
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
    Int j = 0
    While (j < names.Length)
        String name = names[j]
        If ! DeviceExists(name)
            DeviceNew(DevicesLength, name)
        EndIf
        j += 1
    EndWhile
EndEvent

Function DeviceNew(String id, String name)
    if (DevicesLength < 32) 
        Devices[DevicesLength] = name
    Else
        Log("ERROR too many devices: " + DevicesLength)
    EndIf
    DevicesLength += 1
EndFunction

String[] Function GetDevices()
	return Devices
EndFunction

Bool Function DeviceExists(String id)
	return Devices.Find(id) >= 0
EndFunction

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction