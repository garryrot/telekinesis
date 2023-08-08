Scriptname TK_Test_DDVibrateEventObserver extends Quest  

Spell property VibrateSpell auto
Bool InSexScene = False
Bool SexSceneControl = False
Int SexSceneArousal = 1

Event OnInit()
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

Event OnUpdate()
	If InSexScene
		If SexSceneControl 
			Log("Controlled Scene. Arousal: " + SexSceneArousal + "/ 100")
			Int speed = SexSceneArousal
			Tele.Vibrate(speed, 60)
		Else
			Int speed = Utility.RandomInt(1, 100)
			Tele.Vibrate(speed, 60)
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
	Tele.Vibrate(Utility.RandomInt(1, 100), 120)
EndFunction

Function StopSexScene()
	Log("StopSexScene")
	InSexScene = False
	SexSceneArousal = 1
	Tele.Vibrate(0, 0.1)
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
	Tele.Vibrate( Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0) )
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
    Log("OnDeviceEdgedActor")
	Tele.Vibrate( Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0) )
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Log("VibrateStart " + eventName + "|" + argString + "|" + sender)
	Tele.Vibrate(100, 30)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Log("VibrateStop")
	Tele.Vibrate(0, 0.1)
EndEvent

; Toys & Love
Event OnToysPulsate(string eventName, string argString, float argNum, form sender)
	Log("ToysPulsate")
	Tele.Vibrate( Utility.RandomInt(1, 100), 5 )
EndEvent

Event OnToysFondleStart(string eventName, string argString, float argNum, form sender) 
	Log("ToysFondleStart")
	Tele.Vibrate( 10, 30 )
EndEvent

Event OnToysFondleEnd(string eventName, string argString, float argNum, form sender)
	Log("ToysFondleEnd")
	Tele.Vibrate( 0, 0.1 )
EndEvent

Event OnToysSquirt(string eventName, string argString, float argNum, form sender)
	Log("ToysSquirt")
	Tele.Vibrate( 100, 12.0 )
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
	Tele.Vibrate(80, 5)
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	Log("OnToysClimaxSimultaneous")
	Tele.Vibrate(100, 8)
EndEvent

Event OnToysVaginalPenetration(string eventName, string argString, float argNum, form sender)
	Log("OnToysVaginalPenetration")
EndEvent

Event OnToysAnalPenetration(string eventName, string argString, float argNum, form sender)
	Log("OnToysAnalPenetration")
EndEvent

Event OnToysOralPenetration(string eventName, string argString, float argNum, form sender)
	Log("OnToysOralPenetration")
EndEvent

Event OnToysCaressed(string eventName, string argString, float argNum, form sender)
	; This doesn't work as intended
	SexSceneArousal += 3
	If SexSceneArousal > 100
		SexSceneArousal = 100
	EndIf
	Log("OnToysCaressed " + SexSceneArousal)
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	; This doesn't work as intended
	Tele.Vibrate(0, 0.1)
	Log("OnToysDenied " + SexSceneArousal)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction