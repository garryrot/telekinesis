ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Integration _TIntegration = None
Tele_Integration Property TIntegration Hidden
    Tele_Integration Function Get()
        If (_TIntegration == None)
            _TIntegration = (self as Quest) as Tele_Integration
        Endif
        return _TIntegration
    EndFunction 
EndProperty

Tele_Devices _TDevices = None
Tele_Devices Property TDevices Hidden
    Tele_Devices Function Get()
        If (_TDevices == None)
            _TDevices = (self as Quest) as Tele_Devices
        Endif
        return _TDevices
    EndFunction
EndProperty

String[] _ConnectionMenuOptions
String[] _DeviceSelectorOptions ; 0 = All, 1 = Match Tags
String[] _PatternSelectorOptions
String[] _PatternSelectorOptionsStroker
String[] _OstimSpeedOptions

Int[] _UseDeviceOids
Int[] _DeviceEventOids
Int[] _TestVibratePatternOid
Int[] _TestStrokePatternOid

Int[] _StrokerMinPosOid
Int[] _StrokerMaxPosOid
Int[] _StrokerMinMsOid
Int[] _StrokerMaxMsOid
Int[] _StrokerInvertOid
Int[] _VibratorMinSpeedOid
Int[] _VibratorMaxSpeedOid
Int[] _VibratorFactorOid

String[] _StrokeFunscriptNames
String[] _VibrateFunscriptNames

String[] _ActuatorIds
Bool _DebugSpellsAdded

Int Function GetVersion()
    return 16
EndFunction

Event OnConfigInit()
    ModName = "Telekinesis"
    InitLocals()
EndEvent

Event OnVersionUpdate(int newVersion)
    If CurrentVersion < newVersion
        TDevices.LogDebug("Updating MCM from " + CurrentVersion + " to " + newVersion)
    EndIf

    If CurrentVersion > 0 && CurrentVersion < 10 
        ; Older than 1.0.0 Beta
        TIntegration.ResetIntegrationSettings()
    EndIf

    If CurrentVersion < 12 
        ; Older than 1.2.0
        InitLocals()
        TDevices.LogDebug("Resetting device settings, please re-enable them.")
        TDevices.MigrateToV12()
        TIntegration.MigrateToV12()
    EndIf

    If CurrentVersion < 16
        ;  Older than 1.3.0
        InitLocals()
        TIntegration.InitDefaultListeners()
    EndIf
EndEvent

Function InitLocals()
    Pages = new String[10]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Funscript Patterns"
    Pages[3] = "Devious Devices"
    Pages[4] = "Sexlab"
    Pages[5] = "Toys & Love"
    Pages[6] = " OStim"
    Pages[7] = "Skyrim Chain Beasts"
    Pages[8] = "Debug"
    Pages[9] = "Troubleshooting"

    _ConnectionMenuOptions = new String[3]
    _ConnectionMenuOptions[0] = "In-Process"
    _ConnectionMenuOptions[1] = "Intiface (WebSocket)"
    _ConnectionMenuOptions[2] = "Disable"

    _DeviceSelectorOptions = new String[2]
    _DeviceSelectorOptions[0] = "All"
    _DeviceSelectorOptions[1] = "Match Body Parts"

    _PatternSelectorOptions = new String[3]
    _PatternSelectorOptions[0] = "Linear"
    _PatternSelectorOptions[1] = "Funscript"
    _PatternSelectorOptions[2] = "Random Funscript"

    _PatternSelectorOptionsStroker = new String[3]
    _PatternSelectorOptionsStroker[0] = "Stroke"
    _PatternSelectorOptionsStroker[1] = "Funscript"
    _PatternSelectorOptionsStroker[2] = "Random Funscript"
    
    _OstimSpeedOptions = new String[4]
    _OstimSpeedOptions[0] = "Constant"
    _OstimSpeedOptions[1] = "Animation Speed"
    _OstimSpeedOptions[2] = "Excitement"
    _OstimSpeedOptions[3] = "Speed*Excitement"
    
    _UseDeviceOids = new Int[20] ; Reserve mcm space for 5 fields per device
    _DeviceEventOids = new Int[20]
    _StrokerMinPosOid = new Int[20]
    _StrokerMaxPosOid = new Int[20]
    _StrokerMinMsOid = new Int[20]
    _StrokerMaxMsOid = new Int[20]
    _StrokerInvertOid = new Int[20]
    _VibratorMinSpeedOid = new Int[20]
    _VibratorMaxSpeedOid = new Int[20]
    _VibratorFactorOid = new Int[20]
    _ActuatorIds = new String[20]
    _DebugSpellsAdded = false

    _StrokeFunscriptNames = new String[1]
    _VibrateFunscriptNames = new String[1]
    _TestVibratePatternOid = new Int[127]
    _TestStrokePatternOid = new Int[127]
EndFunction

