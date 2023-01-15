Scriptname TK_Test_DDVibrateEventObserver extends Quest  

Spell property VibrateSpell auto
Bool InSexlabScene = False

Event OnInit()
	Log("OnInit")
	ScanForDevices();
	RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
	RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
    RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
    RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
    RegisterForModEvent("DeviceActorOrgasm", "OnDeviceActorOrgasm")
    RegisterForModEvent("DeviceEdgedActor", "OnDeviceEdgedActor")
	Game.GetPlayer().AddSpell(VibrateSpell);
	RegisterForUpdate(3) ; Very short intervall for testing
	InSexlabScene = False
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

	If InSexlabScene
		Float speed = Utility.RandomFloat(0.01, 1.0)
		Log("In SL Scene. Adjusting speed: " + speed)
		TK_Telekinesis.TK_VibrateAll(speed)
	EndIf
EndEvent

Event OnSexlabAnimationStart(int _, bool HasPlayer)
	If !HasPlayer
		 Log("Animation on Non-Player")
		 return
	EndIf
	Log("SL Animation Start")
	InSexlabScene = True
	TK_Telekinesis.TK_VibrateAll(Utility.RandomFloat(0.01, 1.0))
EndEvent

Event OnSexlabAnimationEnd(int _, bool HasPlayer)
	If !HasPlayer
		 Log("Animation on Non-Player")
		 return
	EndIf
	Log("SL Animation End")
	InSexlabScene = False
	TK_Telekinesis.TK_VibrateAll(0)
EndEvent

Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
    Log("OnDeviceActorOrgasm")
	TK_Telekinesis.TK_VibrateAllFor( Utility.RandomFloat(0.1, 1.0), Utility.RandomFloat(5.0, 20.0) )
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
    Log("OnDeviceEdgedActor")
	TK_Telekinesis.TK_VibrateAllFor( Utility.RandomFloat(0.01, 0.2), Utility.RandomFloat(3.0, 8.0) )
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("VibrateStart " + eventName + "|" + argString + "|" + sender)
	TK_Telekinesis.TK_VibrateAll(1.0)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("VibrateStop")
	TK_Telekinesis.TK_VibrateAll(0.0)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction