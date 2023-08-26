ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto

Int selectedConnection = 0
String[] ConnectionMenuOptions
Int[] UseDeviceOids
String[] DeviceNames
Bool SpellsAdded = false
Int EmergencyHotkey = 211 ; Del 

Bool Devious_VibrateEffect = true

Bool Sexlab_Animation = false
Bool Sexlab_ActorOrgasm = false
Bool Sexlab_ActorEdge = false

Bool Toys_VibrateEffect = true
Bool Toys_Animation = false
Bool Toys_OtherEvents = false
Bool Toys_Denial = false

Bool Chainbeasts_Vibrate = true

Int Function GetVersion()
    return 5
EndFunction

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
        TeleDevices.LogDebug("Updating MCM from v" + CurrentVersion + " to v" + aVersion)
    EndIf
    If CurrentVersion < 4
        InitAll()
    EndIf
    If CurrentVersion < 5
        TeleIntegration.Chainbeasts_Vibrate = true
    EndIf
EndEvent

Event OnConfigInit()
    ModName = "Telekinesis"
    InitAll()
    RegisterForKey(EmergencyHotkey)
    TeleIntegration.Devious_VibrateEffect = true
    TeleIntegration.Toys_VibrateEffect = true
EndEvent

Function InitAll()
    Pages = new String[5]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Integration"
    Pages[3] = "Debug"
    Pages[4] = "Troubleshooting"

    ConnectionMenuOptions = new String[3]
    ConnectionMenuOptions[0] = "In-Process (Default)"
    ConnectionMenuOptions[1] = "Intiface (WebSocket)" ; Not supported right now
    ConnectionMenuOptions[2] = "Disable"

    UseDeviceOids = new Int[20] ; Reserve mcm space for 5 fields per device
    DeviceNames = new String[1]
    SpellsAdded = false
EndFunction

Event OnKeyUp(Int KeyCode, Float HoldTime)
    If KeyCode == EmergencyHotkey
        TeleDevices.StopVibrate()
        Tele_Api.StopAll()
        TeleDevices.LogError("Emergency stop")
    Else
        TeleDevices.LogDebug("Unregistered keypress code: " + KeyCode)
    EndIf
EndEvent

Event OnOptionSelect(int oid)
    Int i = 0
    While (i < 31)
        If (oid == UseDeviceOids[i])
            If (i < DeviceNames.Length)
                String device = DeviceNames[i]
                Bool isUsed = ! Tele_Api.GetEnabled(device)
                SetToggleOptionValue(oid, isUsed)
                Tele_Api.SetEnabled(device, isUsed)
            EndIf
        EndIf
        i += 1
    EndWhile

    Tele_Api.SettingsStore()
EndEvent

