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
	String[] evts = TK_Telekinesis.Tk_PollEvents();
	Int i = 0;
	While (i < evts.Length) 
		Log(evts[0])
		i += 1
	EndWhile
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("DeviceVibrateEffectStart")
	TK_Telekinesis.TK_VibrateAll(1.0)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("OnVibrateEffectStop")
	TK_Telekinesis.TK_VibrateAll(0.0)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction