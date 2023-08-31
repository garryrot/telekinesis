ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto

String[] _ConnectionMenuOptions
String[] _DeviceSelectorOptions ; 0 = All, 1 = Match Tags

Int[] _UseDeviceOids
Int[] _DeviceEventOids

String[] _DeviceNames
Bool _DebugSpellsAdded

Int Function GetVersion()
    return 7
EndFunction

Event OnConfigInit()
    ModName = "Telekinesis"
    InitLocals()
EndEvent

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
        TeleDevices.LogDebug("Updating MCM from " + CurrentVersion + " to " + aVersion)
    EndIf

    If CurrentVersion > 0 && CurrentVersion < 7 ; 1.0.0 Beta
        InitLocals()
        TeleIntegration.ResetIntegrationSettings()
    EndIf
EndEvent

Function InitLocals()
    Pages = new String[5]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Integration"
    Pages[3] = "Debug"
    Pages[4] = "Troubleshooting"

    _ConnectionMenuOptions = new String[3]
    _ConnectionMenuOptions[0] = "In-Process (Default)"
    _ConnectionMenuOptions[1] = "Intiface (WebSocket)" ; Not supported right now
    _ConnectionMenuOptions[2] = "Disable"

    _DeviceSelectorOptions = new String[2]
    _DeviceSelectorOptions[0] = "All"
    _DeviceSelectorOptions[1] = "Match Events"

    _UseDeviceOids = new Int[20] ; Reserve mcm space for 5 fields per device
    _DeviceEventOids = new Int[20]
    
    _DeviceNames = new String[1]
    _DebugSpellsAdded = false
EndFunction

