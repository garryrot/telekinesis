Scriptname TK_Test_DDVibrateEventObserver extends Quest  

Actor property Player auto

Event OnInit()
	Log("OnInit")
	ScanForDevices();
	RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
	RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
	RegisterForUpdate(2) ; Very short intervall for testing
EndEvent

Function ScanForDevices()
	Log("Scanning for devices...")
	TK_Telekinesis.TK_ScanForDevices();
EndFunction

Event OnUpdate()
	String evt = TK_Telekinesis.Tk_AwaitNextEvent();
	If (evt != "")
		Log(evt)
	EndIf
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("DeviceVibrateEffectStart")
	int vibrated = TK_Telekinesis.TK_StartVibrateAll(1.0)
	Log("Vibrating " + vibrated + " devices...")
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("OnVibrateEffectStop")
	int stopped = TK_Telekinesis.TK_StartVibrateAll(0.0)
	Log("Stopping " + stopped + " devices...")
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction