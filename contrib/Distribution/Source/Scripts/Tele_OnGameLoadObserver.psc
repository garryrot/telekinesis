ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto

Event OnInit()
    TeleDevices.Notify("Telekinesis v" + TeleDevices.Version + ": Enable connected devices in MCM for usage...")
    TeleDevices.ConnectAndScanForDevices()
EndEvent

Event OnPlayerLoadGame()
	TeleDevices.LogDebug("OnPlayerLoadGame")
	TeleDevices.ConnectAndScanForDevices()
EndEvent