Event OnPageReset(String page)
    _VibrateFunscriptNames = Tele_Api.Qry_Lst("patterns.vibrator")
    _StrokeFunscriptNames = Tele_Api.Qry_Lst("patterns.stroker")
    If page == "General" || page == ""
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddTextOption("Version", TDevices.Version, OPTION_FLAG_DISABLED)
        If ! Tele_Api.Loaded()
            AddTextOption("Connction", "SKSE plugin not loaded", OPTION_FLAG_DISABLED)
            return
        EndIf

        AddHeaderOption("Connection")
        AddMenuOptionST("CONNECTION_MENU", "Connection", _ConnectionMenuOptions[TDevices.ConnectionType])
        Int connection_ws_flags = OPTION_FLAG_DISABLED
        If (TDevices.ConnectionType == 1)
            connection_ws_flags = OPTION_FLAG_NONE
        EndIf

        String status = TDevices.GetConnectionStatus()
        If status == "Failed"
            status = "<font color='#fc0303'>Failed</font>" 
        EndIf
        AddTextOptionST("CONNECTION_STATUS", "Status", status)
        AddInputOptionST("CONNECTION_HOST", "Intiface Host", TDevices.WsHost, connection_ws_flags)
        AddInputOptionST("CONNECTION_PORT", "Intiface Port", TDevices.WsPort, connection_ws_flags)
        AddTextOptionST("ACTION_RECONNECT", "Reconnect...", "Click me")

        AddHeaderOption("Emergency")
        AddTextOptionST("EMERGENCY_STOP", "Stop all devices", "Click me")
        AddKeyMapOptionST("EMERGENCY_HOTKEY", "'Stop all' hotkey",  TIntegration.EmergencyHotkey)

        SetCursorPosition(1)
        AddEmptyOption()
    EndIf
    If page == "Devices"
        SetCursorFillMode(LEFT_TO_RIGHT)
        If ! TDevices.Connects()
            AddHeaderOption("Connection Disabled...")
            return
        EndIf
  
        AddHeaderOption("Discovery")
        AddEmptyOption()

        AddToggleOptionST("ACTION_SCAN_FOR_DEVICES", "Scan for devices", TDevices.ScanningForDevices)
        AddEmptyOption()

        _ActuatorIds = Tele_Api.Qry_Lst("devices")
        Int len = _ActuatorIds.Length
        If len > 20
            TDevices.LogError("Too many devices, ignoring some in MCM")
            len = 20
        EndIf
        Int i = 0
        While (i < len) 
            String actuatorId = _ActuatorIds[i]
            
            If actuatorId != ""
                String status = Tele_Api.Qry_Str_1("device.connection.status", actuatorId)
                bool connects = false
                If TDevices.Connects()
                    connects = Tele_Api.Qry_Bool_1("device.settings.enabled", actuatorId)
                EndIf
                String actuatorType = Tele_Api.Qry_Str_1("device.actuator", actuatorId)
                Bool isStroker = actuatorType == "Position"
                String actuatorIndex = Tele_Api.Qry_Str_1("device.actuator.index", actuatorId)
                String[] events = Tele_Api.Qry_Lst_1("device.settings.events", actuatorId)

                AddHeaderOption(actuatorId)
                AddEmptyOption()

                Int enabled_flag = OPTION_FLAG_DISABLED
                If status == "Connected"
                    enabled_flag = OPTION_FLAG_NONE
                EndIf

                _UseDeviceOids[i] = AddToggleOption("Enabled", connects, enabled_flag)
                If isStroker
                    Int minMs = Tele_Api.Qry_Str_1("device.linear.min_ms", actuatorId) as Int
                    _StrokerMinMsOid[i] = AddSliderOption("Fastest Stroke", minMs, "{0} ms")
                Else
                    Int minSpeed = Tele_Api.Qry_Str_1("device.scalar.min_speed", actuatorId) as Int
                    _VibratorMinSpeedOid[i] = AddSliderOption("Min Speed", minSpeed, "{0} %")
                EndIf

                AddTextOption("State", status, OPTION_FLAG_DISABLED)
                If isStroker
                    Int maxMs = Tele_Api.Qry_Str_1("device.linear.max_ms", actuatorId) as Int
                    _StrokerMaxMsOid[i] = AddSliderOption("Slowest Stroke", maxMs, "{0} ms")
                Else
                    Int maxSpeed = Tele_Api.Qry_Str_1("device.scalar.max_speed", actuatorId) as Int
                    _VibratorMaxSpeedOid[i] = AddSliderOption("Max Speed", maxSpeed, "{0} %")
                EndIf

                AddTextOption("Motor", actuatorType + " " + actuatorIndex, OPTION_FLAG_DISABLED)
                If isStroker
                    Float minPos = Tele_Api.Qry_Str_1("device.linear.min_pos", actuatorId) as Float
                    _StrokerMinPosOid[i] = AddSliderOption("Full In", minPos, "{2}")
                Else
                    Float factor = Tele_Api.Qry_Str_1("device.scalar.factor", actuatorId) as Float
                    _VibratorFactorOid[i] = AddSliderOption("Downscale Factor", factor, "{2}")
                    _DeviceEventOids[i] = AddInputOption("Body Parts", Join(events, ","))
                    AddEmptyOption();
                EndIf

                If isStroker
                    _DeviceEventOids[i] = AddInputOption("Body Parts", Join(events, ","))
                    Float maxPos = Tele_Api.Qry_Str_1("device.linear.max_pos", actuatorId) as Float
                    _StrokerMaxPosOid[i] = AddSliderOption("Full Out", maxPos, "{2}")
                    AddEmptyOption()
                    Bool invertPos = Tele_Api.Qry_Bool_1("device.linear.invert", actuatorId)
                    _StrokerInvertOid[i] = AddToggleOption("Invert Pos", invertPos)
                EndIf
            EndIf

            i += 1
        EndWhile

        If _ActuatorIds.Length == 0
            AddHeaderOption("No devices discovered yet...")
        EndIf
    EndIf

    If page == "Devious Devices"
        SetCursorFillMode(TOP_TO_BOTTOM)

        If TIntegration.ZadLib != None
            AddHeaderOption("In-Game Vibrators")
            AddToggleOptionST("OPTION_DEVIOUS_DEVICES_VIBRATE", "Enable Vibrators", TIntegration.DeviousDevices_Vibrate)
            Int devious_devices_vibrate_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.DeviousDevices_Vibrate
                devious_devices_vibrate_selector_flag = OPTION_FLAG_NONE
            EndIf

            AddHeaderOption("Devices")
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.DeviousDevices_Vibrate_DeviceSelector], devious_devices_vibrate_selector_flag)

            Int devious_devices_vibrate_event_flag = OPTION_FLAG_DISABLED
            If TIntegration.DeviousDevices_Vibrate && TIntegration.DeviousDevices_Vibrate_DeviceSelector == 1
                devious_devices_vibrate_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("OPTION_DEVIOUS_EVENT_ANAL", "Event 'Anal Device'", TIntegration.DeviousDevices_Vibrate_Event_Anal, devious_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_VAGINAL", "Event 'Vaginal Device'", TIntegration.DeviousDevices_Vibrate_Event_Vaginal, devious_devices_vibrate_event_flag)
            AddInputOptionST("OPTION_DEVIOUS_EVENT_NIPPLE", "Event 'Nipple Device'", TIntegration.DeviousDevices_Vibrate_Event_Nipple, devious_devices_vibrate_event_flag)
        
            AddHeaderOption("Actions")
            Int devious_devices_vibrate_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.DeviousDevices_Vibrate
                devious_devices_vibrate_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.DeviousDevices_Vibrate_Pattern], devious_devices_vibrate_pattern_flag)
        
            Int devious_devices_vibrate_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.DeviousDevices_Vibrate && TIntegration.DeviousDevices_Vibrate_Pattern == 1
                devious_devices_vibrate_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TIntegration.DeviousDevices_Vibrate_Funscript, devious_devices_vibrate_funscript_flag)
            AddTextOptionST("MENU_DEVIOUS_DEVICES_VIBRATE_STRENGTH", "Strength", "Controlled by DD", OPTION_FLAG_DISABLED)
        Else
            AddTextOption("Devious Devices", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Sexlab"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If TIntegration.SexLab != None
            AddHeaderOption("Sexlab Animations")
            AddToggleOptionST("OPTION_SEXLAB_ANIMATION", "Enable Vibrators", TIntegration.Sexlab_Animation)
            Int sexlab_animation_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Animation
                sexlab_animation_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Sexlab_Animation_DeviceSelector], sexlab_animation_selector_flag)
            AddHeaderOption("Actions")
            If TIntegration.SexLabAroused != None
                Int sexlab_animation_rousing_flag = OPTION_FLAG_DISABLED
                If TIntegration.Sexlab_Animation
                    sexlab_animation_rousing_flag = OPTION_FLAG_NONE
                EndIf    
                AddToggleOptionST("OPTION_SEXLAB_ANIMATION_ROUSING", "Arousal = Vibration Strength", TIntegration.Sexlab_Animation_Rousing, sexlab_animation_rousing_flag)
            Else
                AddTextOption("Arousal = Vibration Strength", "Requires SLA", OPTION_FLAG_DISABLED)
            EndIf
            AddEmptyOption()

            Int sexlab_animation_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Animation
                sexlab_animation_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.Sexlab_Animation_Pattern], sexlab_animation_pattern_flag)
        
            Int sexlab_animation_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Animation && TIntegration.Sexlab_Animation_Pattern == 1
                sexlab_animation_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_ANIMATION_FUNSCRIPT", "Vibrate Funscript", TIntegration.Sexlab_Animation_Funscript, sexlab_animation_funscript_flag)
        
            AddHeaderOption("Extra Actions")
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_EDGE", "Pause on Actor Edge", TIntegration.Sexlab_ActorEdge)
            AddToggleOptionST("OPTION_SEXLAB_ACTOR_ORGASM", "Strong Vibration on Orgasm", TIntegration.Sexlab_ActorOrgasm)

            SetCursorPosition(1)
            AddHeaderOption("")
            AddToggleOptionST("OPTION_SEXLAB_STROKER", "Enable Strokers", TIntegration.Sexlab_Stroker)
            Int sexlab_stroker_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Stroker
                sexlab_stroker_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddToggleOptionST("OPTION_SEXLAB_OSCILLATORS", "Enable Oscillators", TIntegration.Sexlab_Oscillator)
            If TIntegration.Sexlab_Oscillator
                sexlab_stroker_selector_flag = OPTION_FLAG_NONE
            EndIf
      
            AddMenuOptionST("MENU_SEXLAB_STROKER_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Sexlab_Stroker_DeviceSelector], sexlab_stroker_selector_flag)
      
            AddHeaderOption("Actions")
            If TIntegration.SexLabAroused != None
                Int sexlab_stroker_rousing_flag = OPTION_FLAG_DISABLED
                If (TIntegration.Sexlab_Stroker && TIntegration.Sexlab_Stroker_Pattern == 0) || TIntegration.Sexlab_Oscillator
                    sexlab_stroker_rousing_flag = OPTION_FLAG_NONE
                EndIf    
                AddToggleOptionST("OPTION_SEXLAB_STROKER_ROUSING", "Arousal = Stroker Speed", TIntegration.Sexlab_Stroker_Rousing, sexlab_stroker_rousing_flag)
            Else
                AddTextOption("Arousal = Stroker Speed", "Requires SLA", OPTION_FLAG_DISABLED)
            EndIf

            Int sexlab_stroker_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Stroker
                sexlab_stroker_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_STROKER_PATTERN", "Stroker Pattern", _PatternSelectorOptionsStroker[TIntegration.Sexlab_Stroker_Pattern], sexlab_stroker_pattern_flag)
        
            Int sexlab_stroker_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Sexlab_Stroker && TIntegration.Sexlab_Stroker_Pattern == 1
                sexlab_stroker_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_SEXLAB_STROKER_FUNSCRIPT", "Stroker Funscript", TIntegration.Sexlab_Stroker_Funscript, sexlab_stroker_funscript_flag)
        Else
            AddTextOption("Sexlab", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf
    
    If page == " OStim"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If TIntegration.OStim != None
            AddHeaderOption("Ostim Animations")
            AddToggleOptionST("OPTION_OSTIM_ANIMATION", "Enable Vibrators", TIntegration.Ostim_Animation)
            Int ostim_animation_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Animation
                ostim_animation_selector_flag = OPTION_FLAG_NONE
            EndIf

            AddMenuOptionST("MENU_OSTIM_ANIMATION_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Ostim_Animation_DeviceSelector], ostim_animation_selector_flag)

            AddHeaderOption("Actions")
            Int ostim_animation_speed_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Animation
                ostim_animation_speed_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_ANIMATION_SPEED", "Strength", _OstimSpeedOptions[TIntegration.Ostim_Animation_Speed_Control], ostim_animation_speed_flag)

            Int ostim_animation_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Animation
                ostim_animation_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_ANIMATION_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.Ostim_Animation_Pattern], ostim_animation_pattern_flag)
        
            Int ostim_animation_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Animation && TIntegration.Ostim_Animation_Pattern == 1
                ostim_animation_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_ANIMATION_FUNSCRIPT", "Vibrate Funscript", TIntegration.Ostim_Animation_Funscript, ostim_animation_funscript_flag)

            AddHeaderOption("Body Parts")
            Int ostim_event_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Animation_DeviceSelector == 1 || TIntegration.Ostim_Stroker_DeviceSelector == 1
                ostim_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("OSTIM_EVENT_VAGINAL", "Event Vaginal Stimulation", TIntegration.Ostim_Animation_Event_Vaginal, ostim_event_flag)
            AddInputOptionST("OSTIM_EVENT_ANAL", "Event Anal Stimulation", TIntegration.Ostim_Animation_Event_Anal, ostim_event_flag)
            AddInputOptionST("OSTIM_EVENT_NIPPLE", "Event Nipple Stimulation", TIntegration.Ostim_Animation_Event_Nipple, ostim_event_flag)
   
            SetCursorPosition(1)
            AddHeaderOption("")
            AddToggleOptionST("OPTION_OSTIM_STROKER", "Enable Strokers", TIntegration.Ostim_Stroker)
            Int ostim_stroker_speed_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Stroker && TIntegration.Ostim_Stroker_Pattern == 0
                ostim_stroker_speed_flag = OPTION_FLAG_NONE
            EndIf
            AddToggleOptionST("OPTION_OSTIM_OSCILLATOR", "Enable Oscillators", TIntegration.Ostim_Oscillator)
            If TIntegration.Ostim_Oscillator
                ostim_stroker_speed_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_STROKER_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Ostim_Animation_DeviceSelector], ostim_stroker_speed_flag)

            AddHeaderOption("Actions")
            AddMenuOptionST("MENU_OSTIM_STROKER_SPEED", "Speed", _OstimSpeedOptions[TIntegration.Ostim_Stroker_Speed_Control], ostim_stroker_speed_flag)

            Int ostim_stroker_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Stroker
                ostim_stroker_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_STROKER_PATTERN", "Stroker Pattern", _PatternSelectorOptionsStroker[TIntegration.Ostim_Stroker_Pattern], ostim_stroker_pattern_flag)
        
            Int ostim_stroker_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Ostim_Stroker && TIntegration.Ostim_Stroker_Pattern == 1
                ostim_stroker_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_OSTIM_STROKER_FUNSCRIPT", "Stroker Funscript", TIntegration.Ostim_Stroker_Funscript, ostim_stroker_funscript_flag)

            AddHeaderOption("")
            AddInputOptionST("OSTIM_EVENT_PENIS", "Event Penis Stimulation", TIntegration.Ostim_Animation_Event_Penis, ostim_event_flag)
            AddInputOptionST("OSTIM_EVENT_PENETRATION", "Event Penetrating", TIntegration.Ostim_Animation_Event_Penetration, ostim_event_flag)
        Else
            AddTextOption("OStim", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Toys & Love"
        SetCursorFillMode(TOP_TO_BOTTOM)
        If TIntegration.Toys != None
            AddHeaderOption("In-Game Vibrators")
            AddToggleOptionST("OPTION_TOYS_VIBRATE", "Enable", TIntegration.Toys_Vibrate)    

            AddHeaderOption("Devices")
            Int toys_vibrate_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Vibrate
                toys_vibrate_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Toys_Vibrate_DeviceSelector], toys_vibrate_selector_flag)
        
            Int toys_vibrate_event_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Vibrate && TIntegration.Toys_Vibrate_DeviceSelector == 1
                toys_vibrate_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("INPUT_TOYS_VIBRATE_EVENT", "Match Event", TIntegration.Toys_Vibrate_Event, toys_vibrate_event_flag)
        
            AddHeaderOption("Actions")
            Int toys_vibrate_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Vibrate
                toys_vibrate_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.Toys_Vibrate_Pattern], toys_vibrate_pattern_flag)
        
            Int toys_vibrate_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Vibrate && TIntegration.Toys_Vibrate_Pattern == 1
                toys_vibrate_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TIntegration.Toys_Vibrate_Funscript, toys_vibrate_funscript_flag)
        
            Int toys_vibrate_linear_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Vibrate
                toys_vibrate_linear_flag = OPTION_FLAG_NONE
            EndIf
            AddSliderOptionST("SLIDER_TOYS_VIBRATE_LINEAR_STRENGTH", "Strength", TIntegration.Toys_Vibrate_Linear_Strength, "{0}", toys_vibrate_linear_flag)
            
            SetCursorPosition(1)
            AddHeaderOption("Love Animations")
            AddToggleOptionST("OPTION_TOYS_ANIMATION", "Enable", TIntegration.Toys_Animation)

            AddHeaderOption("Devices")
            Int toys_animation_selector_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation
                toys_animation_selector_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Toys_Animation_DeviceSelector], toys_animation_selector_flag)

            Int toys_animation_event_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation && TIntegration.Toys_Animation_DeviceSelector == 1
                toys_animation_event_flag = OPTION_FLAG_NONE
            EndIf
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_VAGINAL", "Event Vaginal Stimulation", TIntegration.Toys_Animation_Event_Vaginal, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_ANAL", "Event Anal Stimulation", TIntegration.Toys_Animation_Event_Anal, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_ORAL", "Event Oral Stimulation", TIntegration.Toys_Animation_Event_Oral, toys_animation_event_flag)
            AddInputOptionST("INPUT_TOYS_ANIMATION_EVENT_NIPPLE", "Event Nipple Stimulation", TIntegration.Toys_Animation_Event_Nipple, toys_animation_event_flag)

            AddHeaderOption("Actions")
            Int toys_animation_rousing = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation
                toys_animation_rousing = OPTION_FLAG_NONE
            EndIf
            AddToggleOptionST("OPTION_TOYS_ANIMATION_ROUSING", "Rousing = Vibration Strength", TIntegration.Toys_Animation_Rousing, toys_animation_rousing)

            Int toys_animation_pattern_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation
                toys_animation_pattern_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.Toys_Animation_Pattern], toys_animation_pattern_flag)
        
            Int toys_animation_funscript_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation && TIntegration.Toys_Animation_Pattern == 1
                toys_animation_funscript_flag = OPTION_FLAG_NONE
            EndIf
            AddMenuOptionST("MENU_TOYS_ANIMATION_FUNSCRIPT", "Vibrate Funscript", TIntegration.Toys_Animation_Funscript, toys_animation_funscript_flag)
        
            Int toys_animation_linear_flag = OPTION_FLAG_DISABLED
            If TIntegration.Toys_Animation && ! TIntegration.Toys_Animation_Rousing && TIntegration.Toys_Animation_Pattern == 0
                toys_animation_linear_flag = OPTION_FLAG_NONE
            EndIf
            AddSliderOptionST("SLIDER_TOYS_ANIMATION_LINEAR_STRENGTH", "Strength (Linear)", TIntegration.Toys_Animation_Linear_Strength, "{0}", toys_animation_linear_flag)

            AddHeaderOption("Extra Actions")
            AddToggleOptionST("OPTION_TOYS_DENIAL", "Pause on Denial", TIntegration.Toys_Denial)
            AddToggleOptionST("OPTION_TOYS_VAGINAL_PENETRATION", "Strong Vaginal Penetration", TIntegration.Toys_Vaginal_Penetration)
            AddToggleOptionST("OPTION_TOYS_ANAL_PENETRATION", "Strong Anal Penetration", TIntegration.Toys_Anal_Penetration)
            AddToggleOptionST("OPTION_TOYS_ORAL_PENETRATION", "Strong Vaginal Penetration", TIntegration.Toys_Oral_Penetration)
            AddToggleOptionST("OPTION_TOYS_FONDLE", "Vibration on Fondle", TIntegration.Toys_Fondle)
            AddToggleOptionST("OPTION_TOYS_SQUIRT", "Vibration on Squirt", TIntegration.Toys_Squirt)
        Else
            AddTextOption("Toys & Love", "Mod not found", OPTION_FLAG_DISABLED)
        EndIf
    EndIf

    If page == "Skyrim Chain Beasts"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Gemmed Beasts")
        AddToggleOptionST("OPTION_CHAINBEASTS_VIBRATE", "Enable", TIntegration.Chainbeasts_Vibrate)
        Int chainbeasts_vibrate_selector_flag = OPTION_FLAG_DISABLED
        If TIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_selector_flag = OPTION_FLAG_NONE
        EndIf
        
        AddHeaderOption("Devices")
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_DEVICE_SELECTOR", "Filter", _DeviceSelectorOptions[TIntegration.Chainbeasts_Vibrate_DeviceSelector], chainbeasts_vibrate_selector_flag)

        Int chainbeasts_vibrate_event_flag = OPTION_FLAG_DISABLED
        If TIntegration.Chainbeasts_Vibrate && TIntegration.Chainbeasts_Vibrate_DeviceSelector == 1
            chainbeasts_vibrate_event_flag = OPTION_FLAG_NONE
        EndIf
        AddInputOptionST("INPUT_CHAINBEASTS_VIBRATE_EVENT", "Match Event", TIntegration.Chainbeasts_Vibrate_Event, chainbeasts_vibrate_event_flag)

        AddHeaderOption("Action")
        Int chainbeasts_vibrate_pattern_flag = OPTION_FLAG_DISABLED
        If TIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_pattern_flag = OPTION_FLAG_NONE
        EndIf
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_PATTERN", "Vibrate Pattern", _PatternSelectorOptions[TIntegration.Chainbeasts_Vibrate_Pattern], chainbeasts_vibrate_pattern_flag)

        Int chainbeast_vibrate_funscript_flag = OPTION_FLAG_DISABLED
        If TIntegration.Chainbeasts_Vibrate && TIntegration.Chainbeasts_Vibrate_Pattern == 1
            chainbeast_vibrate_funscript_flag = OPTION_FLAG_NONE
        EndIf
        AddMenuOptionST("MENU_CHAINBEASTS_VIBRATE_FUNSCRIPT", "Vibrate Funscript", TIntegration.Chainbeasts_Vibrate_Funscript, chainbeast_vibrate_funscript_flag)

        Int chainbeasts_vibrate_linear_flag = OPTION_FLAG_DISABLED
        If TIntegration.Chainbeasts_Vibrate
            chainbeasts_vibrate_linear_flag = OPTION_FLAG_NONE
        EndIf
	    AddSliderOptionST("SLIDER_CHAINBEASTS_VIBRATE_LINEAR_STRENGTH", "Strength", TIntegration.Chainbeasts_Vibrate_Linear_Strength, "{0}", chainbeasts_vibrate_linear_flag)
    EndIf
    
    If page == "Funscript Patterns"
        SetCursorFillMode(TOP_TO_BOTTOM)
        AddHeaderOption("Vibrator Patterns")
        Int j = 0
        While j < _VibrateFunscriptNames.Length && j < 63
            String vibrate_pattern = _VibrateFunscriptNames[j]
            _TestVibratePatternOid[j] = AddTextOption(vibrate_pattern, "(test me)")
            j += 1
        EndWhile

        SetCursorPosition(1)
        AddHeaderOption("Stroker Patterns")
        Int i = 0
        While i < _StrokeFunscriptNames.Length && i < 63
            String stroker_pattern = _StrokeFunscriptNames[i]
            _TestStrokePatternOid[i] = AddTextOption(stroker_pattern, "(test me)")
            i += 1
        EndWhile
    Endif

    If page == "Debug"
        SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Logging")
        AddToggleOptionST("OPTION_LOG_CONNECTS", "Device connects/disconnects", TDevices.LogDeviceConnects)
        AddToggleOptionST("OPTION_LOG_EVENTS", "Device Starts Moving", TDevices.LogDeviceEvents)
        AddToggleOptionST("OPTION_LOG_EVENTS_ENDS", "Device Stops Moving", TDevices.LogDeviceEvents)
        AddToggleOptionST("OPTION_LOG_DEBUG", "Other messages", TDevices.LogDebugEvents)

        AddHeaderOption("Spells")
        AddToggleOptionST("ACTION_ADD_SPELLS_TO_PLAYER", "Learn debug spells", _DebugSpellsAdded)
    EndIf

    If page == "Troubleshooting"
        SetCursorFillMode(TOP_TO_BOTTOM)
        AddTextOptionST("HELP_DEVICE_NOT_CONNECTING", "Device not connecting", "Read below")
        AddTextOptionST("HELP_DEVICE_NOT_VIBRATING", "Device not vibrating", "Read below")
        AddTextOptionST("HELP_DEVICE_ERRORS", "Device Errors", "Read below")
    EndIf
