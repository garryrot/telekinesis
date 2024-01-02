ScriptName Tele_Integration extends Quest
{
    Integrates devices with the game and mods
    ~ Use this API to enable/disable integration features ~
}

Tele_Devices Property TeleDevices Auto

Actor Property PlayerRef Auto

Quest Property ZadLib Auto        ; ZadLibs
Quest Property SexLab Auto        ; SexLabFramework
Quest Property Toys Auto          ; ToysFramework
Quest Property SexLabAroused Auto ; SlaFrameworkScr
OSexIntegrationMain Property OStim Auto

Bool _InSexlabScene = false
Bool _InToysScene = false
Int _OstimSceneVibrationHandle = -1
Int _OstimMaxSpeed = 4

Bool _DeviousDevices_Vibrate = false
Bool _Sexlab_Animation = false
Bool _Sexlab_ActorOrgasm = false
Bool _Sexlab_ActorEdge = false
Bool _Ostim_Animation = false
Bool _Toys_Animation = false
Bool _Toys_Caressed = false
Bool _Toys_Climax = false
Bool _Toys_Denial = false
Bool _Toys_Fondle = false
Bool _Toys_Squirt = false
Bool _Toys_Vibrate = false
Bool _Toys_Vaginal_Penetration = false
Bool _Toys_Anal_Penetration = false
Bool _Toys_Oral_Penetration = false
Bool _Chainbeasts_Vibrate = false
Int _EmergencyHotkey = 211

Int _DeviousDevicesVibrateHandle = -1
Int _SexlabSceneVibrationHandle = -1
Int _ToysSceneVibrationHandle = -1

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

Int Property DeviousDevices_Vibrate_DeviceSelector = 0 Auto
Int Property DeviousDevices_Vibrate_DeviceSelector_Default = 0 AutoReadOnly
String Property DeviousDevices_Vibrate_Event_Anal = "Anal" Auto
String Property DeviousDevices_Vibrate_Event_Anal_Default = "Anal" Auto
String Property DeviousDevices_Vibrate_Event_Vaginal = "Vaginal" Auto
String Property DeviousDevices_Vibrate_Event_Vaginal_Default = "Vaginal" Auto
String Property DeviousDevices_Vibrate_Event_Nipple = "Nipple" Auto
String Property DeviousDevices_Vibrate_Event_Nipple_Default = "Nipple" Auto
Int Property DeviousDevices_Vibrate_Pattern = 0 Auto
Int Property DeviousDevices_Vibrate_Pattern_Default = 0 AutoReadOnly
Bool Property DeviousDevices_Vibrate_Default = true AutoReadOnly
String Property DeviousDevices_Vibrate_Funscript = "30_Sawtooth" Auto
String Property DeviousDevices_Vibrate_Funscript_Default = "30_Sawtooth" Auto
Bool Property DeviousDevices_Vibrate
    Function Set(Bool enable)
        _DeviousDevices_Vibrate = enable
        If enable
            RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
            RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
        Else
            UnregisterForModEvent("DeviceVibrateEffectStart")
            UnregisterForModEvent("DeviceVibrateEffectStop")
        EndIf
    EndFunction
    Bool Function Get()
        return _DeviousDevices_Vibrate
    EndFunction
EndProperty

String Property Sexlab_Animation_Funscript = "" Auto
String Property Sexlab_Animation_Funscript_Default = "" Auto
Int Property Sexlab_Animation_DeviceSelector = 0 Auto
Int Property Sexlab_Animation_DeviceSelector_Default = 0 AutoReadOnly
Bool Property Sexlab_Animation_Rousing = False Auto
Bool Property Sexlab_Animation_Rousing_Default = False AutoReadOnly
Int Property Sexlab_Animation_Pattern = 0 Auto
Int Property Sexlab_Animation_Pattern_Default = 0 AutoReadOnly
Int Property Sexlab_Animation_Linear_Strength = 80 Auto
Int Property Sexlab_Animation_Linear_Strength_Default = 80 AutoReadOnly
Bool Property Sexlab_Animation_Default = true AutoReadOnly
Bool Property Sexlab_Animation
    Function Set(Bool enable)
        _Sexlab_Animation = enable
        If enable
            RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
            RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
        Else
            _InSexlabScene = False
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

