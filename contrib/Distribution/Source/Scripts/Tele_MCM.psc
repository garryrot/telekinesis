ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto

Int currenctConnection = 0
String[] ConnectionList
Int connectionOid

Int[] UseDeviceOids
String[] DeviceNames

Int Function GetVersion()
	return 1
EndFunction

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
		Debug.Trace(self + " Updating MCM " + CurrentVersion + " to " + aVersion)
    EndIf
    If CurrentVersion < 1
        InitAll()
    EndIf
EndEvent

Event OnConfigInit()
    ModName = "Telekinesis"
    InitAll()
EndEvent

Function InitAll()
    Pages = new String[3]
    Pages[0] = "Connection"
    Pages[1] = "Devices"
    Pages[2] = "Debug"

    ConnectionList = new String[4]
	ConnectionList[0] = "In-Process (Default)"
	ConnectionList[1] = "Intiface (WebSocket)" ; Not supported right now
	ConnectionList[2] = "Test Devices"         ; Not supported right now
 
    ; Reserve mcm space for 5 fields per device
    UseDeviceOids = new Int[20]

    DeviceNames = new String[1]
EndFunction

Event OnOptionSelect(int aOption)
    Int i = 0
    While (i < 31)
        If (aOption == UseDeviceOids[i])
            If (i < DeviceNames.Length)
                String device = DeviceNames[i]
                Bool isUsed = ! Tele.GetEnabled(device)
                SetToggleOptionValue(aOption, isUsed)
                Tele.SetEnabled(device, isUsed)
            EndIf
        EndIf
        i += 1
    EndWhile

    Tele.SettingsStore()
EndEvent

Event OnOptionMenuOpen(Int aOption)
	If (aOption == connectionOid)
		SetMenuDialogStartIndex(currenctConnection)
		SetMenuDialogDefaultIndex(0)
		SetMenuDialogOptions(connectionList)
    EndIf
EndEvent

Event OnOptionMenuAccept(Int aOption, Int aIndex)
	if (aOption == connectionOid)
		currenctConnection = aIndex
		SetMenuOptionValue(aOption, connectionList[currenctConnection])
        Debug.MessageBox("Reconnect now")
	endIf
EndEvent

Event OnPageReset(String page)
    If page == "Connection"
		SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("General")
        
        connectionOid = AddMenuOption("Type", connectionList[currenctConnection])
        AddToggleOptionST("ACTION_RECONNECT", "Reconnect...", false)

        ; AddHeaderOption("In-Process")
        ; AddEmptyOption()

        ; AddHeaderOption("Connectors")
        ; AddToggleOption("Bluetooth LE", true)
        ; AddToggleOption("Lovesense Connect", false, OPTION_FLAG_DISABLED)
        ; AddToggleOption("WebSocket-Connect", false, OPTION_FLAG_DISABLED)
        ; AddToggleOption("SerialPort", false, OPTION_FLAG_DISABLED)
        ; AddToggleOption("X-Input", false, OPTION_FLAG_DISABLED)
        ; AddEmptyOption()
    EndIf

    If page == "Devices"
		SetCursorFillMode(TOP_TO_BOTTOM)
        DeviceNames = Tele.GetDevices()
        Int i = 0

        Int len = DeviceNames.Length
        If len > 20
            TeleDevices.LogError("Too many devices, ignoring some in MCM")
            len = 20
        EndIf

        While (i < len) 
            String name = DeviceNames[i]
            
            If name != ""
                Bool connected = Tele.GetDeviceConnected(name)

                AddHeaderOption(name)
                String status = "Disconnected"
                If connected
                    status = "Connected"
                EndIf
                AddTextOption(Key(i, "Status"), status, OPTION_FLAG_DISABLED)
                AddTextOption(Key(i, "Actions"), Tele.GetDeviceCapabilities(name), OPTION_FLAG_DISABLED)

                Int flags = OPTION_FLAG_DISABLED
                If connected
                    flags = OPTION_FLAG_NONE
                EndIf
                UseDeviceOids[i] = AddToggleOption(Key(i, "Enabled"), Tele.GetEnabled(name), flags)
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

State ACTION_RECONNECT
    Event OnSelectST()
        SetToggleOptionValueST(true)
        Debug.MessageBox("Reconnecting now...")
        TeleDevices.Disconnect()
        Utility.Wait(5)
        TeleDevices.Connect()
        SetToggleOptionValueST(false)
    EndEvent
  
    Event OnDefaultST()
        SetToggleOptionValueST(false)
    EndEvent

    Event OnHighlightST()
        SetInfoText("Disconnect and re-connect all device connections")
    EndEvent
EndState

State OPTION_LOG_CONNECTS
    Event OnSelectST()
        TeleDevices.LogDeviceConnects = !TeleDevices.LogDeviceConnects
        SetToggleOptionValueST(TeleDevices.LogDeviceConnects)
    EndEvent
    
    Event OnDefaultST()
        TeleDevices.LogDeviceConnects = true
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
        TeleDevices.LogDeviceEvents = false
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
            Tele.StopScan()
        Else
            Tele.ScanForDevices()
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
