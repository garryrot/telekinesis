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
        TeleIntegration.UnregisterLegacyUpdate()
    EndIf
EndEvent

Function InitLocals()
    Pages = new String[9]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Funscript Patterns"
    Pages[3] = "Devious Devices"
    Pages[4] = "Sexlab"
    Pages[5] = "Toys & Love"
    Pages[6] = "Skyrim Chain Beasts"
    Pages[7] = "Debug"
    Pages[8] = "Troubleshooting"

    _ConnectionMenuOptions = new String[3]
    _ConnectionMenuOptions[0] = "In-Process (Default)"
    _ConnectionMenuOptions[1] = "Intiface (WebSocket)"
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
        Int connection_ws_flags = OPTION_FLAG_DISABLED
        If (TeleDevices.ConnectionType == 1)
            connection_ws_flags = OPTION_FLAG_NONE
        EndIf
        AddInputOptionST("CONNECTION_HOST", "Intiface Host", TeleDevices.WsHost, connection_ws_flags)
        AddInputOptionST("CONNECTION_PORT", "Intiface Port", TeleDevices.WsPort, connection_ws_flags)
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

    If page == "Devious Devices"
        SetCursorFillMode(TOP_TO_BOTTOM)

        If TeleIntegration.ZadLib != None
            AddHeaderOption("In-Game Vibrators")
            AddToggleOptionST("OPTION_DEVIOUS_DEVICES_VIBRATE", "Enable", TeleIntegration.DeviousDevices_Vibrate)
            Int devious_devices_vibrate_selector_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate
                devious_devices_vibrate_selector_flag = OPTION_FLAG_NONE
            EndIf

            AddHeaderOption("Devices")
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TeleIntegration.DeviousDevices_Vibrate_DeviceSelector], devious_devices_vibrate_selector_flag)

            Int devious_devices_vibrate_event_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate && TeleIntegration.DeviousDevices_Vibrate_DeviceSelector == 1
                devious_devices_vibrate_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("OPTION_DEVIOUS_EVENT_ANAL", "Event on 'Anal'", TeleIntegration.DeviousDevices_Vibrate_Event_Anal, devious_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_VAGINAL", "Event on 'Vaginal'", TeleIntegration.DeviousDevices_Vibrate_Event_Vaginal, devious_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_NIPPLE", "Event on 'Nipple'", TeleIntegration.DeviousDevices_Vibrate_Event_Nipple, devious_devices_vibrate_event_flag)
        
            AddHeaderOption("Actions")
            Int devious_devices_vibrate_pattern_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate
                devious_devices_vibrate_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TeleIntegration.DeviousDevices_Vibrate_Pattern], devious_devices_vibrate_pattern_flag)
        
            Int devious_devices_vibrate_funscript_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.DeviousDevices_Vibrate && TeleIntegration.DeviousDevices_Vibrate_Pattern == 1
                devious_devices_vibrate_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TeleIntegration.DeviousDevices_Vibrate_Funscript, devious_devices_vibrate_funscript_flag)
            AddTextOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_STRENGTH", "Strength", "Controlled by DD", OPTION_FLAG_DISABLED)
        Else
            AddTextOption("Devious Devices", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Sexlab"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If TeleIntegration.SexLab != None
            AddHeaderOption("Sexlab Animations")
            AddToggleOptionST("OPTION_SEXLAB_ANIMATION", "Enable", TeleIntegration.Sexlab_Animation)
            Int sexlab_animation_selector_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Sexlab_Animation
                sexlab_animation_selector_flag = OPTION_FLAG_NONE
            EndIf
            
            AddHeaderOption("Devices")
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TeleIntegration.Sexlab_Animation_DeviceSelector], sexlab_animation_selector_flag)
            AddHeaderOption("Actions")
            If TeleIntegration.SexLabAroused != None
                Int sexlab_animation_rousing_flag = OPTION_FLAG_DISABLED
                If TeleIntegration.Sexlab_Animation
                    sexlab_animation_rousing_flag = OPTION_FLAG_NONE
                EndIf    
                AddToggleOptionST("OPTION_SEXLAB_ANIMATION_ROUSING", "Arousal = Vibration Strength", TeleIntegration.Sexlab_Animation_Rousing, sexlab_animation_rousing_flag)
            Else
                AddTextOption("Arousal = Vibration Strength", "Requires SLA", OPTION_FLAG_DISABLED)
            EndIf

            Int sexlab_animation_pattern_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Sexlab_Animation && ! TeleIntegration.Sexlab_Animation_Rousing
                sexlab_animation_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TeleIntegration.Sexlab_Animation_Pattern], sexlab_animation_pattern_flag)
        
            Int sexlab_animation_funscript_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Sexlab_Animation && ! TeleIntegration.Sexlab_Animation_Rousing && TeleIntegration.Sexlab_Animation_Pattern == 1
                sexlab_animation_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_FUNSCRIPT", "Vibrate Funscript", TeleIntegration.Sexlab_Animation_Funscript, sexlab_animation_funscript_flag)
        
            AddHeaderOption("Extra Actions")
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_EDGE", "Pause on Actor Edge", TeleIntegration.Sexlab_ActorEdge)
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_ORGASM", "Strong Vibration on Orgasm", TeleIntegration.Sexlab_ActorOrgasm)
        Else
            AddTextOption("Sexlab", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Toys & Love"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If TeleIntegration.Toys != None
            AddHeaderOption("In-Game Vibrators")
            AddToggleOptionST("OPTION_TOYS_VIBRATE", "Enable", TeleIntegration.Toys_Vibrate)    

            AddHeaderOption("Devices")
            Int toys_vibrate_selector_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Vibrate
                toys_vibrate_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TeleIntegration.Toys_Vibrate_DeviceSelector], toys_vibrate_selector_flag)
        
            Int toys_vibrate_event_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Vibrate && TeleIntegration.Toys_Vibrate_DeviceSelector == 1
                toys_vibrate_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("INPUT_TOYS_VIBRATE_EVENT", "Match Event", TeleIntegration.Toys_Vibrate_Event, toys_vibrate_event_flag)
        
            AddHeaderOption("Actions")
            Int toys_vibrate_pattern_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Vibrate
                toys_vibrate_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TeleIntegration.Toys_Vibrate_Pattern], toys_vibrate_pattern_flag)
        
            Int toys_vibrate_funscript_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Vibrate && TeleIntegration.Toys_Vibrate_Pattern == 1
                toys_vibrate_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TeleIntegration.Toys_Vibrate_Funscript, toys_vibrate_funscript_flag)
        
            Int toys_vibrate_linear_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Vibrate && TeleIntegration.Toys_Vibrate_Pattern == 0
                toys_vibrate_linear_flag = OPTION_FLAG_NONE
            EndIf
            AddSliderOptionST("SLIDER_TOYS_VIBRATE_LINEAR_STRENGTH", "Strength", TeleIntegration.Toys_Vibrate_Linear_Strength, "{0}", toys_vibrate_linear_flag)
            
            SetCursorPosition(1)
            AddHeaderOption("Love Animations")
            AddToggleOptionST("OPTION_TOYS_ANIMATION", "Enable", TeleIntegration.Toys_Animation)

            AddHeaderOption("Devices")
            Int toys_animation_selector_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation
                toys_animation_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_DEVICE_SELECTOR", "Devices", _DeviceSelectorOptions[TeleIntegration.Toys_Animation_DeviceSelector], toys_animation_selector_flag)

            Int toys_animation_event_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation && TeleIntegration.Toys_Animation_DeviceSelector == 1
                toys_animation_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_VAGINAL", "Event on 'Vaginal'", TeleIntegration.Toys_Animation_Event_Vaginal, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_ANAL", "Event on 'Anal'", TeleIntegration.Toys_Animation_Event_Anal, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_ORAL", "Event on 'Oral'", TeleIntegration.Toys_Animation_Event_Oral, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_NIPPLE", "Event on 'Nipple'", TeleIntegration.Toys_Animation_Event_Nipple, toys_animation_event_flag)

            AddHeaderOption("Actions")
            Int toys_animation_rousing = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation
                toys_animation_rousing = OPTION_FLAG_NONE
            EndIf
            AddToggleOptionST("OPTION_TOYS_ANIMATION_ROUSING", "Rousing = Vibration Strength", TeleIntegration.Toys_Animation_Rousing, toys_animation_rousing)

            Int toys_animation_pattern_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation && ! TeleIntegration.Toys_Animation_Rousing
                toys_animation_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TeleIntegration.Toys_Animation_Pattern], toys_animation_pattern_flag)
        
            Int toys_animation_funscript_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation && ! TeleIntegration.Toys_Animation_Rousing &&  TeleIntegration.Toys_Animation_Pattern == 1
                toys_animation_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_FUNSCRIPT", "Vibrate Funscript", TeleIntegration.Toys_Animation_Funscript, toys_animation_funscript_flag)
        
            Int toys_animation_linear_flag = OPTION_FLAG_DISABLED
            If TeleIntegration.Toys_Animation && ! TeleIntegration.Toys_Animation_Rousing && TeleIntegration.Toys_Animation_Pattern == 0
                toys_animation_linear_flag = OPTION_FLAG_NONE
            EndIf
            AddSliderOptionST("SLIDER_TOYS_ANIMATION_LINEAR_STRENGTH", "Strength (Linear)", TeleIntegration.Toys_Animation_Linear_Strength, "{0}", toys_animation_linear_flag)

            AddHeaderOption("Extra Actions")
            AddToggleOptionST("OPTION_TOYS_DENIAL", "Pause on Denial", TeleIntegration.Toys_Denial)
            AddToggleOptionST("OPTION_TOYS_VAGINAL_PENETRATION", "Strong Vaginal Penetration", TeleIntegration.Toys_Vaginal_Penetration)
            AddToggleOptionST("OPTION_TOYS_ANAL_PENETRATION", "Strong Anal Penetration", TeleIntegration.Toys_Anal_Penetration)
            AddToggleOptionST("OPTION_TOYS_ORAL_PENETRATION", "Strong Vaginal Penetration", TeleIntegration.Toys_Oral_Penetration)
            AddToggleOptionST("OPTION_TOYS_FONDLE", "Vibration on Fondle", TeleIntegration.Toys_Fondle)
            AddToggleOptionST("OPTION_TOYS_SQUIRT", "Vibration on Squirt", TeleIntegration.Toys_Squirt)
        Else
            AddTextOption("Toys & Love", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Skyrim Chain Beasts"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Gemmed Beasts")
        AddToggleOptionST("OPTION_CHAINBEASTS_VIBRATE", "Enable", TeleIntegration.Chainbeasts_Vibrate)
        Int chainbeasts_vibrate_selector_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_selector_flag = OPTION_FLAG_NONE
        EndIf
        
        AddHeaderOption("Devices")
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TeleIntegration.Chainbeasts_Vibrate_DeviceSelector], chainbeasts_vibrate_selector_flag)

        Int chainbeasts_vibrate_event_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_DeviceSelector == 1
            chainbeasts_vibrate_event_flag = OPTION_FLAG_NONE
        EndIf
        AddInputOptionST("INPUT_CHAINBEASTS_VIBRATE_EVENT", "Match Event", TeleIntegration.Chainbeasts_Vibrate_Event, chainbeasts_vibrate_event_flag)

        AddHeaderOption("Action")
        Int chainbeasts_vibrate_pattern_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_pattern_flag = OPTION_FLAG_NONE
        EndIf
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TeleIntegration.Chainbeasts_Vibrate_Pattern], chainbeasts_vibrate_pattern_flag)

        Int chainbeast_vibrate_funscript_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_Pattern == 1
            chainbeast_vibrate_funscript_flag = OPTION_FLAG_NONE
        EndIf
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TeleIntegration.Chainbeasts_Vibrate_Funscript, chainbeast_vibrate_funscript_flag)

        Int chainbeasts_vibrate_linear_flag = OPTION_FLAG_DISABLED
        If TeleIntegration.Chainbeasts_Vibrate && TeleIntegration.Chainbeasts_Vibrate_Pattern == 0
            chainbeasts_vibrate_linear_flag = OPTION_FLAG_NONE
        EndIf
	    AddSliderOptionST("SLIDER_CHAINBEASTS_VIBRATE_LINEAR_STRENGTH", "Strength", TeleIntegration.Chainbeasts_Vibrate_Linear_Strength, "{0}", chainbeasts_vibrate_linear_flag)
    EndIf
    
    If page == "Funscript Patterns"
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


