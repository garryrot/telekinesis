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
    Tele_Integration teleIntegration = GetOwningQuest() as Tele_Integration

    If Game.GetModByName("Devious Devices - Expansion.esm") != 255
        teleIntegration.ZadLib = Quest.GetQuest("zadQuest")
    Else
        teleIntegration.ZadLib = None
    EndIf

    If Game.GetModByName("SexLab.esm") != 255
        teleIntegration.SexLab = Quest.GetQuest("SexLabQuestFramework")
    Else
        teleIntegration.SexLab = None
    EndIf

    If Game.GetModByName("Toys.esm") != 255
        teleIntegration.Toys = Quest.GetQuest("ToysFramework")
    Else
        teleIntegration.Toys = None
    EndIf

    If Game.GetModByName("SexLabAroused.esm") != 255
        teleIntegration.SexLabAroused = Quest.GetQuest("sla_Framework")
    Else
        teleIntegration.SexLabAroused = None
    EndIf
       
    If Game.GetModByName("OStim.esp") != 255
        teleIntegration.OStim = OUtils.GetOStim() as Quest
    Else
        teleIntegration.OStim = None
    EndIf

    teleIntegration.PlayerRef = Game.GetPlayer()
    devices.ConnectAndScanForDevices()
EndFunction