String Property Ostim_Animation_Funscript = "" Auto
String Property Ostim_Animation_Funscript_Default = "" Auto
Int Property Ostim_Animation_DeviceSelector = 0 Auto
Int Property Ostim_Animation_DeviceSelector_Default = 0 AutoReadOnly
Int Property Ostim_Animation_Speed_Control = 1 Auto
Int Property Ostim_Animation_Speed_Control_Default = 1 AutoReadOnly
Int Property Ostim_Animation_Pattern = 0 Auto
Int Property Ostim_Animation_Pattern_Default = 0 AutoReadOnly  
String Property Ostim_Animation_Event_Anal = "Anal" Auto
String Property Ostim_Animation_Event_Anal_Default = "Anal" Auto
String Property Ostim_Animation_Event_Vaginal = "Vaginal" Auto
String Property Ostim_Animation_Event_Vaginal_Default = "Vaginal" Auto
String Property Ostim_Animation_Event_Nipple = "Nipple" Auto
String Property Ostim_Animation_Event_Nipple_Default = "Nipple" Auto
String Property Ostim_Animation_Event_Penetration = "Penetration" Auto
String Property Ostim_Animation_Event_Penetration_Default = "Penetration" Auto
String Property Ostim_Animation_Event_Penis = "Penis" Auto
String Property Ostim_Animation_Event_Penis_Default = "Penis" Auto

Bool Property Ostim_Animation_Default = true AutoReadOnly
Bool Property Ostim_Animation
    Function Set(Bool enable)
        _Ostim_Animation = enable
        If enable
            RegisterForModEvent("OStim_Start", "OnOStimStart")
            RegisterForModEvent("OStim_SceneChanged", "OnOStimSceneChanged")
            RegisterForModEvent("OStim_End", "OnOstimEnd") 
        Else
            UnregisterForModEvent("OStim_Start")
            UnregisterForModEvent("OStim_SceneChanged")
            UnregisterForModEvent("OStim_End")
            If _OstimSceneVibrationHandle != -1
                TeleDevices.StopHandle(_OstimSceneVibrationHandle)
            EndIf
            _OstimSceneVibrationHandle = -1
        EndIf
    EndFunction
    Bool Function Get()
        return _Ostim_Animation
    EndFunction
EndProperty

Bool Property Toys_Vaginal_Penetration_Default = false AutoReadOnly
Bool Property Toys_Vaginal_Penetration
    Function Set(Bool enable)
        _Toys_Vaginal_Penetration = enable
        If enable
            RegisterForModEvent("ToysVaginalPenetration", "OnToysVaginalPenetration")
        Else
            UnregisterForModEvent("ToysVaginalPenetration")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Vaginal_Penetration
    EndFunction
EndProperty

Bool Property Toys_Anal_Penetration_Default = false AutoReadOnly
Bool Property Toys_Anal_Penetration
    Function Set(Bool enable)
        _Toys_Anal_Penetration = enable
        If enable
            RegisterForModEvent("ToysAnalPenetration", "OnToysAnalPenetration")
        Else
            UnregisterForModEvent("ToysAnalPenetration")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Anal_Penetration
    EndFunction
EndProperty

Bool Property Toys_Oral_Penetration_Default = false AutoReadOnly
Bool Property Toys_Oral_Penetration
    Function Set(Bool enable)
        _Toys_Oral_Penetration = enable
        If enable
            RegisterForModEvent("ToysOralPenetration", "OnToysOralPenetration")
        Else
            UnregisterForModEvent("ToysOralPenetration")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Oral_Penetration
    EndFunction
EndProperty

Bool Property Toys_Fondle_Default = false AutoReadOnly
Bool Property Toys_Fondle
    Function Set(Bool enable)
        _Toys_Fondle = enable
        If enable
            RegisterForModEvent("ToysFondled", "OnToysFondleStart")
            RegisterForModEvent("ToysFondle", "OnToysFondleEnd") 
        Else
            UnregisterForModEvent("ToysFondled")
            UnregisterForModEvent("ToysFondle")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Fondle
    EndFunction
EndProperty

Bool Property Toys_Squirt_Default = false AutoReadOnly
Bool Property Toys_Squirt
    Function Set(Bool enable)
        _Toys_Squirt = enable
        If enable
            RegisterForModEvent("ToysSquirt", "OnToysSquirt")
        Else
            UnregisterForModEvent("ToysSquirt")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Squirt
    EndFunction
EndProperty

Bool Property Toys_Climax_Default = false AutoReadOnly
Bool Property Toys_Climax
    Function Set(Bool enable)
        _Toys_Climax = enable
        If enable
            RegisterForModEvent("ToysClimax", "OnToysClimax")
            RegisterForModEvent("ToysClimaxSimultaneous", "OnToysClimaxSimultaneous")
        Else
            UnregisterForModEvent("ToysClimax")
            UnregisterForModEvent("ToysClimaxSimultaneous")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Climax
    EndFunction
EndProperty