; General

State CONNECTION_MENU
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleDevices.ConnectionType)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_ConnectionMenuOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleDevices.ConnectionType = index
        SetMenuOptionValueST(_ConnectionMenuOptions[index])
        Debug.MessageBox("Please reconnect now!")
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleDevices.ConnectionType = 0
        SetMenuOptionValueST(_ConnectionMenuOptions[TeleDevices.ConnectionType])
    EndEvent

    Event OnHighlightST()
        String t = "Specify how Telekinesis performs its device control\n"
        t += "- In-Process: The devices are controlled directly by Telekinesis native plugin (recommended)\n"
        t += "- Intiface (WebSocket): devices are controlled by Intiface, which requires that the app (and\n"
        t += "           its server) are running and listening on the 'WebSocket Host' and 'WebSocket Port'"
        SetInfoText(t)
    EndEvent
EndState

State CONNECTION_HOST
	Event OnInputOpenST()
		SetInputDialogStartText(TeleDevices.WsHost)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleDevices.WsHost = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The host-name of your Intiface Web-Socket Endpoint (check Intiface App). Default: 127.0.0.1")
    EndEvent
EndState

State CONNECTION_PORT
	Event OnInputOpenST()
		SetInputDialogStartText(TeleDevices.WsPort)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleDevices.WsPort = value
        Tele_Api.SettingsSet("connection.websocket", "127.0.0.1:12345")
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The port your Intiface Web-Socket Endpoint (check Intiface App) Default: 12345")
    EndEvent
