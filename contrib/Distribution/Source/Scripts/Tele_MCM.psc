ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto

String[] _ConnectionMenuOptions
String[] _DeviceSelectorOptions ; 0 = All, 1 = Match Tags
String[] _PatternSelectorOptions

Int[] _UseDeviceOids
Int[] _DeviceEventOids
Int[] _TestVibratePatternOid
Int[] _TestStrokePatternOid

String[] _StrokeFunscriptNames
String[] _VibrateFunscriptNames

String[] _DeviceNames
Bool _DebugSpellsAdded

Int Function GetVersion()
    return 9
EndFunction

Event OnConfigInit()
    ModName = "Telekinesis"
    InitLocals()
EndEvent

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
        TeleDevices.LogDebug("Updating MCM from " + CurrentVersion + " to " + aVersion)
    EndIf

    If CurrentVersion > 0 && CurrentVersion < 9 ; 1.0.0 Beta
        InitLocals()
        TeleIntegration.ResetIntegrationSettings()
    EndIf
EndEvent

Function InitLocals()
    Pages = new String[6]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Integration"
    Pages[3] = "Patterns"
    Pages[4] = "Debug"
    Pages[5] = "Troubleshooting"

    _ConnectionMenuOptions = new String[3]
    _ConnectionMenuOptions[0] = "In-Process (Default)"
    _ConnectionMenuOptions[1] = "Intiface (WebSocket)" ; Not supported right now
    _ConnectionMenuOptions[2] = "Disable"

    _DeviceSelectorOptions = new String[2]
    _DeviceSelectorOptions[0] = "All"
    _DeviceSelectorOptions[1] = "Match Events"

    _PatternSelectorOptions = new String[3]
    _PatternSelectorOptions[0] = "Linear"
    _PatternSelectorOptions[1] = "Funscript"
    _PatternSelectorOptions[2] = "Random Funscript"

    _UseDeviceOids = new Int[20] ; Reserve mcm space for 5 fields per device
    _DeviceEventOids = new Int[20]
    
    _DeviceNames = new String[1]
    _DebugSpellsAdded = false

    _StrokeFunscriptNames = new String[127]
    _VibrateFunscriptNames = new String[127]
    _TestVibratePatternOid = new Int[127]
    _TestStrokePatternOid = new Int[127]
EndFunction

