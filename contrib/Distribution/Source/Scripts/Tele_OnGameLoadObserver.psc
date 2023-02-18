ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Tele_Devices Property TeleDevices Auto

Event OnPlayerLoadGame()
	TeleDevices.Log("Tele_OnGameLoadObserver")
    int i = 0
    String[] devices = TeleDevices.GetDevices()
    While (i < TeleDevices.DevicesLength)
        TeleDevices.SetConnected(devices[i], false)
        i += 1
    EndWhile
	Tele.ScanForDevices()
EndEvent