EndState

State ACTION_RECONNECT
    Event OnSelectST()
        SetTextOptionValueST("Reconnecting now...")
        TeleDevices.Reconnect()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Disconnect and re-connect all device connections")
    EndEvent
EndState

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

; Devices 

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

; Devious Devices

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
        SetInfoText("Change the event that is triggered for 'Anal' devices. Default: Anal")
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
        SetInfoText("Change the event that is triggered for 'Nipple' devices. Default: Nipple")
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
        SetInfoText("Change event that is triggered for 'Vaginal' devices. Default: Vaginal")
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

; Sexlab

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
        SetInfoText("Move devices during sexlab player animation")
    EndEvent
EndState

State MENU_SEXLAB_ANIMATION_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Sexlab_Animation_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Sexlab_Animation_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Sexlab_Animation_DeviceSelector = TeleIntegration.Sexlab_Animation_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Sexlab_Animation_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "Set to 'Match Events' when you only want to vibrate devices that match any of the sexlab animation tags\n"
        txt += "Note: Will match any tag, but Anal, Boobjob, Vaginal, Masturbation, Oral are probably the events you want to associate with your devices"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_SEXLAB_ANIMATION_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Sexlab_Animation_Pattern = index
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

State MENU_SEXLAB_ANIMATION_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Sexlab_Animation_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State OPTION_SEXLAB_ANIMATION_ROUSING
    Event OnSelectST()
        TeleIntegration.Sexlab_Animation_Rousing = !TeleIntegration.Sexlab_Animation_Rousing
        SetToggleOptionValueST(TeleIntegration.Sexlab_Animation_Rousing)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Sexlab_Animation_Rousing = TeleIntegration.Sexlab_Animation_Rousing_Default
        SetToggleOptionValueST(TeleIntegration.Sexlab_Animation_Rousing)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibration strength is controlled by SLA Arousal: 10 = 10% strength, 100 = 100% strength...")
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
        SetInfoText("Start an additional (stronger) vibration on all matching devices whenever the player orgasms. This will override/enhance existing vibrations.")
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
        SetInfoText("Stop the vibration on all matching devices for a short time whenever the player edges. This will override existing vibrations.")
    EndEvent
EndState

; Toys & Love

State OPTION_TOYS_VIBRATE
    Event OnSelectST()
        TeleIntegration.Toys_Vibrate = !TeleIntegration.Toys_Vibrate
        SetToggleOptionValueST(TeleIntegration.Toys_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Vibrate = TeleIntegration.Toys_Vibrate_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with Toys & Love in-game vibrators (toys pulsate start/stop)")
    EndEvent
EndState

State MENU_TOYS_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Toys_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Toys_Vibrate_DeviceSelector = TeleIntegration.Toys_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Toys_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Events' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_TOYS_VIBRATE_EVENT
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Toys_Vibrate_Event)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Toys_Vibrate_Event = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate only devices matching the input event")
    EndEvent
EndState

State MENU_TOYS_VIBRATE_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Vibrate_Pattern = index
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

State MENU_TOYS_VIBRATE_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Vibrate_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State SLIDER_TOYS_VIBRATE_LINEAR_STRENGTH
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Toys_Vibrate_Linear_Strength)
		SetSliderDialogDefaultValue(TeleIntegration.Toys_Vibrate_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Toys_Vibrate_Linear_Strength = value as int
		SetSliderOptionValueST(TeleIntegration.Toys_Vibrate_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Toys_Vibrate_Linear_Strength = TeleIntegration.Toys_Vibrate_Linear_Strength_Default
		SetSliderOptionValueST(TeleIntegration.Toys_Vibrate_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
	EndEvent
EndState

State OPTION_TOYS_ANIMATION
    Event OnSelectST()
        TeleIntegration.Toys_Animation = !TeleIntegration.Toys_Animation
        SetToggleOptionValueST(TeleIntegration.Toys_Animation)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Animation = TeleIntegration.Toys_Animation_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Animation)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Enable vibration during Toys & Love Sex animations")
    EndEvent
EndState

State MENU_TOYS_ANIMATION_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TeleIntegration.Toys_Animation_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Animation_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TeleIntegration.Toys_Animation_DeviceSelector = TeleIntegration.Toys_Animation_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TeleIntegration.Toys_Animation_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Events' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Toys_Animation_Event_Vaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Toys_Animation_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Pussy' or 'Vaginal' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_ORAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Toys_Animation_Event_ORAL)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Toys_Animation_Event_ORAL = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Oral' or 'Blowjob' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Toys_Animation_Event_Anal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Toys_Animation_Event_Anal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Anal' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TeleIntegration.Toys_Animation_Event_Nipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TeleIntegration.Toys_Animation_Event_Nipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Nipple' or 'Breast' tags")
    EndEvent
EndState

State OPTION_TOYS_ANIMATION_ROUSING
    Event OnSelectST()
        TeleIntegration.Toys_Animation_Rousing = !TeleIntegration.Toys_Animation_Rousing
        SetToggleOptionValueST(TeleIntegration.Toys_Animation_Rousing)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TeleIntegration.Toys_Animation_Rousing = TeleIntegration.Toys_Animation_Rousing_Default
        SetToggleOptionValueST(TeleIntegration.Toys_Animation_Rousing)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibration strength is directly controlled by rousing: 10 = 10% strength, 100 = 100% strength...")
    EndEvent
EndState

State MENU_TOYS_ANIMATION_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Animation_Pattern = index
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

State MENU_TOYS_ANIMATION_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TeleIntegration.Toys_Animation_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State SLIDER_TOYS_ANIMATION_LINEAR_STRENGTH
	Event OnSliderOpenST()
		SetSliderDialogStartValue(TeleIntegration.Toys_Animation_Linear_Strength)
		SetSliderDialogDefaultValue(TeleIntegration.Toys_Animation_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TeleIntegration.Toys_Animation_Linear_Strength = value as int
		SetSliderOptionValueST(TeleIntegration.Toys_Animation_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TeleIntegration.Toys_Animation_Linear_Strength = TeleIntegration.Toys_Animation_Linear_Strength_Default
		SetSliderOptionValueST(TeleIntegration.Toys_Animation_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
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

; Chainbeasts

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
        SetInfoText("Vibrates devices during gemmed chainbeast capture (Requires V. >= 0.7.0 and recompiled SCB_VibeEffectScript.pex)")
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

; Debug

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

; Help

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