EndEvent

; General
State CONNECTION_MENU
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TDevices.ConnectionType)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_ConnectionMenuOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TDevices.ConnectionType = index
        SetMenuOptionValueST(_ConnectionMenuOptions[index])
        Debug.MessageBox("Please reconnect now!")
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TDevices.ConnectionType = 0
        SetMenuOptionValueST(_ConnectionMenuOptions[TDevices.ConnectionType])
    EndEvent

    Event OnHighlightST()
        String t = "Specify how Telekinesis performs its device control\n"
        t += "- In-Process: Control devices directly through Telekinesis (Recommended)\n"
        t += "- Intiface (WebSocket): Control Devices through a running Intiface App (See docs)\n"
        t += "NOTE: Don't change this if you don't know what it does\n"
        SetInfoText(t)
    EndEvent
EndState

State CONNECTION_STATUS
    Event OnSelectST()
    EndEvent

    Event OnHighlightST()
        String errorDetails = ""
        String status = TDevices.GetConnectionStatus()
        If status == "Failed"
            errorDetails = "\nConnection failed, double check parameters or check 'My Games/Skyrim Special Edition/SKSE/Telekinesis.log' \nError: " + TDevices.ConnectionErrorDetails
        EndIf
        SetInfoText("Connection Status: " + status + errorDetails)
    EndEvent
