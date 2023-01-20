Scriptname TK_Test_DDVibrateEventObserver extends Quest  

Spell property VibrateSpell auto
Bool InSexScene = False
Bool SexSceneControl = False
Int SexSceneArousal = 1

Event OnInit()
	Log("OnInit")
	ScanForDevices();
	; Devious Devices
	RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
	RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
    RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
    RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")

	; Sexlab
    RegisterForModEvent("DeviceActorOrgasm", "OnDeviceActorOrgasm")
    RegisterForModEvent("DeviceEdgedActor", "OnDeviceEdgedActor")

	; Toys
	RegisterForModEvent("ToysPulsate", "OnToysPulsate") ; Pulsate Effect has started. Duration is random lasting from approx. 12 to 35 seconds
	RegisterForModEvent("ToysFondled", "OnToysFondleStart") ; Fondle started - successfully increased rousing
	RegisterForModEvent("ToysFondle", "OnToysFondleEnd") ; Fondle animation has ended (no player controls locking). Anim duration is 10 to 18 seconds.
	RegisterForModEvent("ToysSquirt", "OnToysSquirt") ; SquirtingEffect has started. There can be numerous in a single scene. Is not sent if turned off in MCM. Duration is 12 seconds
	RegisterForModEvent("ToysStartLove", "OnToysSceneStart") ; Sex scene starts
	RegisterForModEvent("ToysLoveSceneEnd", "OnToysSceneEnd") ; Sex scene ends
	RegisterForModEvent("ToysClimax", "OnToysClimax") ; Player has climaxed
	RegisterForModEvent("ToysClimaxSimultaneous", "OnToysClimaxSimultaneous") ; Simultaneous Orgasm. Both player & NPC have climaxed. This can happen multiple times. Sent in addition to other climax events. This event always first
	RegisterForModEvent("ToysVaginalPenetration", "OnToysVaginalPenetration") ; player vaginal penetration during a scene. No worn toy with BlockVaginal keyword. Solo does not count
	RegisterForModEvent("ToysAnalPenetration", "OnToysAnalPenetration") ; player anal penetration during a scene. No worn toy with BlockAnal keyword. Solo does not count
	RegisterForModEvent("ToysOralPenetration", "OnToysOralPenetration") ; player oral penetration during a scene. No worn toy with BlockOral keyword. Solo does not count
	RegisterForModEvent("ToysCaressed", "OnToysCaressed") ; Caressing successfully increased rousing
	RegisterForModEvent("ToysDenied", "OnToysDenied") ; An individuall squirt has been denied

	Game.GetPlayer().AddSpell(VibrateSpell);
	RegisterForUpdate(3) ; Very short intervall for testing
	InSexScene = False
	SexSceneControl = False
	SexSceneArousal = 1
EndEvent

Function ScanForDevices()
	Log("Scanning for devices...")
	Tele.ScanForDevices();
EndFunction

Event OnUpdate()
	String[] evts = Tele.PollEvents();
	Int i = 0;
	While (i < evts.Length) 
		Log(evts[0])
		i += 1
	EndWhile
	If InSexScene
		If SexSceneControl 
			Log("Controlled Scene. Arousal: " + SexSceneArousal + "/ 100")
			Float speed = SexSceneArousal / 100.0
			Tele.VibrateAll(speed)
		Else
			Float speed = Utility.RandomFloat(0.01, 1.0)
			Tele.VibrateAll(speed)
			Log("Unctontrolled Scene. Random: " + speed)
		EndIf
	EndIf
EndEvent

; Sexlab
Function StartSexScene(Bool controlled)
	Log("StartSexScene")
	SexSceneControl = controlled
	InSexScene = True
	SexSceneArousal = 1
	Tele.VibrateAll(Utility.RandomFloat(0.01, 3.0))
EndFunction

Function StopSexScene()
	Log("StopSexScene")
	InSexScene = False
	SexSceneArousal = 1
	Tele.VibrateAll(0)
EndFunction

Event OnSexlabAnimationStart(int _, bool HasPlayer)
	If !HasPlayer
		 Log("Animation on Non-Player")
		 return
	EndIf
	StartSexScene(False)
EndEvent

Event OnSexlabAnimationEnd(int _, bool HasPlayer)
	If !HasPlayer
		 Log("Animation on Non-Player")
		 return
	EndIf
	StopSexScene()
EndEvent

; Devious Devices
Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
    Log("OnDeviceActorOrgasm")
	Tele.VibrateAllFor( Utility.RandomFloat(0.1, 1.0), Utility.RandomFloat(5.0, 20.0) )
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
    Log("OnDeviceEdgedActor")
	Tele.VibrateAllFor( Utility.RandomFloat(0.01, 0.2), Utility.RandomFloat(3.0, 8.0) )
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("VibrateStart " + eventName + "|" + argString + "|" + sender)
	Tele.VibrateAll(1.0)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("VibrateStop")
	Tele.VibrateAll(0.0)
EndEvent

; Toys & Love
Event OnToysPulsate(string eventName, string argString, float argNum, form sender)
	Log("ToysPulsate")
	Tele.VibrateAllFor( Utility.RandomFloat(0.01, 1.0), 5 )
EndEvent

Event OnToysFondleStart(string eventName, string argString, float argNum, form sender) 
	Log("ToysFondleStart")
	Tele.VibrateAll( 0.1 )
EndEvent

Event OnToysFondleEnd(string eventName, string argString, float argNum, form sender)
	Log("ToysFondleEnd")
	Tele.VibrateAll( 0.0 )
EndEvent

Event OnToysSquirt(string eventName, string argString, float argNum, form sender)
	Log("ToysSquirt")
	Tele.VibrateAllFor( 1.0, 12.0 )
EndEvent

Event OnToysSceneStart(string eventName, string argString, float argNum, form sender)
	Log("ToysSceneStart")
	StartSexScene(True)
EndEvent

Event OnToysSceneEnd(string eventName, string argString, float argNum, form sender)
	Log("OnToysSceneEnd")
	StopSexScene()
EndEvent

Event OnToysClimax(string eventName, string argString, float argNum, form sender)
	Log("OnToysClimax")
	Tele.VibrateAllFor(0.8, 5)
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	Log("OnToysClimaxSimultaneous")
	Tele.VibrateAllFor(1.0, 8)
EndEvent

Event OnToysVaginalPenetration(string eventName, string argString, float argNum, form sender)
	SexSceneArousal += 10
	If SexSceneArousal > 100
		SexSceneArousal = 100
	EndIf
	Log("OnToysVaginalPenetration " + SexSceneArousal)
EndEvent

Event OnToysAnalPenetration(string eventName, string argString, float argNum, form sender)
	SexSceneArousal += 10
	If SexSceneArousal > 100
		SexSceneArousal = 100
	EndIf
	Log("OnToysAnalPenetration " + SexSceneArousal)
EndEvent

Event OnToysOralPenetration(string eventName, string argString, float argNum, form sender)
	Log("OnToysOralPenetration " + SexSceneArousal)
EndEvent

Event OnToysCaressed(string eventName, string argString, float argNum, form sender)
	SexSceneArousal += 3
	If SexSceneArousal > 100
		SexSceneArousal = 100
	EndIf
	Log("OnToysCaressed " + SexSceneArousal)
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	Tele.VibrateAll(0.0) ; Pause until next vibration event
	Log("OnToysDenied " + SexSceneArousal)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction