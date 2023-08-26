ScriptName Tele_Integration extends Quest

Tele_Devices Property TeleDevices Auto

Event OnInit()
    RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    UpdateSexScene()
EndEvent

Bool Property Devious_VibrateEffect
    Function Set(Bool enable)
        If enable
            RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
            RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
        Else
            UnregisterForModEvent("DeviceVibrateEffectStart")
            UnregisterForModEvent("DeviceVibrateEffectStop")
        EndIf
    EndFunction
EndProperty

Event OnVibrateEffectStart(string eventName, string actorName, float vibrationStrength, form sender)
    If Game.GetPlayer().GetLeveledActorBase().GetName() != actorName
        
        return ; Not the player
    EndIf

    ; vibrationStrength = BaseStrength * VibratorMultiplicator
    ; BaseStrength:
    ;   1=very weak, 2=weak, 3=Standard, 4=Strong, 5=Very Strong)
    ; VibratorMultiplicator:
    ;   Vaginal Plug      += 0.7
    ;   Anal Plug         += 0.3
    ;   Nipple Piercings  += 0.25
    ;   Vaginal Piercings += 0.5 
    ;   Blindfold         *= 1.15
    
    ; For now as a heuristic a range of 0-5 is assumed
    ; This would be a multiplicator of 1.0 which is basically 2 devices
    ; Everything above will be rounded down to 100% by vibrate
    int speed = Math.Floor(100 * (vibrationStrength / 5))
	Tele_Api.Vibrate(speed, 30)
	TeleDevices.LogDebug("DD OnVibrateEffectStart vibrationStrength: " + vibrationStrength)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
    TeleDevices.LogDebug("DD OnVibrateEffectStop")
EndEvent

Bool Property Sexlab_Animation
    Function set(Bool enable)
        If enable
            RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
            RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
        Else
            UnregisterForModEvent("HookAnimationStart")
            UnregisterForModEvent("HookAnimationEnd")
        EndIf
    EndFunction
EndProperty

Event OnSexlabAnimationStart(int _, bool hasPlayer)
	If !hasPlayer
		 TeleDevices.LogDebug("Animation on Non-Player")
		 return
	EndIf
	StartSexScene()
EndEvent

Event OnSexlabAnimationEnd(int _, bool hasPlayer)
	If !hasPlayer
        TeleDevices.LogDebug("Animation on Non-Player")
		 return
	EndIf
	StopSexScene()
EndEvent

Bool Property Sexlab_ActorOrgasm
    Function set(Bool enable)
        If enable
            RegisterForModEvent("DeviceActorOrgasm", "OnDeviceActorOrgasm")
        Else
            UnregisterForModEvent("DeviceActorOrgasm")
        EndIf
    EndFunction
EndProperty

Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0) )
    TeleDevices.LogDebug("DD OnDeviceActorOrgasm")
EndEvent

Bool Property Sexlab_ActorEdge
    Function set(Bool enable)
        If enable
            RegisterForModEvent("DeviceEdgedActor", "OnDeviceEdgedActor")
        Else
            UnregisterForModEvent("DeviceEdgedActor")
        EndIf
    EndFunction
EndProperty

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0) )
    TeleDevices.LogDebug("DD OnDeviceEdgedActor")
EndEvent

Bool Property Toys_VibrateEffect
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysPulsate", "OnToysPulsate") ; Pulsate Effect has started. Duration is random lasting from approx. 12 to 35 seconds
        Else
            UnregisterForModEvent("ToysPulsate")
        EndIf
    EndFunction
EndProperty

Bool Property Toys_Animation
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysStartLove", "OnToysSceneStart") ; Sex scene starts
            RegisterForModEvent("ToysLoveSceneEnd", "OnToysSceneEnd") ; Sex scene ends
        Else
            UnregisterForModEvent("ToysStartLove")
            UnregisterForModEvent("ToysLoveSceneEnd")
        EndIf
    EndFunction
EndProperty