Event OnPageReset(String page)
    If page == "General" || page == ""
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddTextOption("Version", TeleDevices.Version, OPTION_FLAG_DISABLED)
        If ! Tele_Api.Loaded()
            AddTextOption("Connction", "SKSE plugin not loaded", OPTION_FLAG_DISABLED)
            return
        EndIf

        AddHeaderOption("Connection")
        AddMenuOptionST("CONNECTION_MENU", "Connection", _ConnectionMenuOptions[TeleDevices.ConnectionType])
        AddTextOptionST("ACTION_RECONNECT", "Reconnect...", "")

        AddHeaderOption("Emergency")
        AddTextOptionST("EMERGENCY_STOP", "Stop all devices", "Click me")
        AddKeyMapOptionST("EMERGENCY_HOTKEY", "'Stop all' hotkey",  TeleIntegration.EmergencyHotkey)
    EndIf

    If page == "Devices"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If ! TeleDevices.Connects()
            AddHeaderOption("Connection Disabled...")
            return
        EndIf
  
        AddHeaderOption("Discovery")
        AddToggleOptionST("ACTION_SCAN_FOR_DEVICES", "Scan for devices", TeleDevices.ScanningForDevices)
        _DeviceNames = Tele_Api.GetDevices()
        Int len = _DeviceNames.Length
        If len > 20
            TeleDevices.LogError("Too many devices, ignoring some in MCM")
            len = 20
        EndIf

        Int i = 0
        While (i < len) 
            String name = _DeviceNames[i]
            
            If name != ""
                Bool connected = Tele_Api.GetDeviceConnected(name)

                AddHeaderOption(name)
                String status = "Disconnected"
                If connected
                    status = "Connected"
                EndIf
                AddTextOption(Key(i, "Status"), status, OPTION_FLAG_DISABLED)
                AddTextOption(Key(i, "Actions"), Tele_Api.GetDeviceCapabilities(name), OPTION_FLAG_DISABLED)
                _DeviceEventOids[i] = AddInputOption(Key(i, "Events"), Join(Tele_Api.GetEvents(name), ","))

                Int flags = OPTION_FLAG_DISABLED
                If connected
                    flags = OPTION_FLAG_NONE
                EndIf
                _UseDeviceOids[i] = AddToggleOption(Key(i, "Enabled"), TeleDevices.Connects() && Tele_Api.GetEnabled(name), flags)
            EndIf

            i += 1
        EndWhile

        If _DeviceNames.Length == 0
            AddHeaderOption("No devices discovered yet...")
        EndIf
    EndIf

    If page == "Integration"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Devious Devices")
        If TeleIntegration.ZadLib != None
            AddToggleOptionST("OPTION_DEVIOUS_VIBRATE", "In-Game Vibrators", TeleIntegration.Devious_VibrateEffect)
            int selector_flags = OPTION_FLAG_DISABLED
            If TeleIntegration.Devious_VibrateEffect
                selector_flags = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICE_SELECTOR", "Device Selector", _DeviceSelectorOptions[TeleIntegration.Devious_VibrateEffectDeviceSelector], selector_flags)
            Int flags = OPTION_FLAG_DISABLED
            If TeleIntegration.Devious_VibrateEffect && TeleIntegration.Devious_VibrateEffectDeviceSelector == 1
                flags = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("OPTION_DEVIOUS_EVENT_ANAL", "Event on 'Anal'", TeleIntegration.Devious_VibrateEventAnal, flags)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_VAGINAL", "Event on 'Vaginal'", TeleIntegration.Devious_VibrateEventVaginal, flags)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_NIPPLE", "Event on 'Nipple'", TeleIntegration.Devious_VibrateEventNipple, flags)
        Else
            AddTextOption("In-Game Vibrators", "Not Installed", OPTION_FLAG_DISABLED)
        EndIf

        AddHeaderOption("Toys & Love")
        AddToggleOptionST("OPTION_TOYS_VIBRATE", "In-Game Toys", TeleIntegration.Toys_VibrateEffect)
        AddToggleOptionST("OPTION_TOYS_ANIMATION", "Love Animation", TeleIntegration.Toys_Animation)
        AddToggleOptionST("OPTION_TOYS_DENIAL", "Actor denial", TeleIntegration.Toys_Denial)
        AddToggleOptionST("OPTION_TOYS_OTHER", "Actor tease or orgasm", TeleIntegration.Toys_OtherEvents)

        SetCursorPosition(1)
        AddHeaderOption("Sexlab")
        If TeleIntegration.SexLab != None
            AddToggleOptionST("OPTION_SEXLAB_ANIMATION", "Sexlab Animation", TeleIntegration.Sexlab_Animation)
            int sl_selector_flags = OPTION_FLAG_DISABLED
            If TeleIntegration.Sexlab_Animation
                sl_selector_flags = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_DEVICE_SELECTOR", "Device Selector", _DeviceSelectorOptions[TeleIntegration.Sexlab_AnimationDeviceSelector], sl_selector_flags)
            AddEmptyOption()
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_ORGASM", "Actor Orgasm", TeleIntegration.Sexlab_ActorOrgasm)
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_EDGE", "Actor Edge", TeleIntegration.Sexlab_ActorEdge)
        Else
            AddTextOption("Sexlab", "Not Installed", OPTION_FLAG_DISABLED)
        EndIf

        AddHeaderOption("Skyrim Chainbeasts")
        AddToggleOptionST("OPTION_CHAINBESTS_VIBRATE", "Gemmed Beasts", TeleIntegration.Chainbeasts_Vibrate)
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
        AddToggleOptionST("ACTION_ADD_SPELLS_TO_PLAYER", "Learn debug spells", _DebugSpellsAdded)
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
		SetSliderDialogDefaultValue(TeleIntegration.Chainbeasts_Min_Default)
		SetSliderDialogRange(0, TeleIntegration.Chainbeasts_Max)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Chainbeasts_Min = value as int
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Min)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Chainbeasts_Min = TeleIntegration.Chainbeasts_Min_Default
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Min)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Min vibration strength for chainbeast events")
	EndEvent
EndState

