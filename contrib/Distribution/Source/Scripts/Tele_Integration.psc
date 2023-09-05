ScriptName Tele_Integration extends Quest
{
    Integrates devices with the game and mods
    ~ Use this API to enable/disable integration features ~
}

Tele_Devices Property TeleDevices Auto

ZadLibs Property ZadLib Auto
SexLabFramework Property SexLab Auto

String[] _SceneTags

Bool _InSexScene = false
Bool _InToysSexScene = false
Bool _Devious_VibrateEffect = false
Bool _Sexlab_Animation = false
Bool _Sexlab_ActorOrgasm = false
Bool _Sexlab_ActorEdge = false
Bool _Toys_VibrateEffect = false
Bool _Toys_Animation = false
Bool _Toys_OtherEvents = false
Bool _Toys_Denial = false
Bool _Chainbeasts_Vibrate = false

Int _EmergencyHotkey = 211
Int Property EmergencyHotkey_Default = 211 AutoReadOnly ; del
Int Property EmergencyHotkey
    Function Set(Int keyCode)
        UnregisterForKey(_EmergencyHotkey)
        _EmergencyHotkey = keyCode
        RegisterForKey(_EmergencyHotkey)
    EndFunction
    Int Function Get()
        return _EmergencyHotkey
    EndFunction
EndProperty

String Property Devious_VibrateEventAnal = "Anal" Auto
String Property Devious_VibrateEventVaginal = "Vaginal" Auto
String Property Devious_VibrateEventNipple = "Nipple" Auto
Int Property Devious_VibrateEffectDeviceSelector = 0 Auto
Int Property Devious_VibrateEffectDeviceSelector_Default = 0 AutoReadOnly
Bool Property Devious_VibrateEffect_Default = true AutoReadOnly
Bool Property Devious_VibrateEffect
    Function Set(Bool enable)
        _Devious_VibrateEffect = enable
        If enable 
            RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
            RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
        Else
            UnregisterForModEvent("DeviceVibrateEffectStart")
            UnregisterForModEvent("DeviceVibrateEffectStop")
        EndIf
    EndFunction
    Bool Function Get()
        return _Devious_VibrateEffect
    EndFunction
EndProperty

Int Property Sexlab_AnimationDeviceSelector = 0 Auto
Int Property Sexlab_AnimationDeviceSelector_Default = 0 AutoReadOnly
Bool Property Sexlab_Animation_Default = false AutoReadOnly
Bool Property Sexlab_Animation
    Function Set(Bool enable)
        _Sexlab_Animation = enable
        If enable
            RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
            RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
        Else
            UnregisterForModEvent("HookAnimationStart")
            UnregisterForModEvent("HookAnimationEnd")
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_Animation
    EndFunction
EndProperty

Bool Property Sexlab_ActorOrgasm_Default = false AutoReadOnly
Bool Property Sexlab_ActorOrgasm
    Function Set(Bool enable)
        _Sexlab_ActorOrgasm = enable
        If enable
            RegisterForModEvent("DeviceActorOrgasm", "OnDeviceActorOrgasm")
        Else
            UnregisterForModEvent("DeviceActorOrgasm")
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_ActorOrgasm
    EndFunction
EndProperty

Bool Property Sexlab_ActorEdge_Default = false AutoReadOnly
Bool Property Sexlab_ActorEdge
    Function Set(Bool enable)
        _Sexlab_ActorEdge = enable
        If enable
            RegisterForModEvent("DeviceEdgedActor", "OnDeviceEdgedActor")
        Else
            UnregisterForModEvent("DeviceEdgedActor")
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_ActorEdge
    EndFunction
EndProperty

Bool Property Toys_VibrateEffect_Default = true AutoReadOnly
Bool Property Toys_VibrateEffect
    Function Set(Bool enable)
        _Toys_VibrateEffect = enable
        If enable
            RegisterForModEvent("ToysPulsate", "OnToysPulsate") ; Duration is random lasting from approx. 12 to 35 seconds
        Else
            UnregisterForModEvent("ToysPulsate")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_VibrateEffect
    EndFunction
EndProperty

Bool Property Toys_Animation_Default = false AutoReadOnly
Bool Property Toys_Animation
    Function Set(Bool enable)
        _Toys_Animation = enable
        If enable
            RegisterForModEvent("ToysStartLove", "OnToysSceneStart") ; Sex scene starts
            RegisterForModEvent("ToysLoveSceneEnd", "OnToysSceneEnd") ; Sex scene ends
        Else
            UnregisterForModEvent("ToysStartLove")
            UnregisterForModEvent("ToysLoveSceneEnd")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Animation
    EndFunction
