ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Tele_Devices Property TeleDevices Auto

Event OnPlayerLoadGame()
	TeleDevices.Log("Tele_OnGameLoadObserver")
	Tele.ScanForDevices()
EndEvent