State SLIDER_CHAINBEAST_MAX
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Chainbeasts_Max)
		SetSliderDialogDefaultValue(TeleIntegration.Chainbeasts_Max_Default)
		SetSliderDialogRange(TeleIntegration.Chainbeasts_Min, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Chainbeasts_Max = value as int
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Max)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Chainbeasts_Max = TeleIntegration.Chainbeasts_Max_Default
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
        SetMenuDialogOptions(_ConnectionMenuOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleDevices.ConnectionType = index
        SetMenuOptionValueST(_ConnectionMenuOptions[index])
        Debug.MessageBox("Reconnecting now")
        ActionReconnect()
    EndEvent

    Event OnDefaultST()
        TeleDevices.ConnectionType = 0
        SetMenuOptionValueST(_ConnectionMenuOptions[TeleDevices.ConnectionType])
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
        TeleIntegration.EmergencyHotkey = newKeyCode
        SetKeyMapOptionValueST(TeleIntegration.EmergencyHotkey)
    EndEvent

    Event OnDefaultST()
        TeleIntegration.EmergencyHotkey = TeleIntegration.EmergencyHotkey_Default
        SetKeyMapOptionValueST(TeleIntegration.EmergencyHotkey)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A hotkey for immediately stopping all devices from moving (Default: DEL)")
    EndEvent
EndState

State OPTION_DEVIOUS_VIBRATE
    Event OnSelectST()
        TeleIntegration.Devious_VibrateEffect = !TeleIntegration.Devious_VibrateEffect
        SetToggleOptionValueST(TeleIntegration.Devious_VibrateEffect)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Devious_VibrateEffect = TeleIntegration.Devious_VibrateEffect_Default
        SetToggleOptionValueST(TeleIntegration.Devious_VibrateEffect)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with in-game vibrators (vibrate effect start/stop)")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Devious_VibrateEffectDeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Devious_VibrateEffectDeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Devious_VibrateEffectDeviceSelector = TeleIntegration.Devious_VibrateEffectDeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Devious_VibrateEffectDeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Events' if you only want to vibrate devices that correspond to a matching in-game item\n"
        text += "Supported events: Anal (Buttplug), Vaginal (Vaginal Plug, Piercing), Nipple (Piercing)\n"
        SetInfoText(text)
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Devious_VibrateEventAnal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Devious_VibrateEventAnal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Anal' devices. Default: Anal")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Devious_VibrateEventNipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Devious_VibrateEventNipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Nipple' devices. Default: Nipple")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Devious_VibrateEventVaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Devious_VibrateEventVaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Vaginal' devices. Default: Vaginal")
    EndEvent
EndState

State OPTION_SEXLAB_ANIMATION
    Event OnSelectST()
        TeleIntegration.Sexlab_Animation = !TeleIntegration.Sexlab_Animation
        SetToggleOptionValueST(TeleIntegration.Sexlab_Animation)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Sexlab_Animation = TeleIntegration.Sexlab_Animation_Default
        SetToggleOptionValueST(TeleIntegration.Sexlab_Animation)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on sexlab player animation")
    EndEvent
EndState

State MENU_SEXLAB_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Sexlab_AnimationDeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
        ForcePageReset()
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Sexlab_AnimationDeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Sexlab_AnimationDeviceSelector = TeleIntegration.Sexlab_AnimationDeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Sexlab_AnimationDeviceSelector])
    EndEvent

    Event OnHighlightST()
        String txt = "Set to 'Match Events' when you only want to vibrate devices that match any of the sexlab animation tags\n"
        txt += "Note: Will match any tag, but Anal, Boobjob, Vaginal, Masturbation, Oral are probably the events you want to match."
        SetInfoText(txt)
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_ORGASM
    Event OnSelectST()
        TeleIntegration.Sexlab_ActorOrgasm = !TeleIntegration.Sexlab_ActorOrgasm
        SetToggleOptionValueST(TeleIntegration.Sexlab_ActorOrgasm)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Sexlab_ActorOrgasm = TeleIntegration.Sexlab_ActorOrgasm_Default
        SetToggleOptionValueST(TeleIntegration.Sexlab_ActorOrgasm)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on player orgasm")
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_EDGE
    Event OnSelectST()
        TeleIntegration.Sexlab_ActorEdge = !TeleIntegration.Sexlab_ActorEdge
        SetToggleOptionValueST(TeleIntegration.Sexlab_ActorEdge)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Sexlab_ActorEdge = TeleIntegration.Sexlab_ActorEdge_Default
        SetToggleOptionValueST(TeleIntegration.Sexlab_ActorEdge)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices on player edge")
    EndEvent
EndState