Bool Property Toys_OtherEvents
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysFondled", "OnToysFondleStart") ; Fondle started - successfully increased rousing
            RegisterForModEvent("ToysFondle", "OnToysFondleEnd") ; Fondle animation has ended (no player controls locking). Anim duration is 10 to 18 seconds.
            RegisterForModEvent("ToysSquirt", "OnToysSquirt") ; SquirtingEffect has started. There can be numerous in a single scene. Is not sent if turned off in MCM. Duration is 12 seconds
            RegisterForModEvent("ToysClimax", "OnToysClimax") ; Player has climaxed
            RegisterForModEvent("ToysCaressed", "OnToysCaressed") ; Caressing successfully increased rousing
            RegisterForModEvent("ToysClimaxSimultaneous", "OnToysClimaxSimultaneous") ; Simultaneous Orgasm. Both player & NPC have climaxed. This can happen multiple times. Sent in addition to other climax events. This event always first
            ;RegisterForModEvent("ToysVaginalPenetration", "OnToysVaginalPenetration") ; player vaginal penetration during a scene. No worn toy with BlockVaginal keyword. Solo does not count
            ;RegisterForModEvent("ToysAnalPenetration", "OnToysAnalPenetration") ; player anal penetration during a scene. No worn toy with BlockAnal keyword. Solo does not count
            ;RegisterForModEvent("ToysOralPenetration", "OnToysOralPenetration") ; player oral penetration during a scene. No worn toy with BlockOral keyword. Solo does not count 
        Else
            UnregisterForModEvent("ToysFondled")
            UnregisterForModEvent("ToysFondle")
            UnregisterForModEvent("ToysSquirt")
            UnregisterForModEvent("ToysClimax")
            UnregisterForModEvent("ToysCaressed")
            UnregisterForModEvent("ToysClimaxSimultaneous")
            ;UnregisterForModEvent("ToysVaginalPenetration")
            ;UnregisterForModEvent("ToysAnalPenetration")
            ;UnregisterForModEvent("ToysOralPenetration")
        EndIf
    EndFunction
EndProperty

Bool Property Toys_Denial
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysDenied", "OnToysDenied") ; An individuall squirt has been denied
        Else
            UnregisterForModEvent("ToysDenied")
        EndIf
    EndFunction
EndProperty

Event OnToysPulsate(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate( Utility.RandomInt(1, 100), 5 )
	TeleDevices.LogDebug("ToysPulsate")
EndEvent

Event OnToysFondleStart(string eventName, string argString, float argNum, form sender) 
	Tele_Api.Vibrate(10, 30)
	TeleDevices.LogDebug("ToysFondleStart")
EndEvent

Event OnToysFondleEnd(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
	TeleDevices.LogDebug("ToysFondleEnd")
EndEvent

Event OnToysSquirt(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(100, 12.0)
	TeleDevices.LogDebug("ToysSquirt")
EndEvent

Event OnToysSceneStart(string eventName, string argString, float argNum, form sender)
	StartSexScene()
	TeleDevices.LogDebug("ToysSceneStart")
EndEvent

Event OnToysSceneEnd(string eventName, string argString, float argNum, form sender)
	StopSexScene()
	TeleDevices.LogDebug("OnToysSceneEnd")
EndEvent

Event OnToysClimax(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(80, 5)
	TeleDevices.LogDebug("OnToysClimax")
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(100, 8)
	TeleDevices.LogDebug("OnToysClimaxSimultaneous")
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
	TeleDevices.LogDebug("OnToysDenied")
EndEvent

; Event OnToysVaginalPenetration(string eventName, string argString, float argNum, form sender)
; 	TeleDevices.LogDebug("OnToysVaginalPenetration")
; EndEvent

; Event OnToysAnalPenetration(string eventName, string argString, float argNum, form sender)
; 	TeleDevices.LogDebug("OnToysAnalPenetration")
; EndEvent

; Event OnToysOralPenetration(string eventName, string argString, float argNum, form sender)
; 	TeleDevices.LogDebug("OnToysOralPenetration")
; EndEvent

; Event OnToysCaressed(string eventName, string argString, float argNum, form sender)
; 	TeleDevices.LogDebug("OnToysCaressed")
; EndEvent

Bool InSexScene = False

Function UpdateSexScene()
    If InSexScene
		Int speed = Utility.RandomInt(0, 100)
		Tele_Api.Vibrate(speed, 10)
	EndIf
EndFunction

Function InitSexScene()
	InSexScene = False
EndFunction

Function StartSexScene()
	InSexScene = True
	Tele_Api.Vibrate(Utility.RandomInt(1, 100), 120)
EndFunction

Function StopSexScene()
	InSexScene = False
	Tele_Api.Vibrate(0, 0.1)
EndFunction

Int Property Chainbeasts_Min = 10 Auto
Int Property Chainbeasts_Max = 100 Auto

Bool Property Chainbeasts_Vibrate
    Function set(Bool enable)
        If enable
            TeleDevices.LogDebug("Enabled Chainbeasts Vibrate")
            RegisterForModEvent("SCB_VibeEvent", "OnSCB_VibeEvent")
        Else
            TeleDevices.LogDebug("Disabled Chainbeasts Vibrate")
            UnregisterForModEvent("SCB_VibeEvent")
        EndIf
    EndFunction
EndProperty

Event OnSCB_VibeEvent(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate(Utility.RandomInt(Chainbeasts_Min, Chainbeasts_Max), 3)
	TeleDevices.LogDebug("OnSCB_VibeEvent")
EndEvent