Event OnPageReset(String page)
    _VibrateFunscriptNames = TeleDevices.GetPatternNames(true)
    _StrokeFunscriptNames = Tele_Api.GetPatternNames(false)
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
            AddToggleOptionST("OPTION_DEVIOUS_DEVICES_VIBRATE", "In-Game Vibrators", TeleIntegration.DeviousDevices_Vibrate)
            Int devioues_devices_vibrate_selector_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate
                devioues_devices_vibrate_selector_flag = OPTION_FLAG_NONE
            EndIf

            AddEmptyOption()
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_DEVICE_SELECTOR", "Devices", _DeviceSelectorOptions[TeleIntegration.DeviousDevices_Vibrate_DeviceSelector], devioues_devices_vibrate_selector_flag)

            Int devioues_devices_vibrate_event_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate && TeleIntegration.DeviousDevices_Vibrate_DeviceSelector == 1
                devioues_devices_vibrate_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("OPTION_DEVIOUS_EVENT_ANAL", "Event on 'Anal'", TeleIntegration.DeviousDevices_Vibrate_Event_Anal, devioues_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_VAGINAL", "Event on 'Vaginal'", TeleIntegration.DeviousDevices_Vibrate_Event_Vaginal, devioues_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_NIPPLE", "Event on 'Nipple'", TeleIntegration.DeviousDevices_Vibrate_Event_Nipple, devioues_devices_vibrate_event_flag)
        
            Int devioues_devices_vibrate_pattern_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate
                devioues_devices_vibrate_pattern_flag = OPTION_FLAG_NONE
            EndIf

            AddEmptyOption()
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_PATTERN", "Vibration Pattern", _PatternSelectorOptions[TeleIntegration.DeviousDevices_Vibrate_Pattern], devioues_devices_vibrate_pattern_flag)
        
            Int devioues_devices_vibrate_funscript_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate && TeleIntegration.DeviousDevices_Vibrate_Pattern == 1
                devioues_devices_vibrate_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_FUNSCRIPT", "Vibration Funscript", TeleIntegration.DeviousDevices_Vibrate_Funscript, devioues_devices_vibrate_funscript_flag)
        
            Int devioues_devices_vibrate_linear_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate && TeleIntegration.DeviousDevices_Vibrate_Pattern == 0
                devioues_devices_vibrate_linear_flag = OPTION_FLAG_NONE
            EndIf      
        Else
            AddTextOption("In-Game Vibrators", "Not Installed", OPTION_FLAG_DISABLED)
        EndIf

        AddHeaderOption("Toys & Love")
        AddToggleOptionST("OPTION_TOYS_VIBRATE", "In-Game Toys", TeleIntegration.Toys_VibrateEffect)
        AddEmptyOption()

        ; Match events
        AddToggleOptionST("OPTION_TOYS_ANIMATION", "Love Animation", TeleIntegration.Toys_Animation)
        AddToggleOptionST("OPTION_TOYS_VAGINAL_PENETRATION", "Vaginal Penetration", TeleIntegration.Toys_Vaginal_Penetration)
        AddToggleOptionST("OPTION_TOYS_ANAL_PENETRATION", "Anal Penetration", TeleIntegration.Toys_Anal_Penetration)
        AddToggleOptionST("OPTION_TOYS_ORAL_PENETRATION", "Vaginal Penetration", TeleIntegration.Toys_Oral_Penetration)
        AddEmptyOption()

        AddToggleOptionST("OPTION_TOYS_DENIAL", "Denial", TeleIntegration.Toys_Denial)
        AddToggleOptionST("OPTION_TOYS_FONDLE", "Fondle", TeleIntegration.Toys_Fondle)
        AddToggleOptionST("OPTION_TOYS_SQUIRT", "Squirt", TeleIntegration.Toys_Squirt)
        AddEmptyOption()

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
        AddToggleOptionST("OPTION_CHAINBEASTS_VIBRATE", "Gemmed Beasts", TeleIntegration.Chainbeasts_Vibrate)
        Int chainbeasts_vibrate_selector_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_selector_flag = OPTION_FLAG_NONE
        EndIf
        
        AddEmptyOption()
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_DEVICE_SELECTOR", "Devices", _DeviceSelectorOptions[TeleIntegration.Chainbeasts_Vibrate_DeviceSelector], chainbeasts_vibrate_selector_flag)

        Int chainbeasts_vibrate_event_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_DeviceSelector == 1
            chainbeasts_vibrate_event_flag = OPTION_FLAG_NONE
        EndIf
        AddInputOptionST("INPUT_CHAINBEASTS_VIBRATE_EVENT", "Match Event", TeleIntegration.Chainbeasts_Vibrate_Event, chainbeasts_vibrate_event_flag)

        Int chainbeasts_vibrate_pattern_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_pattern_flag = OPTION_FLAG_NONE
        EndIf
        
        AddEmptyOption()
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_PATTERN", "Vibration Pattern", _PatternSelectorOptions[TeleIntegration.Chainbeasts_Vibrate_Pattern], chainbeasts_vibrate_pattern_flag)

        Int chainbeast_vibrate_funscript_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_Pattern == 1
            chainbeast_vibrate_funscript_flag = OPTION_FLAG_NONE
        EndIf
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_FUNSCRIPT", "Vibration Funscript", TeleIntegration.Chainbeasts_Vibrate_Funscript, chainbeast_vibrate_funscript_flag)

        Int chainbeasts_vibrate_linear_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_Pattern == 0
            chainbeasts_vibrate_linear_flag = OPTION_FLAG_NONE
        EndIf
	    AddSliderOptionST("SLIDER_CHAINBEASTS_VIBRATE_LINEAR_STRENGTH", "Linear Strength", TeleIntegration.Chainbeasts_Vibrate_Linear_Strength, "{0}", chainbeasts_vibrate_linear_flag)
    EndIf
    
    If page == "Patterns"
        SetCursorFillMode(TOP_TO_BOTTOM)
        AddHeaderOption("Vibrator Patterns")
        Int j = 0
        While j < _VibrateFunscriptNames.Length && j < 63
            String vibrate_pattern = _VibrateFunscriptNames[j]
            _TestVibratePatternOid[j] = AddTextOption(vibrate_pattern, "(test me)")
		    SetTextOptionValue(_TestVibratePatternOid[j], "running...")
            j += 1
        EndWhile

        SetCursorPosition(1)
        AddHeaderOption("Stroker Patterns")
        Int i = 0
        While i < _StrokeFunscriptNames.Length && i < 63
            String stroker_pattern = _StrokeFunscriptNames[i]
            _TestStrokePatternOid[j] = AddTextOption(stroker_pattern, "(test me)")
		    SetTextOptionValue(_TestStrokePatternOid[j], "running...")
            i += 1
        EndWhile
    Endif

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

State OPTION_CHAINBEASTS_VIBRATE
    Event OnSelectST()
        TeleIntegration.Chainbeasts_Vibrate = !TeleIntegration.Chainbeasts_Vibrate
        SetToggleOptionValueST(TeleIntegration.Chainbeasts_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Chainbeasts_Vibrate = TeleIntegration.Chainbeasts_Vibrate_Default
        SetToggleOptionValueST(TeleIntegration.Chainbeasts_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrates devices during gemmed chainbeast capture (Requires Chainbeasts Version >= 0.7.0)")
    EndEvent
EndState

State MENU_CHAINBEASTS_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Chainbeasts_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Chainbeasts_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Chainbeasts_Vibrate_DeviceSelector = TeleIntegration.Chainbeasts_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Chainbeasts_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Events' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_CHAINBEASTS_VIBRATE_EVENT
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Chainbeasts_Vibrate_Event)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Chainbeasts_Vibrate_Event = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Select only devices matching the input event")
    EndEvent
EndState

State MENU_CHAINBEASTS_VIBRATE_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Chainbeasts_Vibrate_Pattern = index
        SetMenuOptionValueST(_PatternSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_PatternSelectorOptions[0])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("'Linear': Constant vibration strength. 'Funscript': Vibration is controlled by a named funscript file. 'Random Funscript': Use a randomly selected funscript.")
    EndEvent
EndState

State MENU_CHAINBEASTS_VIBRATE_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Chainbeasts_Vibrate_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State SLIDER_CHAINBEASTS_VIBRATE_LINEAR_STRENGTH
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Chainbeasts_Vibrate_Linear_Strength)
		SetSliderDialogDefaultValue(TeleIntegration.Chainbeasts_Vibrate_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Chainbeasts_Vibrate_Linear_Strength = value as int
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Vibrate_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Chainbeasts_Vibrate_Linear_Strength = TeleIntegration.Chainbeasts_Vibrate_Linear_Strength_Default
		SetSliderOptionValueST(TeleIntegration.Chainbeasts_Vibrate_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
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

State OPTION_DEVIOUS_DEVICES_VIBRATE
    Event OnSelectST()
        TeleIntegration.DeviousDevices_Vibrate = !TeleIntegration.DeviousDevices_Vibrate
        SetToggleOptionValueST(TeleIntegration.DeviousDevices_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.DeviousDevices_Vibrate = TeleIntegration.DeviousDevices_Vibrate_Default
        SetToggleOptionValueST(TeleIntegration.DeviousDevices_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Enable vibration support for devious devices in-game vibrators")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICES_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.DeviousDevices_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.DeviousDevices_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.DeviousDevices_Vibrate_DeviceSelector = TeleIntegration.DeviousDevices_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.DeviousDevices_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Events' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.DeviousDevices_Vibrate_Event_Anal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.DeviousDevices_Vibrate_Event_Anal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Anal' devices. Default: Anal")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.DeviousDevices_Vibrate_Event_Nipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.DeviousDevices_Vibrate_Event_Nipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Nipple' devices. Default: Nipple")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.DeviousDevices_Vibrate_Event_Vaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.DeviousDevices_Vibrate_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for 'Vaginal' devices. Default: Vaginal")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICES_VIBRATE_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.DeviousDevices_Vibrate_Pattern = index
        SetMenuOptionValueST(_PatternSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_PatternSelectorOptions[0])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("'Linear': Constant speed based on devious devices event data. 'Funscript': Vibration is controlled by a named funscript file. 'Random Funscript': Use a randomly selected funscript.")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICES_VIBRATE_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.DeviousDevices_Vibrate_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
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
        SetInfoText("Plays a long and soft vibration patttern during each toys&love animation that involves the player")
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
        SetInfoText("'Rewards' a successfull 'denial' event with a 7s long stop period (no device will vibrate)")
    EndEvent
EndState

State OPTION_TOYS_VAGINAL_PENETRATION
    Event OnSelectST()
        TeleIntegration.Toys_Vaginal_Penetration = !TeleIntegration.Toys_Vaginal_Penetration
        SetToggleOptionValueST(TeleIntegration.Toys_Vaginal_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Vaginal_Penetration = TeleIntegration.Toys_Vaginal_Penetration_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Vaginal_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Vaginal' event/tag on 'vaginal penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_ANAL_PENETRATION
    Event OnSelectST()
        TeleIntegration.Toys_Anal_Penetration = !TeleIntegration.Toys_Anal_Penetration
        SetToggleOptionValueST(TeleIntegration.Toys_Anal_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Anal_Penetration = TeleIntegration.Toys_Anal_Penetration_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Anal_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Anal' event/tag on 'anal penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_ORAL_PENETRATION
    Event OnSelectST()
        TeleIntegration.Toys_Oral_Penetration = !TeleIntegration.Toys_Oral_Penetration
        SetToggleOptionValueST(TeleIntegration.Toys_Oral_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Oral_Penetration = TeleIntegration.Toys_Oral_Penetration_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Oral_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Oral' event/tag on 'oral penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_FONDLE
    Event OnSelectST()
        TeleIntegration.Toys_Fondle = !TeleIntegration.Toys_Fondle
        SetToggleOptionValueST(TeleIntegration.Toys_Fondle)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Fondle = TeleIntegration.Toys_Fondle_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Fondle)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A light vibration on all devices during the 'fondle' event")
    EndEvent
EndState

State OPTION_TOYS_SQUIRT
    Event OnSelectST()
        TeleIntegration.Toys_Squirt = !TeleIntegration.Toys_Squirt
        SetToggleOptionValueST(TeleIntegration.Toys_Squirt)
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Squirt = TeleIntegration.Toys_Squirt_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Squirt)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A strong 12s vibration on each 'squirt' event")
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
    i = 0
    While (i < _TestVibratePatternOid.Length)
        If (oid == _TestVibratePatternOid[i])
            String patternName = _VibrateFunscriptNames[i]
            String[] allEvents = new String[1]
            TeleDevices.VibratePattern(patternName, 30, allEvents)
        EndIf
        i += 1
    EndWhile
    i = 0
    While (i < _TestStrokePatternOid.Length)
        If (oid == _TestStrokePatternOid[i])
            String patternName = _StrokeFunscriptNames[i]
            Debug.MessageBox("Not supported yet")
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