EndProperty
 
Bool Property Toys_OtherEvents_Default = false AutoReadOnly
Bool Property Toys_OtherEvents
    Function Set(Bool enable)
        _Toys_OtherEvents = enable
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
    Bool Function Get()
        return _Toys_OtherEvents
    EndFunction
EndProperty

Bool Property Toys_Denial_Default = false AutoReadOnly
Bool Property Toys_Denial
    Function Set(Bool enable)
        _Toys_Denial = enable
        If enable
            RegisterForModEvent("ToysDenied", "OnToysDenied") ; An individuall squirt has been denied
        Else
            UnregisterForModEvent("ToysDenied")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Denial
    EndFunction
EndProperty

Int Property Chainbeasts_Min = 80 Auto
Int Property Chainbeasts_Min_Default = 80 AutoReadOnly
Int Property Chainbeasts_Max = 100 Auto
Int Property Chainbeasts_Max_Default = 100 AutoReadOnly
Bool Property Chainbeasts_Vibrate_Default = true AutoReadOnly
Bool Property Chainbeasts_Vibrate
    Function Set(Bool enable)
        _Chainbeasts_Vibrate = enable
        If enable
            TeleDevices.LogDebug("Enabled Chainbeasts Vibrate")
            RegisterForModEvent("SCB_VibeEvent", "OnSCB_VibeEvent")
        Else
            TeleDevices.LogDebug("Disabled Chainbeasts Vibrate")
            UnregisterForModEvent("SCB_VibeEvent")
        EndIf
    EndFunction
    Bool Function Get()
        return _Chainbeasts_Vibrate
    EndFunction
EndProperty

Event OnInit()
    RegisterForUpdate(5)
    InitDefaultOnEventHandlers()
EndEvent

Function InitDefaultOnEventHandlers()
    EmergencyHotkey = EmergencyHotkey_Default
    Devious_VibrateEffect = true
    Toys_VibrateEffect = true
    Chainbeasts_Vibrate = true
EndFunction

Function ResetIntegrationSettings()
    TeleDevices.Notify("Resetting integration settings")
    EmergencyHotkey = EmergencyHotkey_Default
    Devious_VibrateEffect = Devious_VibrateEffect_Default
    Devious_VibrateEffectDeviceSelector = Devious_VibrateEffectDeviceSelector_Default
    Sexlab_Animation = Sexlab_Animation_Default
    Sexlab_AnimationDeviceSelector = Sexlab_AnimationDeviceSelector_Default
    Sexlab_ActorOrgasm = Sexlab_ActorOrgasm_Default
    Sexlab_ActorEdge = Sexlab_ActorEdge_Default
    Toys_VibrateEffect = Toys_VibrateEffect_Default
    Toys_Animation = Toys_Animation_Default
    Toys_OtherEvents = Toys_OtherEvents_Default
    Toys_Denial = Toys_Denial_Default
    Chainbeasts_Vibrate = Chainbeasts_Vibrate_Default
    Chainbeasts_Min = Chainbeasts_Min_Default
    Chainbeasts_Max = Chainbeasts_Max_Default
EndFunction

; Key Events

Event OnKeyUp(Int keyCode, Float HoldTime)
    If keyCode == _EmergencyHotkey
        TeleDevices.VibrateStopAll()
        Tele_Api.StopAll()
        TeleDevices.LogError("Emergency stop")
    Else
        TeleDevices.LogDebug("Unregistered keypress code: " + KeyCode)
    EndIf
EndEvent

; Sex animation handling

Event OnUpdate()
    UpdateSexScene()
EndEvent

Function UpdateSexScene()
    Int speed = Utility.RandomInt(0, 100)
    If _InSexScene
        If Sexlab_AnimationDeviceSelector == 1
            TeleDevices.VibrateEvents(speed, 5, _SceneTags)
        Else
            TeleDevices.Vibrate(speed, 5)
        EndIf
	EndIf
    If _InToysSexScene
		TeleDevices.Vibrate(speed, 5)
    EndIF
EndFunction

Function StartSexLabScene(String[] tags)
	_InSexScene = True
    _SceneTags = tags
	UpdateSexScene()
EndFunction