Event OnPageReset(String page)

    If page == "General" || page == ""
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddTextOption("Version", TeleDevices.Version, OPTION_FLAG_DISABLED)
        If ! Tele_Api.Loaded()
            AddTextOption("Connction", "SKSE plugin not loaded", OPTION_FLAG_DISABLED)
            return
        EndIf

        AddHeaderOption("Connection")
        AddMenuOptionST("CONNECTION_MENU", "Connection", ConnectionMenuOptions[TeleDevices.ConnectionType])
        AddTextOptionST("ACTION_RECONNECT", "Reconnect...", "")

        AddHeaderOption("Emergency")
        AddTextOptionST("EMERGENCY_STOP", "Stop all devices", "Click me")
        AddKeyMapOptionST("EMERGENCY_HOTKEY", "'Stop all' hotkey", EmergencyHotkey)
    EndIf

    If page == "Devices"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If ! TeleDevices.Connects()
            AddHeaderOption("Connection Disabled...")
            return
        EndIf
  
        AddHeaderOption("Discovery")
        AddToggleOptionST("ACTION_SCAN_FOR_DEVICES", "Scan for devices", TeleDevices.ScanningForDevices)
        DeviceNames = Tele_Api.GetDevices()
        Int len = DeviceNames.Length
        If len > 20
            TeleDevices.LogError("Too many devices, ignoring some in MCM")
            len = 20
        EndIf

        Int i = 0
        While (i < len) 
            String name = DeviceNames[i]
            
            If name != ""
                Bool connected = Tele_Api.GetDeviceConnected(name)

                AddHeaderOption(name)
                String status = "Disconnected"
                If connected
                    status = "Connected"
                EndIf
                AddTextOption(Key(i, "Status"), status, OPTION_FLAG_DISABLED)
                AddTextOption(Key(i, "Actions"), Tele_Api.GetDeviceCapabilities(name), OPTION_FLAG_DISABLED)

                Int flags = OPTION_FLAG_DISABLED
                If connected
                    flags = OPTION_FLAG_NONE
                EndIf
                UseDeviceOids[i] = AddToggleOption(Key(i, "Enabled"), TeleDevices.Connects() && Tele_Api.GetEnabled(name), flags)
            EndIf

            i += 1
        EndWhile

        If DeviceNames.Length == 0
            AddHeaderOption("No devices discovered yet...")
        EndIf
    EndIf

    If page == "Integration"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Devious Devices")
        AddToggleOptionST("OPTION_DEVIOUS_VIBRATE", "In-Game Vibrators", Devious_VibrateEffect)

        AddHeaderOption("Toys & Love")
        AddToggleOptionST("OPTION_TOYS_VIBRATE", "In-Game Toys", Toys_VibrateEffect)
        AddToggleOptionST("OPTION_TOYS_ANIMATION", "Love Animation", Toys_Animation)
        AddToggleOptionST("OPTION_TOYS_DENIAL", "Actor denial", Toys_Denial)
        AddToggleOptionST("OPTION_TOYS_OTHER", "Actor tease or orgasm", Toys_OtherEvents)

        SetCursorPosition(1)
        AddHeaderOption("Sexlab")
        AddToggleOptionST("OPTION_SEXLAB_ANIMATION", "Sexlab Animation", Sexlab_Animation)
        AddToggleOptionST("OPTION_SEXLAB_ACTOR_ORGASM", "Actor Orgasm", Sexlab_ActorOrgasm)
        AddToggleOptionST("OPTION_SEXLAB_ACTOR_EDGE", "Actor Edge", Sexlab_ActorEdge)

        AddHeaderOption("Skyrim Chainbeasts")
        AddToggleOptionST("OPTION_CHAINBESTS_VIBRATE", "Gemmed Beasts", Chainbeasts_Vibrate)
	    AddSliderOptionST("SLIDER_CHAINBEAST_MIN", "Min Strength", TeleIntegration.Chainbeasts_Min)
	    AddSliderOptionST("SLIDER_CHAINBEAST_MAX", "Max Strength", TeleIntegration.Chainbeasts_Max)
    EndIf

    If page == "Debug"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Logging")
        AddToggleOptionST("OPTION_LOG_CONNECTS", "Device connects/disconnects", TeleDevices.LogDeviceConnects)
        AddToggleOptionST("OPTION_LOG_EVENTS", "Device events (Vibrations, etc.)", TeleDevices.LogDeviceEvents)
        AddToggleOptionST("OPTION_LOG_DEBUG", "Other messages", TeleDevices.LogDebugEvents)

        AddHeaderOption("Spells")
        AddToggleOptionST("ACTION_ADD_SPELLS_TO_PLAYER", "Learn debug spells", SpellsAdded)
    EndIf

    If page == "Troubleshooting"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddTextOptionST("HELP_DEVICE_NOT_CONNECTING", "Device not connecting", "Read below")
        AddTextOptionST("HELP_DEVICE_NOT_VIBRATING", "Device not vibrating", "Read below")
    EndIf
EndEvent

State SLIDER_CHAINBEAST_MIN
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Chainbeasts_Min)
		SetSliderDialogDefaultValue(50)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Chainbeasts_Min = value as int
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Min)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Chainbeasts_Min = 80
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Min)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Min vibration strength for chainbeast events")
	EndEvent
EndState