EndState

State CONNECTION_HOST
	Event OnInputOpenST()
		SetInputDialogStartText(TDevices.WsHost)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TDevices.WsHost = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The host-name of your Intiface Web-Socket Endpoint (check Intiface App). Default: 127.0.0.1")
    EndEvent
EndState

State CONNECTION_PORT
	Event OnInputOpenST()
		SetInputDialogStartText(TDevices.WsPort)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TDevices.WsPort = value
        Tele_Api.Cmd_1("connection.websocket", "127.0.0.1:12345")
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The port your Intiface Web-Socket Endpoint (check Intiface App) Default: 12345")
    EndEvent
EndState

State ACTION_RECONNECT
    Event OnSelectST()
        SetTextOptionValueST("Reconnecting now...")
        TDevices.Reconnect()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Disconnect and re-connect all device connections")
    EndEvent
EndState

State EMERGENCY_STOP
    Event OnSelectST()
        SetTextOptionValueST("Stopping...")
        Tele_Api.Cmd("stop_all")
    EndEvent

    Event OnHighlightST()
        SetInfoText("Immediately stop all devices from moving")
    EndEvent
EndState

State EMERGENCY_HOTKEY
    Event OnKeyMapChangeST(int newKeyCode, string conflictControl, string conflictName)
        TIntegration.EmergencyHotkey = newKeyCode
        SetKeyMapOptionValueST(TIntegration.EmergencyHotkey)
    EndEvent

    Event OnDefaultST()
        TIntegration.EmergencyHotkey = TIntegration.EmergencyHotkey_Default
        SetKeyMapOptionValueST(TIntegration.EmergencyHotkey)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A hotkey for immediately stopping all devices from moving (Default: DEL)")
    EndEvent
EndState

; Devices 

State ACTION_SCAN_FOR_DEVICES
    Event OnSelectST()
        If TDevices.ScanningForDevices
            Tele_Api.Cmd("stop_scan")
        Else
            Tele_Api.Cmd("start_scan")
        EndIf
        TDevices.ScanningForDevices = !TDevices.ScanningForDevices
        SetToggleOptionValueST(TDevices.ScanningForDevices)
    EndEvent
    
    Event OnDefaultST()
        TDevices.ScanningForDevices = true
        SetToggleOptionValueST(TDevices.ScanningForDevices)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Automatically scan for new devices (resets to 'true' on each restart)")
    EndEvent
EndState

; Devious Devices

