ScriptName Tele_OnGameLoadObserver extends ReferenceAlias

Event OnInit()
    Tele_Devices devices = GetOwningQuest() as Tele_Devices
    devices.Notify("Telekinesis v" + devices.Version + ": Enable devices in MCM for usage...")
    LoadTelekinesis(devices)
EndEvent

Event OnPlayerLoadGame()
    Tele_Devices devices = GetOwningQuest() as Tele_Devices
    LoadTelekinesis(devices)
EndEvent

Function LoadTelekinesis(Tele_Devices devices)
    Tele_Integration integration = GetOwningQuest() as Tele_Integration

    If Game.GetModByName("Devious Devices - Expansion.esm") != 255
        integration.ZadLib = Quest.GetQuest("zadQuest")
    Else
        integration.ZadLib = None
    EndIf

    If Game.GetModByName("SexLab.esm") != 255
        integration.SexLab = Quest.GetQuest("SexLabQuestFramework")
    Else
        integration.SexLab = None
    EndIf

    If Game.GetModByName("Toys.esm") != 255
        integration.Toys = Quest.GetQuest("ToysFramework")
    Else
        integration.Toys = None
    EndIf

    If Game.GetModByName("SexLabAroused.esm") != 255
        integration.SexLabAroused = Quest.GetQuest("sla_Framework")
    Else
        integration.SexLabAroused = None
    EndIf
       
    If Game.GetModByName("OStim.esp") != 255
        integration.HasOStim = True
    Else
        integration.HasOStim = False
    EndIf
    
    integration.PlayerRef = Game.GetPlayer()
    integration.Maintenance()
    devices.ConnectAndScanForDevices()
EndFunction