State SLIDER_CHAINBEAST_MAX
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Chainbeasts_Max)
		SetSliderDialogDefaultValue(50)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Chainbeasts_Max = value as int
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Max)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Chainbeasts_Max = 100
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Max)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Max vibration strength for chainbeast events")
	EndEvent
EndState


State CONNECTION_MENU
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleDevices.ConnectionType)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(ConnectionMenuOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleDevices.ConnectionType = index
        SetMenuOptionValueST(ConnectionMenuOptions[index])
        Debug.MessageBox("Reconnecting now")
        ActionReconnect()
    EndEvent

    Event OnDefaultST()
        TeleDevices.ConnectionType = 0
        SetMenuOptionValueST(ConnectionMenuOptions[TeleDevices.ConnectionType])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Specify how Telekinesis connects to Buttplug.io")
    EndEvent
EndState

State ACTION_RECONNECT
    Event OnSelectST()
        SetTextOptionValueST("Reconnecting now...")
        ActionReconnect()
        SetTextOptionValueST("Done!")
    EndEvent

    Event OnHighlightST()
        SetInfoText("Disconnect and re-connect all device connections")
    EndEvent
EndState

Function ActionReconnect()
    TeleDevices.Disconnect()
    Utility.Wait(3)
    TeleDevices.ConnectAndScanForDevices()
EndFunction

State EMERGENCY_STOP
    Event OnSelectST()
        SetTextOptionValueST("Stopping...")
        Tele_Api.StopAll()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Immediately stop all devices from moving")
    EndEvent
EndState

State EMERGENCY_HOTKEY
    Event OnKeyMapChangeST(int newKeyCode, string conflictControl, string conflictName)
        UnregisterForKey(EmergencyHotkey)
        EmergencyHotkey = newKeyCode
        SetKeyMapOptionValueST(EmergencyHotkey)
        RegisterForKey(EmergencyHotkey)
    EndEvent

    Event OnDefaultST()
        UnregisterForKey(EmergencyHotkey)
        EmergencyHotkey = 55
        SetKeyMapOptionValueST(EmergencyHotkey)
        RegisterForKey(EmergencyHotkey)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A hotkey for immediately stopping all devices from moving (Default: DEL)")
    EndEvent
EndState

State OPTION_DEVIOUS_VIBRATE
    Event OnSelectST()
        Devious_VibrateEffect = !Devious_VibrateEffect
        SetToggleOptionValueST(Devious_VibrateEffect)
        TeleIntegration.Devious_VibrateEffect = Devious_VibrateEffect
    EndEvent
    
    Event OnDefaultST()
        Devious_VibrateEffect = true
        SetToggleOptionValueST(Devious_VibrateEffect)
        TeleIntegration.Devious_VibrateEffect = Devious_VibrateEffect
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with in-game vibrators (vibrate effect start/stop)")
    EndEvent
EndState

State OPTION_SEXLAB_ANIMATION
    Event OnSelectST()
        Sexlab_Animation = !Sexlab_Animation
        SetToggleOptionValueST(Sexlab_Animation)
        TeleIntegration.Sexlab_Animation = Sexlab_Animation
    EndEvent
    
    Event OnDefaultST()
        Sexlab_Animation = false
        SetToggleOptionValueST(Sexlab_Animation)
        TeleIntegration.Sexlab_Animation = Sexlab_Animation
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on sexlab player animation")
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_ORGASM
    Event OnSelectST()
        Sexlab_ActorOrgasm = !Sexlab_ActorOrgasm
        SetToggleOptionValueST(Sexlab_ActorOrgasm)
        TeleIntegration.Sexlab_ActorOrgasm = Sexlab_ActorOrgasm
    EndEvent
    
    Event OnDefaultST()
        Sexlab_ActorOrgasm = false
        SetToggleOptionValueST(Sexlab_ActorOrgasm)
        TeleIntegration.Sexlab_ActorOrgasm = Sexlab_ActorOrgasm
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on player orgasm")
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_EDGE
    Event OnSelectST()
        Sexlab_ActorEdge = !Sexlab_ActorEdge
        SetToggleOptionValueST(Sexlab_ActorEdge)
        TeleIntegration.Sexlab_ActorEdge = Sexlab_ActorEdge
    EndEvent
    
    Event OnDefaultST()
        Sexlab_ActorEdge = false
        SetToggleOptionValueST(Sexlab_ActorEdge)
        TeleIntegration.Sexlab_ActorEdge = Sexlab_ActorEdge
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on player edge")
    EndEvent
EndState

State OPTION_TOYS_VIBRATE
    Event OnSelectST()
        Toys_VibrateEffect = !Toys_VibrateEffect
        SetToggleOptionValueST(Toys_VibrateEffect)
        TeleIntegration.Toys_VibrateEffect = Toys_VibrateEffect
    EndEvent
    
    Event OnDefaultST()
        Toys_VibrateEffect = false
        SetToggleOptionValueST(Toys_VibrateEffect)
        TeleIntegration.Toys_VibrateEffect = Toys_VibrateEffect
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with in-game vibrators (toys pulsate start/stop)")
    EndEvent
EndState

State OPTION_TOYS_ANIMATION
    Event OnSelectST()
        Toys_Animation = !Toys_Animation
        SetToggleOptionValueST(Toys_Animation)
        TeleIntegration.Toys_Animation = Toys_Animation
    EndEvent
    
    Event OnDefaultST()
        Toys_Animation = false
        SetToggleOptionValueST(Toys_Animation)
        TeleIntegration.Toys_Animation = Toys_Animation
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during 'Toys & Love' player sex animation")
    EndEvent
EndState

State OPTION_TOYS_DENIAL
    Event OnSelectST()
        Toys_Denial = !Toys_Denial
        SetToggleOptionValueST(Toys_Denial)
        TeleIntegration.Toys_Denial = Toys_Denial
    EndEvent
    
    Event OnDefaultST()
        Toys_Denial = false
        SetToggleOptionValueST(Toys_Denial)
        TeleIntegration.Toys_Denial = Toys_Denial
    EndEvent

    Event OnHighlightST()
        SetInfoText("Stop device movement on denial (toys denial event)")
    EndEvent
EndState

State OPTION_TOYS_OTHER
    Event OnSelectST()
        Toys_OtherEvents = !Toys_OtherEvents
        SetToggleOptionValueST(Toys_OtherEvents)
        TeleIntegration.Toys_OtherEvents = Toys_OtherEvents
    EndEvent
    
    Event OnDefaultST()
        Toys_OtherEvents = false
        SetToggleOptionValueST(Toys_OtherEvents)
        TeleIntegration.Toys_OtherEvents = Toys_OtherEvents
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during other 'Toys & Love' events: Fondled, Fondle, Squirt, Climax, ClimaxSimultaneous, Caressed, Denied")
    EndEvent
EndState

State OPTION_CHAINBESTS_VIBRATE
    Event OnSelectST()
        Chainbeasts_Vibrate = !Chainbeasts_Vibrate
        SetToggleOptionValueST(Chainbeasts_Vibrate)
        TeleIntegration.Chainbeasts_Vibrate = Chainbeasts_Vibrate
    EndEvent
    
    Event OnDefaultST()
        Chainbeasts_Vibrate = true
        SetToggleOptionValueST(Chainbeasts_Vibrate)
        TeleIntegration.Chainbeasts_Vibrate = Chainbeasts_Vibrate
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrates devices during gemmed chainbeast capture (Requires Chainbeasts Version >= 0.7.0)")
    EndEvent
EndState

State OPTION_LOG_CONNECTS
    Event OnSelectST()
        TeleDevices.LogDeviceConnects = !TeleDevices.LogDeviceConnects
        SetToggleOptionValueST(TeleDevices.LogDeviceConnects)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(TeleDevices.LogDeviceConnects)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show notification when a device connects/disconnects")
    EndEvent
EndState

State OPTION_LOG_EVENTS
    Event OnSelectST()
        TeleDevices.LogDeviceEvents = !TeleDevices.LogDeviceEvents
        SetToggleOptionValueST(TeleDevices.LogDeviceEvents)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(TeleDevices.LogDeviceEvents)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show notification when a device event (Vibration etc.) occurs")
    EndEvent
EndState

State OPTION_LOG_DEBUG
    Event OnSelectST()
        TeleDevices.LogDebugEvents = !TeleDevices.LogDebugEvents
        SetToggleOptionValueST(TeleDevices.LogDebugEvents)
    EndEvent
    
    Event OnDefaultST()
        TeleDevices.LogDebugEvents = false
        SetToggleOptionValueST(TeleDevices.LogDebugEvents)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show internal debug notifications")
    EndEvent
EndState

State ACTION_SCAN_FOR_DEVICES
    Event OnSelectST()
        If TeleDevices.ScanningForDevices
            Tele_Api.StopScan()
        Else
            Tele_Api.ScanForDevices()
        EndIf
        TeleDevices.ScanningForDevices = !TeleDevices.ScanningForDevices
        SetToggleOptionValueST(TeleDevices.ScanningForDevices)
    EndEvent
    
    Event OnDefaultST()
        TeleDevices.ScanningForDevices = true
        SetToggleOptionValueST(TeleDevices.ScanningForDevices)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Automatically scan for new devices (resets to 'true' on each restart)")
    EndEvent
EndState

State ACTION_ADD_SPELLS_TO_PLAYER
    Event OnSelectST()
        Actor player = Game.GetPlayer()
        If ! SpellsAdded
            If ! player.HasSpell(TeleDevices.Tele_VibrateSpellWeak)
                player.AddSpell(TeleDevices.Tele_VibrateSpellWeak)
            EndIf
            If ! player.HasSpell(TeleDevices.Tele_VibrateSpellMedium)
                player.AddSpell(TeleDevices.Tele_VibrateSpellMedium)
            EndIf
            If ! player.HasSpell(TeleDevices.Tele_VibrateSpellStrong)
                player.AddSpell(TeleDevices.Tele_VibrateSpellStrong)
            EndIf
            If ! player.HasSpell(TeleDevices.Tele_Stop)
                player.AddSpell(TeleDevices.Tele_Stop)
            EndIf
            SpellsAdded = true
        Else
            If player.HasSpell(TeleDevices.Tele_VibrateSpellWeak)
                player.RemoveSpell(TeleDevices.Tele_VibrateSpellWeak)
            EndIf
            If player.HasSpell(TeleDevices.Tele_VibrateSpellMedium)
                player.RemoveSpell(TeleDevices.Tele_VibrateSpellMedium)
            EndIf
            If player.HasSpell(TeleDevices.Tele_VibrateSpellStrong)
                player.RemoveSpell(TeleDevices.Tele_VibrateSpellStrong)
            EndIf
            If player.HasSpell(TeleDevices.Tele_Stop)
                player.RemoveSpell(TeleDevices.Tele_Stop)
            EndIf
            SpellsAdded = false
        EndIf
        SetToggleOptionValueST(SpellsAdded)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(SpellsAdded)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Add spells for controlling device vibration")
    EndEvent
EndState

State HELP_DEVICE_NOT_CONNECTING
    Event OnSelectST()
    EndEvent
    Event OnHighlightST()
        String a = "If a device does not connect, check that:\n"
        String b = "1. Windows has bluetooth active and the device is connected\n"
        String c = "2. Device has full battery\n"
        String d = "3. Device is supported by buttplug.io (test in Intiface app)\n"
        SetInfoText(a + b + c + d)
    EndEvent
EndState

State HELP_DEVICE_NOT_VIBRATING
    Event OnSelectST()
    EndEvent
    Event OnHighlightST()
        String a = "If a device connects but does not vibrate, check that:\n"
        String b = "1. Device is enabled (MCM: Devices Page)\n"
        String c = "2. Device has full battery\n"
        String d = "3. Device works in buttplug.io (test in Intiface app)\n"
        SetInfoText(a + b + c + d)
    EndEvent
EndState

String Function Key( String index, String name )
    return "[" + index + "] " + name
EndFunction
