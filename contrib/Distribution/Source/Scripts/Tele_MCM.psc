ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto

Int currenctConnection = 0
String[] ConnectionList

Int connectionOid
Int reconnectOid
Int[] UseDeviceOids

Int function GetVersion()
	return 1
endFunction

Event OnVersionUpdate(int aVersion)
    If CurrentVersion < aVersion
		Debug.Trace(self + " Updating MCM " + CurrentVersion + " to " + aVersion)
    EndIf
    If CurrentVersion < 1
        UseDeviceOids = new Int[32]
    EndIf
EndEvent

Event OnConfigInit()
    TeleDevices.Log("Tele_MCM OnConfigInit")

    ModName = "Telekinesis"

    Pages = new String[2]
    Pages[0] = "Connection"
    Pages[1] = "Devices"

    ConnectionList = new String[4]
	ConnectionList[0] = "In-Process (Default)"
	ConnectionList[1] = "Intiface (WebSocket)" ; Not supported right now
	ConnectionList[2] = "Test Devices"         ; Not supported right now

    UseDeviceOids = new Int[32]
EndEvent

Event OnOptionSelect(int aOption)
    If (aOption == reconnectOid)
        SetToggleOptionValue(aOption, true)
        Debug.MessageBox("Reconnecting, close MCM now...")
        Tele.Close()
        Utility.Wait(5)
        Tele.ScanForDevices()
        SetToggleOptionValue(aOption, false)
        
    EndIf
    
    Int i = 0;
    While (i < 32)
        If (aOption == UseDeviceOids[i])
            Bool isUsed = ! TeleDevices.GetUsed(i)
            SetToggleOptionValue(aOption, isUsed)
            TeleDevices.SetUsed(i, isUsed)
        EndIf
        i += 1
    EndWhile
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
        Debug.MessageBox("Please reconnect now...")
	endIf
EndEvent

Event OnPageReset(String page)
    If page == "Connection"
		SetCursorFillMode(TOP_TO_BOTTOM)

        AddHeaderOption("General")

        connectionOid = AddMenuOption("Type", connectionList[currenctConnection])
        reconnectOid = AddToggleOption("Reconnect...", false)

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
        String[] names = TeleDevices.GetDevices()
        Int i = 0
        Int deviceCount = 0
        While (i < names.Length) 
            String name = names[i]
            
            If name != ""
                deviceCount += 1
                Bool connected = Tele.GetDeviceConnected(name)

                AddHeaderOption(name)
                AddTextOption(Key(i, "Connected"), connected, OPTION_FLAG_DISABLED)
                AddTextOption(Key(i, "Actions"), Tele.GetDeviceCapabilities(name), OPTION_FLAG_DISABLED)

                Int flags = OPTION_FLAG_DISABLED
                If connected
                    flags = OPTION_FLAG_NONE
                EndIf

                UseDeviceOids[i] = AddToggleOption(Key(i, "Use"), TeleDevices.IsUsed(name), flags)
            EndIf

            i += 1
        EndWhile

        If deviceCount == 0
            AddHeaderOption("No Devices Connected...")
        EndIf
    EndIf
EndEvent

String Function Key( String index, String name )
    return "[" + index + "] " + name
EndFunction
