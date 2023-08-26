ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto

Event OnInit()
    TeleDevices.Notify("Telekinesis v" + TeleDevices.Version + ": Enable connected devices in MCM for usage...")
    LoadTelekinesis()
EndEvent

Event OnPlayerLoadGame()
    LoadTelekinesis()
EndEvent

Function LoadTelekinesis()
	TeleDevices.LogDebug("Loading")
    If Game.GetModByName("Devious Devices - Expansion.esm") != 255
        TeleIntegration.ZadLib = (Quest.GetQuest("zadQuest") As ZadLibs)
    Else
        TeleIntegration.ZadLib = None
    EndIf
    TeleDevices.ConnectAndScanForDevices()
EndFunction