ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto

Int selectedConnection = 0
String[] ConnectionMenuOptions
Int[] UseDeviceOids
String[] DeviceNames

Int Function GetVersion()
	return 3
EndFunction

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
        TeleDevices.LogDebug("Updating MCM from v" + CurrentVersion + " to v" + aVersion)
    EndIf
    If CurrentVersion < 3
        InitAll()
    EndIf
EndEvent

Event OnConfigInit()
    ModName = "Telekinesis"
    InitAll()
EndEvent

Function InitAll()
    Pages = new String[3]
    Pages[0] = "General"
    Pages[1] = "Devices"
    Pages[2] = "Debug"

    ConnectionMenuOptions = new String[3]
	ConnectionMenuOptions[0] = "In-Process (Default)"
	ConnectionMenuOptions[1] = "Intiface (WebSocket)" ; Not supported right now
    ConnectionMenuOptions[2] = "Disable"

    UseDeviceOids = new Int[20] ; Reserve mcm space for 5 fields per device

    DeviceNames = new String[1]
EndFunction

Event OnOptionSelect(int aOption)
    Int i = 0
    While (i < 31)
        If (aOption == UseDeviceOids[i])
            If (i < DeviceNames.Length)
                String device = DeviceNames[i]
                Bool isUsed = ! Tele_Api.GetEnabled(device)
                SetToggleOptionValue(aOption, isUsed)
                Tele_Api.SetEnabled(device, isUsed)
            EndIf
        EndIf
        i += 1
    EndWhile

    Tele_Api.SettingsStore()
EndEvent

Event OnPageReset(String page)
    If page == "General"
		SetCursorFillMode(TOP_TO_BOTTOM)

        AddTextOption("Version", TeleDevices.MajorVersion + "." + TeleDevices.MinorVersion, OPTION_FLAG_DISABLED)

        AddHeaderOption("Connection")
        AddMenuOptionST("CONNECTION_MENU", "Connection", ConnectionMenuOptions[selectedConnection])
        AddTextOptionST("ACTION_RECONNECT", "Reconnect...", "")

        AddHeaderOption("Emergency")
        AddTextOptionST("EMERGENCY_STOP", "Stop all devices", "")
    EndIf

    If page == "Devices"
		SetCursorFillMode(TOP_TO_BOTTOM)
        DeviceNames = Tele_Api.GetDevices()
        Int i = 0

        Int len = DeviceNames.Length
        If len > 20
            TeleDevices.LogError("Too many devices, ignoring some in MCM")
            len = 20
        EndIf

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
                UseDeviceOids[i] = AddToggleOption(Key(i, "Enabled"), Tele_Api.GetEnabled(name), flags)
            EndIf

            i += 1
        EndWhile

        If DeviceNames.Length == 0
            AddHeaderOption("No devices discovered yet...")
        EndIf
    EndIf

    If page == "Debug"
		SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("Logging")
        AddToggleOptionST("OPTION_LOG_CONNECTS", "Device Connects/Disconnects", TeleDevices.LogDeviceConnects)
        AddToggleOptionST("OPTION_LOG_EVENTS", "Device Events (Vibrations, etc.)", TeleDevices.LogDeviceEvents)
        AddToggleOptionST("OPTION_LOG_DEBUG", "Other messages", TeleDevices.LogDebugEvents)

        AddHeaderOption("Actions")
        AddToggleOptionST("ACTION_SCAN_FOR_DEVICES", "Scan for devices", TeleDevices.ScanningForDevices)
    EndIf
EndEvent

Bool property stoppingDeviceScan = false auto

State CONNECTION_MENU
	Event OnMenuOpenST()
		SetMenuDialogStartIndex(selectedConnection)
		SetMenuDialogDefaultIndex(0)
		SetMenuDialogOptions(ConnectionMenuOptions)
    EndEvent

	event OnMenuAcceptST(int index)
		selectedConnection = index
		SetMenuOptionValueST(ConnectionMenuOptions[selectedConnection])
        Debug.MessageBox("Reconnect now!")
	EndEvent

	Event OnDefaultST()
		selectedConnection = 0
		SetMenuOptionValueST(ConnectionMenuOptions[selectedConnection])
	EndEvent

	Event OnHighlightST()
		SetInfoText("Specifies how telekinesis connects to Buttplug.IO")
	EndEvent
EndState

State ACTION_RECONNECT
    Event OnSelectST()
        SetTextOptionValueST("Reconnecting now...")
        TeleDevices.Disconnect()
        Utility.Wait(5)
        TeleDevices.Connect()
        SetTextOptionValueST("Done!")
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


String Function Key( String index, String name )
    return "[" + index + "] " + name
EndFunction