State OPTION_DEVIOUS_DEVICES_VIBRATE
    Event OnSelectST()
        TIntegration.DeviousDevices_Vibrate = !TIntegration.DeviousDevices_Vibrate
        SetToggleOptionValueST(TIntegration.DeviousDevices_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.DeviousDevices_Vibrate = TIntegration.DeviousDevices_Vibrate_Default
        SetToggleOptionValueST(TIntegration.DeviousDevices_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Enable vibration support for devious devices in-game vibrators")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICES_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.DeviousDevices_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.DeviousDevices_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.DeviousDevices_Vibrate_DeviceSelector = TIntegration.DeviousDevices_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.DeviousDevices_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Body Parts' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.DeviousDevices_Vibrate_Event_Anal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.DeviousDevices_Vibrate_Event_Anal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Change the event that is triggered for in-game 'Anal' devices (vibrating buttplugs). Default: Anal")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.DeviousDevices_Vibrate_Event_Nipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.DeviousDevices_Vibrate_Event_Nipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Change the event that is triggered for in-game 'Nipple' devices (vibrating piercings). Default: Nipple")
    EndEvent
EndState

State OPTION_DEVIOUS_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.DeviousDevices_Vibrate_Event_Vaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.DeviousDevices_Vibrate_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Change event that is triggered for in-game 'Vaginal' devices, i. clitoral piercings,. Default: Vaginal")
    EndEvent
EndState

State MENU_DEVIOUS_DEVICES_VIBRATE_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.DeviousDevices_Vibrate_Pattern = index
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
        TIntegration.DeviousDevices_Vibrate_Funscript = _VibrateFunscriptNames[index]
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
        TIntegration.Sexlab_Animation = !TIntegration.Sexlab_Animation
        SetToggleOptionValueST(TIntegration.Sexlab_Animation)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_Animation = TIntegration.Sexlab_Animation_Default
        SetToggleOptionValueST(TIntegration.Sexlab_Animation)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during sexlab player animation")
    EndEvent
EndState

State MENU_SEXLAB_ANIMATION_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Sexlab_Animation_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Sexlab_Animation_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Sexlab_Animation_DeviceSelector = TIntegration.Sexlab_Animation_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Sexlab_Animation_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "Set to 'Match Body Parts' when you only want to vibrate devices that match any of the sexlab animation tags\n"
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
        TIntegration.Sexlab_Animation_Pattern = index
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
        TIntegration.Sexlab_Animation_Funscript = _VibrateFunscriptNames[index]
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
        TIntegration.Sexlab_Animation_Rousing = !TIntegration.Sexlab_Animation_Rousing
        SetToggleOptionValueST(TIntegration.Sexlab_Animation_Rousing)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_Animation_Rousing = TIntegration.Sexlab_Animation_Rousing_Default
        SetToggleOptionValueST(TIntegration.Sexlab_Animation_Rousing)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibration strength is controlled by SLA Arousal: 10 = 10% strength, 100 = 100% strength...")
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_ORGASM
    Event OnSelectST()
        TIntegration.Sexlab_ActorOrgasm = !TIntegration.Sexlab_ActorOrgasm
        SetToggleOptionValueST(TIntegration.Sexlab_ActorOrgasm)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_ActorOrgasm = TIntegration.Sexlab_ActorOrgasm_Default
        SetToggleOptionValueST(TIntegration.Sexlab_ActorOrgasm)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Start an additional (stronger) vibration on all matching devices whenever the player orgasms. This will override/enhance existing vibrations.")
    EndEvent
EndState

State OPTION_SEXLAB_ACTOR_EDGE
    Event OnSelectST()
        TIntegration.Sexlab_ActorEdge = !TIntegration.Sexlab_ActorEdge
        SetToggleOptionValueST(TIntegration.Sexlab_ActorEdge)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_ActorEdge = TIntegration.Sexlab_ActorEdge_Default
        SetToggleOptionValueST(TIntegration.Sexlab_ActorEdge)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Stop the vibration on all matching devices for a short time whenever the player edges. This will override existing vibrations.")
    EndEvent
EndState

State OPTION_SEXLAB_STROKER
    Event OnSelectST()
        TIntegration.Sexlab_Stroker = !TIntegration.Sexlab_Stroker
        SetToggleOptionValueST(TIntegration.Sexlab_Stroker)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_Stroker = TIntegration.Sexlab_Stroker_Default
        SetToggleOptionValueST(TIntegration.Sexlab_Stroker)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move linear/positional devices during sexlab player animation")
    EndEvent
EndState

State OPTION_SEXLAB_OSCILLATORS
    Event OnSelectST()
        TIntegration.Sexlab_Oscillator = !TIntegration.Sexlab_Oscillator
        SetToggleOptionValueST(TIntegration.Sexlab_Oscillator)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_Oscillator = TIntegration.Sexlab_Oscillator_Default
        SetToggleOptionValueST(TIntegration.Sexlab_Oscillator)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String info = "Oscillates devices during sexlab player animation\n"
        info += "NOTE: Oscillators are specific strokers that do not accept positional input\n"
        info += "and instead operate with a scalar speed value (like vibrators). These do not support\n"
        info += "playing funscripts and are always controlled by rousing or speed."
        SetInfoText(info)
    EndEvent
EndState


State MENU_SEXLAB_STROKER_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Sexlab_Stroker_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Sexlab_Stroker_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Sexlab_Stroker_DeviceSelector = TIntegration.Sexlab_Stroker_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Sexlab_Stroker_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "Set to 'Match Body Parts' when you only want to move strokers that match any of the sexlab animation tags\n"
        txt += "Note: Will match any tag, but Anal, Boobjob, Vaginal, Masturbation, Oral are probably the events you want to associate with your devices"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_SEXLAB_STROKER_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptionsStroker)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Sexlab_Stroker_Pattern = index
        SetMenuOptionValueST(_PatternSelectorOptionsStroker[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_PatternSelectorOptionsStroker[0])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("'Funscript': Stroker is controlled by a named funscript file. 'Random Funscript': Use a randomly selected funscript.")
    EndEvent
EndState

State MENU_SEXLAB_STROKER_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_StrokeFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Sexlab_Stroker_Funscript = _StrokeFunscriptNames[index]
        SetMenuOptionValueST(_StrokeFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_StrokeFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State OPTION_SEXLAB_STROKER_ROUSING
    Event OnSelectST()
        TIntegration.Sexlab_Stroker_Rousing = !TIntegration.Sexlab_Stroker_Rousing
        SetToggleOptionValueST(TIntegration.Sexlab_Stroker_Rousing)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Sexlab_Stroker_Rousing = TIntegration.Sexlab_Stroker_Rousing_Default
        SetToggleOptionValueST(TIntegration.Sexlab_Stroker_Rousing)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibration strength is controlled by SLA Arousal: 10 = 10% strength, 100 = 100% strength...")
    EndEvent
EndState

; OStim

State OPTION_OSTIM_ANIMATION
    Event OnSelectST()
        TIntegration.Ostim_Animation = !TIntegration.Ostim_Animation
        SetToggleOptionValueST(TIntegration.Ostim_Animation)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Ostim_Animation = TIntegration.Ostim_Animation_Default
        SetToggleOptionValueST(TIntegration.Ostim_Animation)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move devices during OStim player animation")
    EndEvent
EndState

State MENU_OSTIM_ANIMATION_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Ostim_Animation_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Ostim_Animation_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Ostim_Animation_DeviceSelector = TIntegration.Ostim_Animation_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Ostim_Animation_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "By default vibrate 'All' devices on sexual OStim scenes (anything involving vagina,anal,nipple or penis)\n" 
        txt += "'Match Body Parts' will only vibrate devices that match specific body parts (configured below and in 'Devices' Page)\n"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_OSTIM_STROKER_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Ostim_Stroker_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Ostim_Stroker_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Ostim_Stroker_DeviceSelector = TIntegration.Ostim_Stroker_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Ostim_Stroker_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "By default strokes 'All' devices on sexual OStim scenes (anything involving vagina,anal,nipple or penis)\n" 
        txt += "'Match Body Parts' will only stroke devices that match specific body parts (configured below and in 'Devices' Page)\n"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_OSTIM_STROKER_SPEED
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_OstimSpeedOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Stroker_Speed_Control = index
        SetMenuOptionValueST(_OstimSpeedOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_OstimSpeedOptions[TIntegration.Ostim_Stroker_Speed_Control_Default])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "Configure dynamic speed based on either animation speed, excitement or both combined. Default: Speed\n"
        txt += "Speed: Animation Speed 1/4 = 25%, Animation Speed 2/4 = 50%...\n"
        txt += "Excitement: Excitement 1% = 1%, Excitement 50% = 50%"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_OSTIM_STROKER_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptionsStroker)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Stroker_Pattern = index
        SetMenuOptionValueST(_PatternSelectorOptionsStroker[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_PatternSelectorOptionsStroker[0])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("'Funscript': Vibration is controlled by a named funscript file. 'Random Funscript': Use a randomly selected funscript.")
    EndEvent
EndState

State MENU_OSTIM_STROKER_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_StrokeFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Stroker_Funscript = _StrokeFunscriptNames[index]
        SetMenuOptionValueST(_StrokeFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_StrokeFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.funscript")
    EndEvent
EndState

State MENU_OSTIM_ANIMATION_SPEED
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_OstimSpeedOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Animation_Speed_Control = index
        SetMenuOptionValueST(_OstimSpeedOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_OstimSpeedOptions[TIntegration.Ostim_Animation_Speed_Control_Default])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String txt = "Configure dynamic speed control based on either animation speed, excitement or both combined. Default: Speed\n"
        txt += "Speed: Animation Speed 1/4 = 25%, Animation Speed 2/4 = 50%...\n"
        txt += "Excitement: Excitement 1% = 1%, Excitement 50% = 50%"
        SetInfoText(txt)
    EndEvent
EndState

State MENU_OSTIM_ANIMATION_PATTERN
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_PatternSelectorOptions)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Animation_Pattern = index
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

State MENU_OSTIM_ANIMATION_FUNSCRIPT
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(0)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_VibrateFunscriptNames)
    EndEvent

    Event OnMenuAcceptST(int index)
        TIntegration.Ostim_Animation_Funscript = _VibrateFunscriptNames[index]
        SetMenuOptionValueST(_VibrateFunscriptNames[index])
    EndEvent

    Event OnDefaultST()
        SetMenuOptionValueST(_VibrateFunscriptNames[0])
    EndEvent

    Event OnHighlightST()
        SetInfoText("Select a funscript pattern. Patterns are stored in Data/SKSE/Plugins/Telekinesis/Patterns/*.vibration.funscript")
    EndEvent
EndState

State OSTIM_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Ostim_Animation_Event_Anal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Ostim_Animation_Event_Anal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered on in-game anal stimulation of the player. Default: Anal")
    EndEvent
EndState

State OSTIM_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Ostim_Animation_Event_Nipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Ostim_Animation_Event_Nipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered on in-game nipple stimulation of the player. Default: Nipple")
    EndEvent
EndState

State OSTIM_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Ostim_Animation_Event_Vaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Ostim_Animation_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("The event that is triggered for in-game vaginal stimulation of the player. Default: Vaginal")
    EndEvent
EndState

State OSTIM_EVENT_PENETRATION
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Ostim_Animation_Event_Penetration)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Ostim_Animation_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Triggered when the player penetrates an in-game character, i.e NPC fucking the player. Default: Penetration")
    EndEvent