; EndProperty
String Property Toys_Vibrate_Funscript = "" Auto
String Property Toys_Vibrate_Funscript_Default = "" Auto
Int Property Toys_Vibrate_DeviceSelector = 0 Auto
Int Property Toys_Vibrate_DeviceSelector_Default = 0 AutoReadOnly
String Property Toys_Vibrate_Event = "Vaginal" Auto
String Property Toys_Vibrate_Event_Default = "Vaginal" AutoReadOnly
Int Property Toys_Vibrate_Pattern = 0 Auto
Int Property Toys_Vibrate_Pattern_Default = 0 AutoReadOnly
Int Property Toys_Vibrate_Linear_Strength = 80 Auto
Int Property Toys_Vibrate_Linear_Strength_Default = 80 AutoReadOnly
Bool Property Toys_Vibrate_Default = true AutoReadOnly
Bool Property Toys_Vibrate
    Function Set(Bool enable)
        _Toys_Vibrate = enable
        If enable
            RegisterForModEvent("ToysPulsate", "OnToysPulsate")
        Else
            UnregisterForModEvent("ToysPulsate")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Vibrate
    EndFunction
EndProperty

String Property Toys_Animation_Funscript = "" Auto
String Property Toys_Animation_Funscript_Default = "" Auto
Int Property Toys_Animation_DeviceSelector = 0 Auto
Int Property Toys_Animation_DeviceSelector_Default = 0 AutoReadOnly
Bool Property Toys_Animation_Match_Tags = false Auto
Bool Property Toys_Animation_Match_Tags_Default = false AutoReadOnly
String Property Toys_Animation_Event_Vaginal = "Vaginal" Auto
String Property Toys_Animation_Event_Vaginal_Default = "Vaginal" AutoReadOnly
String Property Toys_Animation_Event_Oral = "Oral" Auto
String Property Toys_Animation_Event_Oral_Default = "Oral" AutoReadOnly
String Property Toys_Animation_Event_Anal = "Anal" Auto
String Property Toys_Animation_Event_Anal_Default = "Anal" AutoReadOnly
String Property Toys_Animation_Event_Nipple = "Nipple" Auto
String Property Toys_Animation_Event_Nipple_Default = "Nipple" AutoReadOnly
Bool Property Toys_Animation_Rousing = true Auto
Bool Property Toys_Animation_Rousing_Default = true AutoReadOnly
Int Property Toys_Animation_Pattern = 0 Auto
Int Property Toys_Animation_Pattern_Default = 0 AutoReadOnly
Int Property Toys_Animation_Linear_Strength = 80 Auto
Int Property Toys_Animation_Linear_Strength_Default = 80 AutoReadOnly
Bool Property Toys_Animation_Default = true AutoReadOnly
Bool Property Toys_Animation
    Function Set(Bool enable)
        _Toys_Animation = enable
        If enable
            RegisterForModEvent("ToysLoveSceneEnd", "OnToysSceneEnd")
            RegisterForModEvent("ToysLoveSceneInfo", "OnToysLoveSceneInfo") 
        Else
            UnregisterForModEvent("ToysLoveSceneEnd")
            UnregisterForModEvent("ToysLoveSceneInfo")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Animation
    EndFunction
EndProperty
 
Bool Property Toys_Caressed_Default = false AutoReadOnly
Bool Property Toys_Caressed
    Function Set(Bool enable)
        _Toys_Caressed = enable
        If enable
            RegisterForModEvent("ToysCaressed", "OnToysCaressed") ; Caressing successfully increased rousing
        Else
            UnregisterForModEvent("ToysCaressed")
        EndIf
    EndFunction
    Bool Function Get()
        return _Toys_Caressed
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

Int Property Chainbeasts_Vibrate_DeviceSelector = 0 Auto
Int Property Chainbeasts_Vibrate_DeviceSelector_Default = 0 AutoReadOnly
String Property Chainbeasts_Vibrate_Event = "Vaginal" Auto
String Property Chainbeasts_Vibrate_Event_Default = "Vaginal" AutoReadOnly
Int Property Chainbeasts_Vibrate_Pattern = 1 Auto
Int Property Chainbeasts_Vibrate_Pattern_Default = 1 AutoReadOnly
String Property Chainbeasts_Vibrate_Funscript = "03_Wub-Wub-Wub" Auto
String Property Chainbeasts_Vibrate_Funscript_Default = "03_Wub-Wub-Wub" Auto
Int Property Chainbeasts_Vibrate_Linear_Strength = 80 Auto
Int Property Chainbeasts_Vibrate_Linear_Strength_Default = 80 AutoReadOnly
Bool Property Chainbeasts_Vibrate_Default = true AutoReadOnly
Bool Property Chainbeasts_Vibrate
    Function Set(Bool enable)
        _Chainbeasts_Vibrate = enable
        If enable
            RegisterForModEvent("SCB_VibeEvent", "OnSCB_VibeEvent")
        Else
            UnregisterForModEvent("SCB_VibeEvent")
        EndIf
    EndFunction
    Bool Function Get()
        return _Chainbeasts_Vibrate
    EndFunction
EndProperty

Event OnInit()
    InitDefaultOnEventHandlers()
EndEvent

; Key Events

Event OnKeyUp(Int keyCode, Float HoldTime)
    If keyCode == _EmergencyHotkey
        TeleDevices.EmergencyStop()
    Else
        TeleDevices.LogDebug("Unregistered keypress code: " + KeyCode)
    EndIf
EndEvent

; Devious Devices Events

Event OnVibrateEffectStart(String eventName, String actorName, Float vibrationStrength, Form sender)
    If PlayerRef.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    If ZadLib == None
        return ; Should not happen
    EndIf

    ; Reverse DD multi device calculation to get the actual strength
    String[] events = new String[3]
    Float numVibratorsMult = 0
    If PlayerRef.WornHasKeyword((ZadLib as ZadLibs).zad_DeviousPlugVaginal)
        numVibratorsMult += 0.7
        events[0] = DeviousDevices_Vibrate_Event_Vaginal
    EndIf
    If PlayerRef.WornHasKeyword((ZadLib as ZadLibs).zad_DeviousPlugAnal)
        numVibratorsMult += 0.3
        events[1] = DeviousDevices_Vibrate_Event_Anal
    EndIf
    If PlayerRef.WornHasKeyword((ZadLib as ZadLibs).zad_DeviousPiercingsNipple)
        numVibratorsMult += 0.25
        events[2] = DeviousDevices_Vibrate_Event_Nipple
    EndIf
    If PlayerRef.WornHasKeyword((ZadLib as ZadLibs).zad_DeviousPiercingsVaginal)
        numVibratorsMult += 0.5
        events[0] = DeviousDevices_Vibrate_Event_Vaginal
    EndIf
    If PlayerRef.WornHasKeyword((ZadLib as ZadLibs).zad_DeviousBlindfold) 
        numVibratorsMult /= 1.15
    EndIf
    Int strength = Math.Floor((vibrationStrength / numVibratorsMult) * 20)
    _DeviousDevicesVibrateHandle = StartVibration(DeviousDevices_Vibrate_DeviceSelector, -1, DeviousDevices_Vibrate_Pattern, DeviousDevices_Vibrate_Funscript, strength, events)
	; TeleDevices.LogDebug("OnVibrateEffectStart strength: " + strength)
EndEvent

Event OnVibrateEffectStop(string eventName, string actorName, float argNum, form sender)
    If PlayerRef.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    If ZadLib == None
        return ; Should not happen
    EndIf
    TeleDevices.StopHandle(_DeviousDevicesVibrateHandle)
EndEvent

; Sexlab Events

Event OnSexlabAnimationStart(int threadID, bool hasPlayer)
	If !hasPlayer
		return
	EndIf
    sslThreadController controller = (Sexlab as SexLabFramework).GetController(threadID)
    sslBaseAnimation animation = controller.Animation

    _SexlabSceneVibrationHandle = StartVibration(Sexlab_Animation_DeviceSelector, -1, Sexlab_Animation_Pattern, Sexlab_Animation_Funscript, Sexlab_Animation_Linear_Strength, animation.GetTags())
    If Sexlab_Animation_Rousing
        _InSexlabScene = True
        UnregisterForUpdate()
        UpdateRousingControlledSexScene()
    EndIf
EndEvent

Event OnSexlabAnimationEnd(int _, bool hasPlayer)
	If !hasPlayer
		return
	EndIf
	_InSexlabScene = False
    TeleDevices.StopHandle(_SexlabSceneVibrationHandle)
EndEvent

Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
	TeleDevices.Vibrate(Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0))
    ; TeleDevices.LogDebug("OnDeviceActorOrgasm")
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
	TeleDevices.Vibrate(Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0))
    ; TeleDevices.LogDebug("OnDeviceEdgedActor")
EndEvent

; OStim 

Bool Function OstimPlayerHasVaginalStimulation(String sceneID, Int playerTarget, Int playerActor)
    Int activeVaginalStim = OMetadata.FindAnyActionForActorCSV(sceneID, playerActor, "femalemasturbation,tribbing")
    Int passiveVaginalStim = OMetadata.FindAnyActionForTargetCSV(sceneID, playerTarget, "cunnilingus,lickingvagina,rubbingclitoris,vaginalsex,vaginalfisting,vaginalfingering,vaginaltoying")
    return activeVaginalStim != -1 || passiveVaginalStim != -1
EndFunction

Bool Function OstimPlayerHasAnalStimulation(String sceneID, Int playerTarget, Int playerActor)
    Int passiveAction = OMetadata.FindAnyActionForTargetCSV(sceneID, playerTarget, "analfingering,analfisting,analsex,analtoying,anilingus,rimjob")
    return passiveAction != -1
EndFunction

Bool Function OstimPlayerHasNippleStimulation(String sceneID, Int playerTarget, Int playerActor)
    Int passiveAction = OMetadata.FindAnyActionForTargetCSV( sceneID, playerTarget, "gropingbreast,lickingnipple,boobjob,suckingnipple" )
    return passiveAction != -1
EndFunction

Bool Function OstimPlayerHasPenisStimulation(String sceneID, Int playerTarget, Int playerActor)
    Int passivePenisStim = OMetadata.FindAnyActionForTargetCSV(sceneID, playerTarget, "deepthroat,lickingpenis,grindingpenis,thighjob,handjob")
    Int activePenisStim = OMetadata.FindAnyActionForActorCSV( sceneID, playerActor, "analsex,malemasturbation,vaginalsex" )
    return passivePenisStim != -1 || activePenisStim != -1
EndFunction

Bool Function OstimPlayerIsPenetrated(String sceneID, Int playerTarget, Int playerActor)
    Int passiveAction = OMetadata.FindAnyActionForTargetCSV( sceneID, playerTarget, "analsex,analfisting,analfingering,deepthroat,lickingpenis,vaginalfisting" )
    return passiveAction != -1
EndFunction

Event OnOStimStart(string eventName, string strArg, float numArg, Form sender)
    TeleDevices.LogDebug("OnOStimStart")
    UnregisterForUpdate()
    UpdateRousingControlledSexScene()
EndEvent

Event OnOstimEnd(string eventName, string sceneID, float numArg, Form sender)
    TeleDevices.LogDebug("OnOstimEnd")
    If _OstimSceneVibrationHandle != -1
        TeleDevices.StopHandle(_OstimSceneVibrationHandle)
        _OstimSceneVibrationHandle = -1
    EndIf
EndEvent

Event OnOStimSceneChanged(string eventName, string sceneID, float numArg, Form sender)
    If OThread.GetScene(0) != sceneID
        return
    EndIf
    TeleDevices.LogDebug("OnOStimSceneChanged: " + sceneID + " "  + numArg)

    Int playerActorIndex = -1
    Int playerTargetIndex = -1

    Int[] sceneActors = OMetadata.GetActionActors(sceneID)
    Int i = sceneActors.Length
    While i > 0
        i -= 1
        Int actorIndex = sceneActors[i]
        If OStim.GetActor(actorIndex) == PlayerRef
            playerActorIndex = actorIndex
        EndIf
    EndWhile

    Int[] sceneTargets = OMetadata.GetActionTargets(sceneID)
    Int j = sceneTargets.Length
    While j > 0
        j -= 1
        Int actorIndex = sceneTargets[j]
        If OStim.GetActor(actorIndex) == PlayerRef
            playerTargetIndex = actorIndex
        EndIf
    EndWhile
    
    Bool hasVaginalStim = OstimPlayerHasVaginalStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasAnalStim = OstimPlayerHasAnalStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasNippleStim = OstimPlayerHasNippleStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool isPenetrated = OstimPlayerIsPenetrated(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasPenisStim = OstimPlayerHasPenisStimulation(sceneID, playerTargetIndex, playerActorIndex)
    
    String[] evts = new String[5]
    If hasVaginalStim
        evts[0] = Ostim_Animation_Event_Vaginal
    EndIf
    If hasNippleStim
        evts[1] = Ostim_Animation_Event_Nipple 
    EndIf
    If hasAnalStim
        evts[2] = Ostim_Animation_Event_Anal
    EndIf
    If hasPenisStim
        evts[3] = Ostim_Animation_Event_Penis
    EndIf
    If isPenetrated
        evts[4] = Ostim_Animation_Event_Penetration
    EndIf
    _OstimMaxSpeed = OMetadata.GetMaxSpeed(sceneID)

    Int oldHandle = _OstimSceneVibrationHandle
    If hasVaginalStim || hasAnalStim || hasNippleStim || hasPenisStim || isPenetrated
        _OstimSceneVibrationHandle = StartVibration(Ostim_Animation_DeviceSelector, -1, Ostim_Animation_Pattern, Ostim_Animation_Funscript, GetOStimSpeed(), evts)
    Else
        _OstimSceneVibrationHandle = -1
    EndIf
    If oldHandle != -1
        TeleDevices.StopHandle(oldHandle)
    EndIf
EndEvent

Int Function GetOStimSpeed()
    If Ostim_Animation_Speed_Control == 1
        Float speed = OThread.GetSpeed(0) as Float
        If speed == 0.0
            speed = 0.5
        EndIf
        Float factor = speed / _OstimMaxSpeed as Float
        return (100 * factor) as Int
    EndIf

    If Ostim_Animation_Speed_Control == 2
        Float excitement = OActor.GetExcitement(PlayerRef)
        return excitement as Int
    EndIf

    If Ostim_Animation_Speed_Control == 3
        Float speed = OThread.GetSpeed(0) as Float
        If speed == 0.0
            speed = 0.5
        EndIf
        Float speedFactor = speed / (_OstimMaxSpeed as Float)
        Float excitement = OActor.GetExcitement(PlayerRef)
        return (excitement * speedFactor) as Int
    EndIf

    return 100
EndFunction

; Toys & Love Events

Event OnToysPulsate(string eventName, string argString, float argNum, form sender)
    ; Duration is random lasting from approx. 12 to 35 seconds
    Int duration = Utility.RandomInt(12,35)
    String[] events = new String[1]
    events[0] = Toys_Vibrate_Event
        StartVibration(Toys_Vibrate_DeviceSelector, duration, Toys_Vibrate_Pattern, Toys_Vibrate_Funscript, Toys_Vibrate_Linear_Strength, events)
	; TeleDevices.LogDebug("ToysPulsate")
EndEvent

Int _ToysFondleHandle = -1
Event OnToysFondleStart(string eventName, string argString, float argNum, form sender)
    ; Fondle started - successfully increased rousing
	_ToysFondleHandle = TeleDevices.Vibrate(40, -1)
	; TeleDevices.LogDebug("ToysFondleStart")
EndEvent

Event OnToysFondleEnd(string eventName, string argString, float argNum, form sender)
    ; Fondle animation has ended (no player controls locking). Anim duration is 10 to 18 seconds.
	TeleDevices.StopHandle(_ToysFondleHandle)
	; TeleDevices.LogDebug("ToysFondleEnd")
EndEvent

Event OnToysSquirt(string eventName, string argString, float argNum, form sender)
    ; SquirtingEffect has started. There can be numerous in a single scene. Is not sent if turned off in MCM. Duration is 12 seconds
	TeleDevices.Vibrate(100, 12.0)
	; TeleDevices.LogDebug("ToysSquirt")
EndEvent

Event OnToysLoveSceneInfo(string loveName, Bool playerInScene, int numStages, Bool playerConsent, Form actInPos1, Form actInPos2, Form actInPos3, Form actInPos4, Form actInPos5)
    ; - ToysLoveSceneInfo - Dual purpose event: 1) Get Scene Info. 2) Event indicates start of animating. It's the moment actors are in place and the first animation has started. Scene Info includes:
    ; 	- LoveName, PlayerInScene, NumStages, PlayerConsent, ActInPos1.. Pos2.. Pos3.. Pos4.. Pos5
    ; 	- Actors as Form, given in scene position. The Player will always be in Position 1 or 2
    ; 	- event is sent for Player-less scenes. The param PlayerInScene will be false
    ; 	**Custom Parameters** Event <callbackName>(string LoveName, Bool PlayerInScene, int NumStages, Bool PlayerConsent, Form ActInPos1, Form ActInPos2, Form ActInPos3, Form ActInPos4, Form ActInPos5)
    If ! playerInScene
        return
    EndIf

    String[] events = GetLoveTags(loveName)
    _ToysSceneVibrationHandle = StartVibration(Toys_Animation_DeviceSelector, -1, Toys_Animation_Pattern, Toys_Animation_Funscript, Toys_Animation_Linear_Strength, events)

    If Toys_Animation_Rousing
        _InToysScene = true
        UnregisterForUpdate()
        UpdateRousingControlledSexScene()
    EndIf
EndEvent

String[] Function GetLoveTags(String loveName)
    String[] events = new String[4]
    If (Toys as ToysFramework).SceneHasTag( loveName, "Vaginal") ; || Toys.SceneHasTag( loveName, "Pussy") || Toys.SceneHasTag( loveName, "Fisting") 
        events[0] = Toys_Animation_Event_Vaginal
    EndIf
    If (Toys as ToysFramework).SceneHasTag( loveName, "Anal") ;|| Toys.SceneHasTag( loveName, "Fisting")
        events[1] = Toys_Animation_Event_Anal
    EndIf
    If (Toys as ToysFramework).SceneHasTag( loveName, "Oral") ;|| Toys.SceneHasTag( loveName, "Blowjob")
        events[2] = Toys_Animation_Event_Oral
    EndIf
    If (Toys as ToysFramework).SceneHasTag( loveName, "Nipple") || (Toys as ToysFramework).SceneHasTag( loveName, "Breast"); || Toys.SceneHasTag( loveName, "Breast")
        events[3] = Toys_Animation_Event_Nipple
    EndIf
    return events
EndFunction

Event OnToysSceneEnd(string eventName, string argString, float argNum, form sender)
    _InToysScene = false
    TeleDevices.StopHandle(_ToysSceneVibrationHandle)
	; TeleDevices.LogDebug("OnToysSceneEnd")
EndEvent

Event OnToysClimax(string eventName, string argString, float argNum, form sender)
    ; Simultaneous Orgasm. Both player & NPC have climaxed. This can happen multiple times. Sent in addition to other climax events. This event always first
	TeleDevices.Vibrate(80, 5)
	; TeleDevices.LogDebug("OnToysClimax")
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	TeleDevices.Vibrate(100, 7)
	; TeleDevices.LogDebug("OnToysClimaxSimultaneous")
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	TeleDevices.Vibrate(0, 7)
	; TeleDevices.LogDebug("OnToysDenied")
EndEvent

Event OnToysVaginalPenetration(string eventName, string argString, float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Vaginal"
    TeleDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
 	; TeleDevices.LogDebug("OnToysVaginalPenetration")
EndEvent

Event OnToysAnalPenetration(string eventName, string argString, float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Anal"
    TeleDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
 	; TeleDevices.LogDebug("OnToysAnalPenetration")
EndEvent

Event OnToysOralPenetration(string eventName, string argString, float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Oral"
    TeleDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
 	; TeleDevices.LogDebug("OnToysOralPenetration")
EndEvent

; Skyrim Chain Beasts Events

Event OnSCB_VibeEvent(string eventName, string strArg, float numArg, Form sender)
    String[] evts = new String[1]
    evts[0] = Chainbeasts_Vibrate_Event
    StartVibration(Chainbeasts_Vibrate_DeviceSelector, 3, Chainbeasts_Vibrate_Pattern, Chainbeasts_Vibrate_Funscript, Chainbeasts_Vibrate_Linear_Strength, evts)
	; TeleDevices.LogDebug("OnSCB_VibeEvent")
EndEvent

; Publics

Function InitDefaultOnEventHandlers()
    EmergencyHotkey = EmergencyHotkey_Default
    DeviousDevices_Vibrate = true
    Toys_Vibrate = true
    Chainbeasts_Vibrate = true
EndFunction

Function ResetIntegrationSettings()
    TeleDevices.Notify("All settings reset to default")
    DeviousDevices_Vibrate = DeviousDevices_Vibrate_Default
    DeviousDevices_Vibrate_DeviceSelector = DeviousDevices_Vibrate_DeviceSelector_Default
    DeviousDevices_Vibrate_Event_Anal = DeviousDevices_Vibrate_Event_Anal_Default
    DeviousDevices_Vibrate_Event_Vaginal = DeviousDevices_Vibrate_Event_Vaginal_Default
    DeviousDevices_Vibrate_Event_Nipple = DeviousDevices_Vibrate_Event_Nipple_Default
    DeviousDevices_Vibrate_Funscript = DeviousDevices_Vibrate_Funscript_Default
    DeviousDevices_Vibrate_Pattern = DeviousDevices_Vibrate_Pattern_Default
    Sexlab_Animation = Sexlab_Animation_Default
    Sexlab_Animation_DeviceSelector = Sexlab_Animation_DeviceSelector_Default
    Sexlab_Animation_Funscript = Sexlab_Animation_Funscript_Default
    Sexlab_Animation_Pattern = Sexlab_Animation_Pattern_Default
    Sexlab_Animation_Linear_Strength = Sexlab_Animation_Linear_Strength_Default
    Sexlab_ActorOrgasm = Sexlab_ActorOrgasm_Default
    Sexlab_ActorEdge = Sexlab_ActorEdge_Default
    Ostim_Animation_Funscript = Ostim_Animation_Funscript_Default
    Ostim_Animation_DeviceSelector = Ostim_Animation_DeviceSelector_Default
    Ostim_Animation_Pattern = Ostim_Animation_Pattern_Default
    Ostim_Animation_Event_Anal = Ostim_Animation_Event_Anal_Default
    Ostim_Animation_Event_Vaginal = Ostim_Animation_Event_Vaginal_Default
    Ostim_Animation_Event_Nipple = Ostim_Animation_Event_Nipple_Default
    Ostim_Animation_Event_Penetration = Ostim_Animation_Event_Penetration_Default
    Ostim_Animation_Event_Penis = Ostim_Animation_Event_Penis_Default
    Ostim_Animation_Speed_Control = Ostim_Animation_Speed_Control_Default
    Toys_Animation = Toys_Animation_Default
    Toys_Animation_DeviceSelector = Toys_Animation_DeviceSelector_Default
    Toys_Animation_Rousing = Toys_Animation_Rousing_Default
    Toys_Animation_Match_Tags = Toys_Animation_Match_Tags_Default
    Toys_Animation_Event_Vaginal = Toys_Animation_Event_Vaginal_Default
    Toys_Animation_Event_Oral = Toys_Animation_Event_Oral_Default
    Toys_Animation_Event_Anal = Toys_Animation_Event_Anal_Default
    Toys_Animation_Event_Nipple = Toys_Animation_Event_Nipple_Default
    Toys_Animation_Funscript = Toys_Animation_Funscript_Default
    Toys_Animation_Pattern = Toys_Animation_Pattern_Default
    Toys_Animation_Linear_Strength = Toys_Animation_Linear_Strength_Default
    Toys_Caressed = Toys_Caressed_Default
    Toys_Climax = Toys_Climax_Default
    Toys_Denial = Toys_Denial_Default
    Toys_Fondle = Toys_Fondle_Default
    Toys_Squirt = Toys_Squirt_Default
    Toys_Vibrate = Toys_Vibrate_Default
    Toys_Vibrate_DeviceSelector = Toys_Vibrate_DeviceSelector_Default
    Toys_Vibrate_Event = Toys_Vibrate_Event_Default
    Toys_Vibrate_Funscript = Toys_Vibrate_Funscript_Default
    Toys_Vibrate_Pattern = Toys_Vibrate_Pattern_Default
    Toys_Vibrate_Linear_Strength = Toys_Vibrate_Linear_Strength_Default
    Toys_Vaginal_Penetration = Toys_Vaginal_Penetration_Default
    Toys_Anal_Penetration = Toys_Anal_Penetration_Default
    Toys_Oral_Penetration = Toys_Oral_Penetration_Default
    Chainbeasts_Vibrate = Chainbeasts_Vibrate_Default
    Chainbeasts_Vibrate_DeviceSelector = Chainbeasts_Vibrate_DeviceSelector_Default
    Chainbeasts_Vibrate_Event = Chainbeasts_Vibrate_Event_Default
    Chainbeasts_Vibrate_Funscript = Chainbeasts_Vibrate_Funscript_Default
    Chainbeasts_Vibrate_Pattern = Chainbeasts_Vibrate_Pattern_Default
    Chainbeasts_Vibrate_Linear_Strength = Chainbeasts_Vibrate_Linear_Strength_Default
    EmergencyHotkey = EmergencyHotkey_Default
EndFunction

; Privates

Event OnUpdate()
    UpdateRousingControlledSexScene()
EndEvent

Function UpdateRousingControlledSexScene()
    If _InToysScene
        Int rousing = (Toys as ToysFramework).GetRousing()
        TeleDevices.UpdateHandle(_ToysSceneVibrationHandle, rousing)
    ElseIf _InSexlabScene
        Int arousal = (SexLabAroused as slaFrameworkScr).GetActorArousal(PlayerRef)
        TeleDevices.UpdateHandle(_ToysSceneVibrationHandle, arousal)
	ElseIf _OstimSceneVibrationHandle != -1
        TeleDevices.UpdateHandle(_OstimSceneVibrationHandle, GetOStimSpeed())
    EndIf
    RegisterForSingleUpdate(2)
EndFunction

Int Function StartVibration(Int deviceSelector, Float duration_sec, Int patternType, String funscript, Int speed, String[] evts)
    String[] events = new String[1]
    If deviceSelector == 1
        events = evts
    EndIf
    If patternType == 2
        String random_funscript = TeleDevices.GetRandomPattern(true)
        return TeleDevices.VibratePattern(random_funscript, speed, duration_sec, events)
    ElseIf patternType == 1
        return TeleDevices.VibratePattern(funscript, speed, duration_sec, events)
    Else
        return TeleDevices.VibrateEvents(speed, duration_sec, events)
    EndIf
EndFunction

; Version Updates

Function MigrateToV12()
    UnregisterForUpdate()
EndFunction