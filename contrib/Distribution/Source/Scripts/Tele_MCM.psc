ScriptName Tele_MCM extends SKI_ConfigBase 

Tele_Devices Property TeleDevices Auto

Int mcmReconnectOid

Int currenctConnection = 0
Int ConnectionListId
String[] ConnectionList

Int function GetVersion()
	return -100
endFunction

event OnVersionUpdate(int aVersion)
    if CurrentVersion < aVersion
		Debug.Trace(self + " Updating MCM " + CurrentVersion + " to " + aVersion)
    EndIf
endEvent

Event OnConfigInit()
    ModName = "Telekinesis"
    Pages = new String[2]
    Pages[0] = "Connection"
    Pages[1] = "Devices"

    ConnectionList = new String[4]
	ConnectionList[0] = "In-Process (Default)"
	ConnectionList[1] = "Intiface Central"
	ConnectionList[2] = "G.I.F.T"
	ConnectionList[3] = "Test"
EndEvent

event OnOptionMenuOpen(Int aOption)
	if (aOption == connectionListId)
		SetMenuDialogStartIndex(currenctConnection)
		SetMenuDialogDefaultIndex(0)
		SetMenuDialogOptions(connectionList)
	endIf
endEvent

event OnOptionMenuAccept(Int aOption, Int aIndex)
	if (aOption == connectionListId)
		currenctConnection = aIndex
		SetMenuOptionValue(aOption, connectionList[currenctConnection])
	endIf
endEvent

Event OnPageReset(String page)

    If page == "Connection"
		SetCursorFillMode(TOP_TO_BOTTOM)
        AddHeaderOption("General")
        connectionListId = AddMenuOption("Type", connectionList[currenctConnection])
        mcmReconnectOid = AddToggleOption("Reconnect...", false)

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
        String[] devices = TeleDevices.GetDevices()
        Int i = 0;
        While (i < TeleDevices.DevicesLength) 
            String device = devices[i]

            String status = "Disconnected"
            Int flags = OPTION_FLAG_DISABLED
            If TeleDevices.GetConnected(device)
                status = "Connected"
                flags = OPTION_FLAG_NONE
            EndIf

            AddHeaderOption(device)
            AddTextOption( Key( i, "Connection"), status)
            AddToggleOption( Key( i, "Vibrator"),TeleDevices.CanVibrate(device), OPTION_FLAG_DISABLED)
            AddToggleOption( Key( i, "Use"), TeleDevices.GetUsed(device), flags)
            i += 1
        EndWhile 
    EndIf
EndEvent

String Function Key( String index, String name )
    return "[" + index + "] " + name
EndFunction