EndState

State OSTIM_EVENT_PENIS
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Ostim_Animation_Event_Penis)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Ostim_Animation_Event_Penis = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Triggered on in-game penis stimulation, i.e. Player penetrating an NPC or an NPC pleasuring the player. Default: Penis")
    EndEvent
EndState

State OPTION_OSTIM_STROKER
    Event OnSelectST()
        TIntegration.Ostim_Stroker = !TIntegration.Ostim_Stroker
        SetToggleOptionValueST(TIntegration.Ostim_Stroker)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Ostim_Stroker = TIntegration.Ostim_Stroker_Default
        SetToggleOptionValueST(TIntegration.Ostim_Stroker)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Move linear/positional devices during sexlab player animation")
    EndEvent
EndState

State OPTION_OSTIM_OSCILLATOR
    Event OnSelectST()
        TIntegration.Ostim_Oscillator = !TIntegration.Ostim_Oscillator
        SetToggleOptionValueST(TIntegration.Ostim_Oscillator)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Ostim_Oscillator = TIntegration.Ostim_Oscillator_Default
        SetToggleOptionValueST(TIntegration.Ostim_Oscillator)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String info = "Oscillates devices during ostim player animation.\n"
        info += "NOTE: Oscillators are specific strokers that do not accept positional input\n"
        info += "and instead operate with a scalar speed value (like vibrators). These do not support\n"
        info += "playing funscripts and are always controlled by rousing or speed."
        SetInfoText(info)
    EndEvent
EndState

; Toys & Love

State OPTION_TOYS_VIBRATE
    Event OnSelectST()
        TIntegration.Toys_Vibrate = !TIntegration.Toys_Vibrate
        SetToggleOptionValueST(TIntegration.Toys_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Vibrate = TIntegration.Toys_Vibrate_Default
        SetToggleOptionValueST(TIntegration.Toys_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Sync with Toys & Love in-game vibrators (toys pulsate start/stop)")
    EndEvent
EndState

State MENU_TOYS_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Toys_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Toys_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Toys_Vibrate_DeviceSelector = TIntegration.Toys_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Toys_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Body Parts' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_TOYS_VIBRATE_EVENT
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Toys_Vibrate_Event)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Toys_Vibrate_Event = value
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
        TIntegration.Toys_Vibrate_Pattern = index
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
        TIntegration.Toys_Vibrate_Funscript = _VibrateFunscriptNames[index]
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
		SetSliderDialogStartValue(TIntegration.Toys_Vibrate_Linear_Strength)
		SetSliderDialogDefaultValue(TIntegration.Toys_Vibrate_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TIntegration.Toys_Vibrate_Linear_Strength = value as int
		SetSliderOptionValueST(TIntegration.Toys_Vibrate_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TIntegration.Toys_Vibrate_Linear_Strength = TIntegration.Toys_Vibrate_Linear_Strength_Default
		SetSliderOptionValueST(TIntegration.Toys_Vibrate_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
	EndEvent
EndState

State OPTION_TOYS_ANIMATION
    Event OnSelectST()
        TIntegration.Toys_Animation = !TIntegration.Toys_Animation
        SetToggleOptionValueST(TIntegration.Toys_Animation)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Animation = TIntegration.Toys_Animation_Default
        SetToggleOptionValueST(TIntegration.Toys_Animation)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Enable vibration during Toys & Love Sex animations")
    EndEvent
EndState

State MENU_TOYS_ANIMATION_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Toys_Animation_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Toys_Animation_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Toys_Animation_DeviceSelector = TIntegration.Toys_Animation_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Toys_Animation_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Body Parts' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_VAGINAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Toys_Animation_Event_Vaginal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Toys_Animation_Event_Vaginal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Pussy' or 'Vaginal' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_ORAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Toys_Animation_Event_ORAL)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Toys_Animation_Event_ORAL = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Oral' or 'Blowjob' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_ANAL
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Toys_Animation_Event_Anal)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Toys_Animation_Event_Anal = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Anal' tags")
    EndEvent
EndState

State INPUT_TOYS_ANIMATION_EVENT_NIPPLE
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Toys_Animation_Event_Nipple)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Toys_Animation_Event_Nipple = value
		SetInputOptionValueST(value)
	EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrate devices matching this event when animation contains 'Nipple' or 'Breast' tags")
    EndEvent
EndState

State OPTION_TOYS_ANIMATION_ROUSING
    Event OnSelectST()
        TIntegration.Toys_Animation_Rousing = !TIntegration.Toys_Animation_Rousing
        SetToggleOptionValueST(TIntegration.Toys_Animation_Rousing)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Animation_Rousing = TIntegration.Toys_Animation_Rousing_Default
        SetToggleOptionValueST(TIntegration.Toys_Animation_Rousing)
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
        TIntegration.Toys_Animation_Pattern = index
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
        TIntegration.Toys_Animation_Funscript = _VibrateFunscriptNames[index]
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
		SetSliderDialogStartValue(TIntegration.Toys_Animation_Linear_Strength)
		SetSliderDialogDefaultValue(TIntegration.Toys_Animation_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TIntegration.Toys_Animation_Linear_Strength = value as int
		SetSliderOptionValueST(TIntegration.Toys_Animation_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TIntegration.Toys_Animation_Linear_Strength = TIntegration.Toys_Animation_Linear_Strength_Default
		SetSliderOptionValueST(TIntegration.Toys_Animation_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
	EndEvent
EndState

State OPTION_TOYS_DENIAL
    Event OnSelectST()
        TIntegration.Toys_Denial = !TIntegration.Toys_Denial
        SetToggleOptionValueST(TIntegration.Toys_Denial)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Denial = TIntegration.Toys_Denial_Default
        SetToggleOptionValueST(TIntegration.Toys_Denial)
    EndEvent

    Event OnHighlightST()
        SetInfoText("'Rewards' a successfull 'denial' event with a 7s long stop period (no device will vibrate)")
    EndEvent
EndState

State OPTION_TOYS_VAGINAL_PENETRATION
    Event OnSelectST()
        TIntegration.Toys_Vaginal_Penetration = !TIntegration.Toys_Vaginal_Penetration
        SetToggleOptionValueST(TIntegration.Toys_Vaginal_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Vaginal_Penetration = TIntegration.Toys_Vaginal_Penetration_Default
        SetToggleOptionValueST(TIntegration.Toys_Vaginal_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Vaginal' event/tag on 'vaginal penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_ANAL_PENETRATION
    Event OnSelectST()
        TIntegration.Toys_Anal_Penetration = !TIntegration.Toys_Anal_Penetration
        SetToggleOptionValueST(TIntegration.Toys_Anal_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Anal_Penetration = TIntegration.Toys_Anal_Penetration_Default
        SetToggleOptionValueST(TIntegration.Toys_Anal_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Anal' event/tag on 'anal penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_ORAL_PENETRATION
    Event OnSelectST()
        TIntegration.Toys_Oral_Penetration = !TIntegration.Toys_Oral_Penetration
        SetToggleOptionValueST(TIntegration.Toys_Oral_Penetration)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Oral_Penetration = TIntegration.Toys_Oral_Penetration_Default
        SetToggleOptionValueST(TIntegration.Toys_Oral_Penetration)
    EndEvent

    Event OnHighlightST()
        String t = "Emits a strong 12s vibration on 'Oral' event/tag on 'oral penetration' event.\n"
        t += "This will override the base animation pattern on all affected devices during that time."
        SetInfoText(t)
    EndEvent
EndState

State OPTION_TOYS_FONDLE
    Event OnSelectST()
        TIntegration.Toys_Fondle = !TIntegration.Toys_Fondle
        SetToggleOptionValueST(TIntegration.Toys_Fondle)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Fondle = TIntegration.Toys_Fondle_Default
        SetToggleOptionValueST(TIntegration.Toys_Fondle)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A light vibration on all devices during the 'fondle' event")
    EndEvent
EndState

State OPTION_TOYS_SQUIRT
    Event OnSelectST()
        TIntegration.Toys_Squirt = !TIntegration.Toys_Squirt
        SetToggleOptionValueST(TIntegration.Toys_Squirt)
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Toys_Squirt = TIntegration.Toys_Squirt_Default
        SetToggleOptionValueST(TIntegration.Toys_Squirt)
    EndEvent

    Event OnHighlightST()
        SetInfoText("A strong 12s vibration on each 'squirt' event")
    EndEvent
EndState

; Chainbeasts

State OPTION_CHAINBEASTS_VIBRATE
    Event OnSelectST()
        TIntegration.Chainbeasts_Vibrate = !TIntegration.Chainbeasts_Vibrate
        SetToggleOptionValueST(TIntegration.Chainbeasts_Vibrate)
        ForcePageReset()
    EndEvent
    
    Event OnDefaultST()
        TIntegration.Chainbeasts_Vibrate = TIntegration.Chainbeasts_Vibrate_Default
        SetToggleOptionValueST(TIntegration.Chainbeasts_Vibrate)
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        SetInfoText("Vibrates devices during gemmed chainbeast capture (Requires V. >= 0.7.0 and recompiled SCB_VibeEffectScript.pex)")
    EndEvent
EndState

State MENU_CHAINBEASTS_VIBRATE_DEVICE_SELECTOR
    Event OnMenuOpenST()
        SetMenuDialogStartIndex(TIntegration.Chainbeasts_Vibrate_DeviceSelector)
        SetMenuDialogDefaultIndex(0)
        SetMenuDialogOptions(_DeviceSelectorOptions)
    EndEvent

    event OnMenuAcceptST(int index)
        TIntegration.Chainbeasts_Vibrate_DeviceSelector = index
        SetMenuOptionValueST(_DeviceSelectorOptions[index])
        ForcePageReset()
    EndEvent

    Event OnDefaultST()
        TIntegration.Chainbeasts_Vibrate_DeviceSelector = TIntegration.Chainbeasts_Vibrate_DeviceSelector_Default
        SetMenuOptionValueST(_DeviceSelectorOptions[TIntegration.Chainbeasts_Vibrate_DeviceSelector])
        ForcePageReset()
    EndEvent

    Event OnHighlightST()
        String text = "Set to 'Match Body Parts' if you only want to vibrate devices that correspond to a matching in-game item\n"
        SetInfoText(text)
    EndEvent
EndState

State INPUT_CHAINBEASTS_VIBRATE_EVENT
	Event OnInputOpenST()
		SetInputDialogStartText(TIntegration.Chainbeasts_Vibrate_Event)
	EndEvent
	
	Event OnInputAcceptST(String value)
		TIntegration.Chainbeasts_Vibrate_Event = value
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
        TIntegration.Chainbeasts_Vibrate_Pattern = index
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
        TIntegration.Chainbeasts_Vibrate_Funscript = _VibrateFunscriptNames[index]
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
		SetSliderDialogStartValue(TIntegration.Chainbeasts_Vibrate_Linear_Strength)
		SetSliderDialogDefaultValue(TIntegration.Chainbeasts_Vibrate_Linear_Strength_Default)
		SetSliderDialogRange(0, 100)
		SetSliderDialogInterval(1)
	EndEvent

	Event OnSliderAcceptST(float value)
		TIntegration.Chainbeasts_Vibrate_Linear_Strength = value as int
		SetSliderOptionValueST(TIntegration.Chainbeasts_Vibrate_Linear_Strength)
	EndEvent

	Event OnDefaultST()
		TIntegration.Chainbeasts_Vibrate_Linear_Strength = TIntegration.Chainbeasts_Vibrate_Linear_Strength_Default
		SetSliderOptionValueST(TIntegration.Chainbeasts_Vibrate_Linear_Strength)
	EndEvent

	Event OnHighlightST()
		SetInfoText("Vibration strength for linear pattern")
	EndEvent
EndState

; Debug

State OPTION_LOG_CONNECTS
    Event OnSelectST()
        TDevices.LogDeviceConnects = !TDevices.LogDeviceConnects
        SetToggleOptionValueST(TDevices.LogDeviceConnects)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(TDevices.LogDeviceConnects)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show notification when a device connects/disconnects")
    EndEvent
EndState

State OPTION_LOG_EVENTS_ENDS
    Event OnSelectST()
        TDevices.LogDeviceEventEnd = !TDevices.LogDeviceEventEnd
        SetToggleOptionValueST(TDevices.LogDeviceEventEnd)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(TDevices.LogDeviceEventEnd)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show notification when device stops moving (vibration etc.)")
    EndEvent
EndState

State OPTION_LOG_EVENTS
    Event OnSelectST()
        TDevices.LogDeviceEvents = !TDevices.LogDeviceEvents
        SetToggleOptionValueST(TDevices.LogDeviceEvents)
    EndEvent
    
    Event OnDefaultST()
        SetToggleOptionValueST(TDevices.LogDeviceEvents)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show notification when device starts moving (vibration etc.)")
    EndEvent
EndState

State OPTION_LOG_DEBUG
    Event OnSelectST()
        TDevices.LogDebugEvents = !TDevices.LogDebugEvents
        SetToggleOptionValueST(TDevices.LogDebugEvents)
    EndEvent
    
    Event OnDefaultST()
        TDevices.LogDebugEvents = false
        SetToggleOptionValueST(TDevices.LogDebugEvents)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Show internal debug notifications")
    EndEvent
EndState

State ACTION_ADD_SPELLS_TO_PLAYER
    Event OnSelectST()
        Actor player = Game.GetPlayer()
        If ! _DebugSpellsAdded
            If ! player.HasSpell(TDevices.Tele_VibrateSpellWeak)
                player.AddSpell(TDevices.Tele_VibrateSpellWeak)
            EndIf
            If ! player.HasSpell(TDevices.Tele_VibrateSpellMedium)
                player.AddSpell(TDevices.Tele_VibrateSpellMedium)
            EndIf
            If ! player.HasSpell(TDevices.Tele_VibrateSpellStrong)
                player.AddSpell(TDevices.Tele_VibrateSpellStrong)
            EndIf
            If ! player.HasSpell(TDevices.Tele_Stop)
                player.AddSpell(TDevices.Tele_Stop)
            EndIf
            _DebugSpellsAdded = true
        Else
            If player.HasSpell(TDevices.Tele_VibrateSpellWeak)
                player.RemoveSpell(TDevices.Tele_VibrateSpellWeak)
            EndIf
            If player.HasSpell(TDevices.Tele_VibrateSpellMedium)
                player.RemoveSpell(TDevices.Tele_VibrateSpellMedium)
            EndIf
            If player.HasSpell(TDevices.Tele_VibrateSpellStrong)
                player.RemoveSpell(TDevices.Tele_VibrateSpellStrong)
            EndIf
            If player.HasSpell(TDevices.Tele_Stop)
                player.RemoveSpell(TDevices.Tele_Stop)
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

State HELP_DEVICE_ERRORS
    Event OnSelectST()
    EndEvent
    Event OnHighlightST()
        String a = "Red device errors indicate that a device cannot be used:\n"
        String b = "1. Try to restart the device and check that is has enough battery\n"
        String c = "2. Check erors in 'My Gymes/Skyrim Special Edition/SKSE/Telekinesis.log'\n"
        String d = "3. Restart the connection in MCM"
        SetInfoText(a + b + c + d)
    EndEvent
EndState

Event OnOptionSelect(int oid)
    Int i = 0
    String device = ""
    While (i < 31 && i < _ActuatorIds.Length)
        If (oid == _UseDeviceOids[i])
            device = _ActuatorIds[i]
            Bool isUsed = ! Tele_Api.Qry_Bool_1("device.settings.enabled", device)
            SetToggleOptionValue(oid, isUsed)
            If isUsed
                Tele_Api.Cmd_1("device.settings.enable", device)
            Else
                Tele_Api.Cmd_1("device.settings.disable", device)
            EndIf
        EndIf
        If (oid == _StrokerInvertOid[i])
            device = _ActuatorIds[i]
            Bool isInverted = ! Tele_Api.Qry_Bool_1("device.linear.invert", device)
            SetToggleOptionValue(oid, isInverted)
            If isInverted
                Tele_Api.Cmd_1("device.linear.invert.enable", device)
            Else
                Tele_Api.Cmd_1("device.linear.invert.disable", device)
            EndIf
        EndIf
        i += 1
    EndWhile
    i = 0
    While (i < _TestVibratePatternOid.Length)
        If (oid == _TestVibratePatternOid[i])
            String patternName = _VibrateFunscriptNames[i]
            String[] allEvents = new String[1]
            TDevices.VibratePattern(patternName, 100, 30, allEvents)
		    SetTextOptionValue(_TestVibratePatternOid[i], "running...")
        EndIf
        i += 1
    EndWhile
    i = 0
    While (i < _TestStrokePatternOid.Length)
        If (oid == _TestStrokePatternOid[i])
            String patternName = _StrokeFunscriptNames[i]
            String[] allEvents = new String[1]
            TDevices.LinearPattern(patternName, 100, 30, allEvents)
		    SetTextOptionValue(_TestStrokePatternOid[i], "running...")
        EndIf
        i += 1
    EndWhile
    Tele_Api.Cmd("settings.store")
EndEvent

Event OnOptionSliderOpen(Int oid)
    Int i = 0
    While (i < _StrokerMinPosOid.Length)
        If (oid == _VibratorMinSpeedOid[i])
            Int minSpeed = Tele_Api.Qry_Str_1("device.scalar.min_speed", _ActuatorIds[i]) as Int
            SetSliderDialogStartValue(minSpeed)
            SetSliderDialogDefaultValue(0)
            SetSliderDialogRange(0, 100)
            SetSliderDialogInterval(1) 
        EndIf
        If (oid == _VibratorMaxSpeedOid[i])
            Int maxSpeed = Tele_Api.Qry_Str_1("device.scalar.max_speed", _ActuatorIds[i]) as Int
            SetSliderDialogStartValue(maxSpeed)
            SetSliderDialogDefaultValue(100)
            SetSliderDialogRange(0, 100)
            SetSliderDialogInterval(1) 
        EndIf
        If (oid == _VibratorFactorOid[i])
            Float factor = Tele_Api.Qry_Str_1("device.scalar.factor", _ActuatorIds[i]) as Float
            SetSliderDialogStartValue(factor)
            SetSliderDialogDefaultValue(1.0)
            SetSliderDialogRange(0, 1.0)
            SetSliderDialogInterval(0.01) 
        EndIf
        If (oid == _StrokerMinPosOid[i])
            Float minPos = Tele_Api.Qry_Str_1("device.linear.min_pos", _ActuatorIds[i]) as Float
            SetSliderDialogStartValue(minPos)
            SetSliderDialogDefaultValue(0)
            SetSliderDialogRange(0.0, 1.0)
            SetSliderDialogInterval(0.01) 
        EndIf
        If (oid == _StrokerMaxPosOid[i])
            Float maxPos = Tele_Api.Qry_Str_1("device.linear.max_pos", _ActuatorIds[i]) as Float
            SetSliderDialogStartValue(maxPos)
            SetSliderDialogDefaultValue(1)
            SetSliderDialogRange(0.0, 1.0)
            SetSliderDialogInterval(0.01)
        EndIf
        If (oid == _StrokerMinMsOid[i])
            Float minMs = Tele_Api.Qry_Str_1("device.linear.min_ms", _ActuatorIds[i]) as Float
            SetSliderDialogStartValue(minMs)
            SetSliderDialogDefaultValue(250)
            SetSliderDialogRange(50, 10000)
            SetSliderDialogInterval(10)
        EndIf
        If (oid == _StrokerMaxMsOid[i])
            Float maxMs = Tele_Api.Qry_Str_1("device.linear.max_ms", _ActuatorIds[i]) as Float
            SetSliderDialogStartValue(maxMs)
            SetSliderDialogDefaultValue(3000)
            SetSliderDialogRange(50, 10000)
            SetSliderDialogInterval(10)
        EndIf
        i += 1
    EndWhile
EndEvent

Event OnOptionSliderAccept(Int oid, Float value)
    Int i = 0
    While (i < _StrokerMinPosOid.Length)
        If (oid == _VibratorMinSpeedOid[i])
            SetSliderOptionValue(oid, value, "{0}%")
            Tele_Api.Cmd_2("device.scalar.min_speed", _ActuatorIds[i], value as Int)
        EndIf
        If (oid == _VibratorMaxSpeedOid[i])
            SetSliderOptionValue(oid, value, "{0}%")
            Tele_Api.Cmd_2("device.scalar.max_speed", _ActuatorIds[i], value as Int)
        EndIf
        If (oid == _VibratorFactorOid[i])
            SetSliderOptionValue(oid, value, "{2}")
            Tele_Api.Cmd_2("device.scalar.factor", _ActuatorIds[i], value)
        EndIf
        If (oid == _StrokerMinPosOid[i])
            SetSliderOptionValue(oid, value, "{2}")
            Tele_Api.Cmd_2("device.linear.min_pos", _ActuatorIds[i], value)
        EndIf
        If (oid == _StrokerMaxPosOid[i])
            SetSliderOptionValue(oid, value, "{2}")
            Tele_Api.Cmd_2("device.linear.max_pos", _ActuatorIds[i], value)
        EndIf
        If (oid == _StrokerMinMsOid[i])
            SetSliderOptionValue(oid, value, "{0} ms")
            Tele_Api.Cmd_2("device.linear.min_ms", _ActuatorIds[i], value as Int)
        EndIf
        If (oid == _StrokerMaxMsOid[i])
            SetSliderOptionValue(oid, value, "{0} ms")
            Tele_Api.Cmd_2("device.linear.max_ms", _ActuatorIds[i], value as Int)
        EndIf
        i += 1
    EndWhile
    Tele_Api.Cmd("settings.store")
EndEvent

Event OnOptionInputAccept(Int oid, String value)
    Int i = 0
    While (i < 31 && i < _ActuatorIds.Length)
        If (oid == _DeviceEventOids[i])
            Tele_Api.Cmd_2("device.settings.events", _ActuatorIds[i], value)
            SetInputOptionValue(oid, value)
        EndIf
        i += 1
    EndWhile
    Tele_Api.Cmd("settings.store")
EndEvent

Event OnOptionHighlight(Int oid)
    Int i = 0
    While (i < 31 && i < _ActuatorIds.Length)
        If (oid == _DeviceEventOids[i])  
            String infoText = "A comma-separated list of body parts associated with this device.\n"
            infoText += "By default, the terms 'Nipple', 'Vaginal', 'Anal', 'Penetration' are used to describe these, but any is possible.\n"
            infoText += "Example: Vaginal,Anal,Nipple\n"
            SetInfoText(infoText)
        EndIf
        If (oid == _StrokerMinPosOid[i]) 
            String infoText = "The hardware position that is considered a full penetration\n"
            infoText += "In funscripts, 0.0 is considered the lowest possible hardware position i.e. a full penetration\n"
            infoText += "Set it to a higher value to have less penetration, 0.1 = 10% less, 0.2 = 20% less..."
            SetInfoText(infoText)
        EndIf
        If (oid == _StrokerMaxPosOid[i])
            String infoText = "The hardware position that is considered the least penetration\n"
            infoText += "In funscripts, 1.0 is considered the highest possible hardware position i.e. the least penetration\n"
            infoText += "Set it to a lower value to have a lower min position, 0.8 = 20% lower, 0.7 = 30% lower..."
            SetInfoText(infoText)
        EndIf
        If (oid == _StrokerMinMsOid[i])
            String infoText = "The duration of the fastest possible stroke in milliseconds\n"
            infoText += "This setting is ignored in funscripts."
            SetInfoText(infoText)
        EndIf
        If (oid == _StrokerMaxMsOid[i])
            String infoText = "The duration of the slowest possible stroke in milliseconds\n"
            infoText += "This setting is ignored in funscripts."
            SetInfoText(infoText)
        EndIf
        If (oid == _StrokerInvertOid[i])
            String infoText = "Invert the position of all position points for this device\n"
            infoText += "Position 1.0 will become 0.0, position 0.2 will become 0.8 and so on...\n"
            infoText += "Enable this, in case you have a device that does not work with the default direction"
            SetInfoText(infoText)
        EndIf
        If (oid == _VibratorMinSpeedOid[i])
            String infoText = "Minimum vibration strength in percent\n"
            infoText += "Default: 0. Increase this, if your device does not work well with low speed\n"
            infoText += "This setting is ignored for 0% i.e. commands that stop the device."
            SetInfoText(infoText)
        EndIf
        If (oid == _VibratorMaxSpeedOid[i])
            String infoText = "Maximum vibration strength in percent\n"
            infoText += "Default 100. Decrease this, if your device is too strong"
            SetInfoText(infoText)
        EndIf
        i += 1
    EndWhile
EndEvent

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

; Deprecated

Tele_Devices Property TeleDevices Auto
Tele_Integration Property TeleIntegration Auto