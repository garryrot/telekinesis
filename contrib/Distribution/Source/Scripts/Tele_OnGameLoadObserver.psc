ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Tele_Devices Property TeleDevices Auto

Event OnPlayerLoadGame()
	TeleDevices.LogDebug("OnPlayerLoadGame")
	TeleDevices.Connect()
EndEvent