State OPTION_TOYS_VIBRATE
    Event OnSelectST()
        TeleIntegration.Toys_VibrateEffect = !TeleIntegration.Toys_VibrateEffect
        SetToggleOptionValueST(TeleIntegration.Toys_VibrateEffect)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_VibrateEffect = TeleIntegration.Toys_VibrateEffect_Default
        SetToggleOptionValueST(TeleIntegration.Toys_VibrateEffect)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with in-game vibrators (toys pulsate start/stop)")
    EndEvent
EndState

State OPTION_TOYS_ANIMATION
    Event OnSelectST()
        TeleIntegration.Toys_Animation = !TeleIntegration.Toys_Animation
        SetToggleOptionValueST(TeleIntegration.Toys_Animation)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Animation = TeleIntegration.Toys_Animation_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Animation)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during 'Toys & Love' player sex animation")
    EndEvent
EndState

State OPTION_TOYS_DENIAL
    Event OnSelectST()
        TeleIntegration.Toys_Denial = !TeleIntegration.Toys_Denial
        SetToggleOptionValueST(TeleIntegration.Toys_Denial)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Denial = TeleIntegration.Toys_Denial_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Denial)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Stop device movement on denial (toys denial event)")
    EndEvent
EndState

State OPTION_TOYS_OTHER
    Event OnSelectST()
        TeleIntegration.Toys_OtherEvents = !TeleIntegration.Toys_OtherEvents
        SetToggleOptionValueST(TeleIntegration.Toys_OtherEvents)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_OtherEvents = TeleIntegration.Toys_OtherEvents_Default
        SetToggleOptionValueST(TeleIntegration.Toys_OtherEvents)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during other 'Toys & Love' events: Fondled, Fondle, Squirt, Climax, ClimaxSimultaneous, Caressed, Denied")
    EndEvent
EndState

State OPTION_CHAINBESTS_VIBRATE
    Event OnSelectST()
        TeleIntegration.Chainbeasts_Vibrate = !TeleIntegration.Chainbeasts_Vibrate
        SetToggleOptionValueST(TeleIntegration.Chainbeasts_Vibrate)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Chainbeasts_Vibrate = TeleIntegration.Chainbeasts_Vibrate_Default
        SetToggleOptionValueST(TeleIntegration.Chainbeasts_Vibrate)
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
        If ! _DebugSpellsAdded
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
            _DebugSpellsAdded = true
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
            _DebugSpellsAdded = false
        EndIf
        SetToggleOptionValueST(_DebugSpellsAdded)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(_DebugSpellsAdded)
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

Event OnOptionSelect(int oid)
    Int i = 0
    While (i < 31 && i < _DeviceNames.Length)
        If (oid == _UseDeviceOids[i])
            String device = _DeviceNames[i]
            Bool isUsed = ! Tele_Api.GetEnabled(device)
            SetToggleOptionValue(oid, isUsed)
            Tele_Api.SetEnabled(device, isUsed)
        EndIf
        i += 1
    EndWhile
    Tele_Api.SettingsStore()
EndEvent

Event OnOptionInputAccept(int oid, string value)
    Int i = 0
    While (i < 31 && i < _DeviceNames.Length)
        If (oid == _DeviceEventOids[i])
            String name = _DeviceNames[i]
            Tele_Api.SetEvents(name, StringUtil.Split(value, ","))
            SetInputOptionValue(oid, value)
        EndIf
        i += 1
    EndWhile
    Tele_Api.SettingsStore()
EndEvent

Event OnOptionHighlight(int oid)
    Int i = 0
    While (i < 31 && i < _DeviceNames.Length)
        If (oid == _DeviceEventOids[i])  
            String infoText = "A comma-separated list of events that are associated with this device\n"
            infoText += "Example 1: Vaginal,Anal,Nipple\n"
            infoText += "Example 2: Nipple"
            SetInfoText(infoText)
        EndIf
        i += 1
    EndWhile
EndEvent

String Function Key( String index, String name )
    return "[" + index + "] " + name
EndFunction

String Function Join(String[] segments, String separator)
    String joined = ""
    Int j = 0
    While (j < segments.Length)
        If j > 0
            joined += separator
        EndIf
        joined += segments[j]
        j += 1
    EndWhile
    return joined
EndFunction