Function StopSexLabScene()
	_InSexScene = False
    ; If Sexlab_AnimationDeviceSelector == 1
    ;     TeleDevices.VibrateEvents(0, 0.1, _SceneTags)
    ; Else
    ;     TeleDevices.Vibrate(0, 0.1)
    ; EndIF
EndFunction

Function StartToysScene()
	_InToysSexScene = True
	UpdateSexScene()
EndFunction

Function StopToysScene()
	_InToysSexScene = False
EndFunction

; Devious Devices Events

Event OnVibrateEffectStart(String eventName, String actorName, Float vibrationStrength, Form sender)
    Actor player = Game.GetPlayer()
    If player.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    If ZadLib == None
        return ; Should not happen
    EndIf

    String[] events = GetDDTags(player)
    Int strength = Math.Floor((vibrationStrength / _NumVibratorsMult) * 20)
    If Devious_VibrateEffectDeviceSelector == 1
        TeleDevices.VibrateEvents(strength, -1, events)
    Else
        TeleDevices.Vibrate(strength, -1)
    EndIf
	TeleDevices.LogDebug("OnVibrateEffectStart strength: " + strength)
EndEvent

Event OnVibrateEffectStop(string eventName, string actorName, float argNum, form sender)
    Actor player = Game.GetPlayer()
    If player.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    If ZadLib == None
        return ; Should not happen
    EndIf

    If Devious_VibrateEffectDeviceSelector == 1
        String[] events = GetDDTags(player)
        TeleDevices.VibrateStop(events)
    Else
        TeleDevices.VibrateStopAll()
    EndIf
EndEvent

Float _NumVibratorsMult
String[] Function GetDDTags(Actor player)
    ; Reverse DD multi device calculation to get the actual strength
    String[] events = new String[3]
    _NumVibratorsMult = 0
    If player.WornHasKeyword(ZadLib.zad_DeviousPlugVaginal)
        _NumVibratorsMult += 0.7
        events[0] = Devious_VibrateEventVaginal
    EndIf
    If player.WornHasKeyword(ZadLib.zad_DeviousPlugAnal)
        _NumVibratorsMult += 0.3
        events[1] = Devious_VibrateEventAnal
    EndIf
    If player.WornHasKeyword(ZadLib.zad_DeviousPiercingsNipple)
        _NumVibratorsMult += 0.25
        events[2] = Devious_VibrateEventNipple
    EndIf
    If player.WornHasKeyword(ZadLib.zad_DeviousPiercingsVaginal)
        _NumVibratorsMult += 0.5
        events[0] = Devious_VibrateEventVaginal
    EndIf
    If player.WornHasKeyword(ZadLib.zad_DeviousBlindfold) 
        _NumVibratorsMult /= 1.15
    EndIf
    return events
EndFunction


; Sexlab Events

Event OnSexlabAnimationStart(int threadID, bool hasPlayer)
	If !hasPlayer
		TeleDevices.LogDebug("Animation on Non-Player")
		return
	EndIf
    sslThreadController Controller = Sexlab.GetController(threadID)
    sslBaseAnimation animation = Controller.Animation
	StartSexLabScene(animation.GetTags()) 
EndEvent

Event OnSexlabAnimationEnd(int _, bool hasPlayer)
	If !hasPlayer
        TeleDevices.LogDebug("Animation on Non-Player")
		 return
	EndIf
	StopSexLabScene()
EndEvent

Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0) )
    TeleDevices.LogDebug("OnDeviceActorOrgasm")
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0) )
    TeleDevices.LogDebug("OnDeviceEdgedActor")
EndEvent

; Toys & Love Events

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
	StartToysScene()
	TeleDevices.LogDebug("ToysSceneStart")
EndEvent

Event OnToysSceneEnd(string eventName, string argString, float argNum, form sender)
	StopToysScene()
	TeleDevices.LogDebug("OnToysSceneEnd")
EndEvent

Event OnToysClimax(string eventName, string argString, float argNum, form sender)
	TeleDevices.Vibrate(80, 5)
	TeleDevices.LogDebug("OnToysClimax")
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	TeleDevices.Vibrate(100, 8)
	TeleDevices.LogDebug("OnToysClimaxSimultaneous")
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	TeleDevices.Vibrate(0, 5)
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

; Skyrim Chain Beasts Events

Event OnSCB_VibeEvent(string eventName, string strArg, float numArg, Form sender)
	TeleDevices.Vibrate(Utility.RandomInt(Chainbeasts_Min, Chainbeasts_Max), 3)
	TeleDevices.LogDebug("OnSCB_VibeEvent")
EndEvent
