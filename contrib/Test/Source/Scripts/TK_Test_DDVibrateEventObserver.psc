Scriptname TK_Test_DDVibrateEventObserver extends Quest  

Spell property VibrateSpell auto

Event OnInit()
	Log("OnInit")
	ScanForDevices();
	RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
	RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
	Game.GetPlayer().AddSpell(VibrateSpell);
	RegisterForUpdate(3) ; Very short intervall for testing
EndEvent

Function ScanForDevices()
	Log("Scanning for devices...")
	TK_Telekinesis.TK_ScanForDevices();
EndFunction

Event OnUpdate()
	String evt = TK_Telekinesis.Tk_PollEvents();
	If (evt != "")
		Log(evt)
	EndIf
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("DeviceVibrateEffectStart")
	TK_Telekinesis.TK_StartVibrateAll(1.0)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("OnVibrateEffectStop")
	TK_Telekinesis.TK_StartVibrateAll(0.0)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction