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
        TeleIntegration.ZadLib = Quest.GetQuest("zadQuest")
    Else
        TeleIntegration.ZadLib = None
    EndIf

    If Game.GetModByName("SexLab.esm") != 255
        TeleIntegration.SexLab = Quest.GetQuest("SexLabQuestFramework")
    Else
        TeleIntegration.SexLab = None
    EndIf

    If Game.GetModByName("Toys.esm") != 255
        TeleIntegration.Toys = Quest.GetQuest("ToysFramework")
    Else
        TeleIntegration.Toys = None
    EndIf

    If Game.GetModByName("SexLabAroused.esm") != 255
        TeleIntegration.SexLabAroused = Quest.GetQuest("sla_Framework")
    Else
        TeleIntegration.SexLabAroused = None
    EndIf
       
    If Game.GetModByName("OStim.esp") != 255
        TeleIntegration.OStim = True
    Else
        TeleIntegration.OStim = False
    EndIf
    
    TeleDevices.ConnectAndScanForDevices()
EndFunction