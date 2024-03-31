ScriptName Tele_Integration extends Quest
{
    Integrates devices with the game and mods
    ~ Use this API to enable/disable integration features ~
}

Tele_Devices _TeleDevices = None
Tele_Devices Property TDevices Hidden
    Tele_Devices Function Get()
        If _TeleDevices == None
            _TeleDevices = (self as Quest) as Tele_Devices
        EndIf
        return _TeleDevices
    EndFunction
EndProperty 

; Reset by OnGameLoadObserver on every start
Actor Property PlayerRef Auto Hidden

; Type ZadLibs
Quest Property ZadLib Auto Hidden 

; Type SexLabFramework
Quest Property SexLab Auto Hidden

; Type ToysFramework
Quest Property Toys Auto Hidden

; Type SlaFrameworkScr
Quest Property SexLabAroused Auto Hidden

; Type MilkQuest
Bool Property MilkMod Auto Hidden

; Using the type OSexIntegrationMain breaks the script loading
Bool Property HasOStim Auto Hidden

Event OnInit()
    InitDefaultOnEventHandlers() ; TODO unneeded?
    InitDefaultListeners()
EndEvent

Event OnUpdate()
    UpdateRousingControlledSexScene()
EndEvent

Int _SexlabSceneVibrationHandle = -1
Int _SexlabSceneStrokerHandle = -1
Int _SexlabSceneOscillatorHandle = -1
Int _OstimSceneVibrationHandle = -1
Int _OstimSceneStrokerHandle = -1
Int _OstimSceneOscillatorHandle = -1
Int _ToysSceneVibrationHandle = -1
Bool _InSexlabScene = false
Bool _InToysScene = false

Function Maintenance()
    ; Resuming scenes on game load is not supported, just reset it
    _SexlabSceneVibrationHandle = -1
    _SexlabSceneStrokerHandle = -1
    _SexlabSceneOscillatorHandle = -1
    _OstimSceneVibrationHandle = -1
    _OstimSceneStrokerHandle = -1
    _OstimSceneOscillatorHandle = -1
    _ToysSceneVibrationHandle = -1
    _InSexlabScene = false
    _InToysScene = false
    UnregisterForUpdate()
EndFunction

Function InitDefaultListeners()
    InitDeviousDevicesHandlers()
    InitOstimHandlers()
    InitSexlabHandlers()
    InitMilkModHandlers()
EndFunction

Function InitDefaultOnEventHandlers()
    EmergencyHotkey = EmergencyHotkey_Default
    DeviousDevices_Vibrate = true
    Toys_Vibrate = true
    Chainbeasts_Vibrate = true
    MilkMod_Vibrate = true
EndFunction

Function UpdateRousingControlledSexScene()
    ; TDevices.LogDebug("UpdateRousingControlled OS-Stroker: " + _OstimSceneStrokerHandle + " OS-Vib " + _OstimSceneVibrationHandle + " SL-Stroker:" + _SexlabSceneStrokerHandle + " SLVib: " + _SexlabSceneVibrationHandle)
    If _InToysScene
        Int speed = (Toys as ToysFramework).GetRousing()
        TDevices.UpdateHandle(_ToysSceneVibrationHandle, speed)
    EndIf

    If _InSexlabScene
        Int speed = (SexLabAroused as slaFrameworkScr).GetActorArousal(PlayerRef)
        If _SexlabSceneVibrationHandle != -1
            TDevices.UpdateHandle(_SexlabSceneVibrationHandle, speed)
        EndIf
        If _SexlabSceneStrokerHandle != -1
            TDevices.UpdateHandle(_SexlabSceneStrokerHandle, speed)
        EndIf
        If _SexlabSceneOscillatorHandle != -1
            TDevices.UpdateHandle(_SexlabSceneOscillatorHandle, speed)
        EndIf
    EndIf
    
    If _OstimSceneVibrationHandle != -1
        Int speed = GetOStimSpeed(Ostim_Animation_Speed_Control)
        If (speed == 0)
            speed = 1
        EndIf
        TDevices.UpdateHandle(_OstimSceneVibrationHandle, speed)
    EndIf
    If _OstimSceneStrokerHandle != -1
        If Ostim_Stroker_Pattern == 0
            Int speed = GetOStimSpeed(Ostim_Stroker_Speed_Control)
            TDevices.UpdateHandle(_OstimSceneStrokerHandle, speed)
        EndIf
    EndIf
    If _OstimSceneOscillatorHandle != -1
        TDevices.UpdateHandle(_OstimSceneOscillatorHandle, GetOStimSpeed(Ostim_Stroker_Speed_Control))
    EndIf
    RegisterForSingleUpdate(2)
EndFunction

Int Function StartStroke(Int deviceSelector, Float duration_sec, Int patternType, String funscript, Int speed, String[] evts)
    ; TDevices.LogDebug("StartStroke")
    String[] events = new String[1]
    If deviceSelector == 1
        events = evts
    EndIf
    If patternType > 0
        If patternType == 1
            funscript = TDevices.GetRandomPattern(false)
        EndIf
            return TDevices.LinearPattern(funscript, 100, duration_sec, events)
        EndIf
        return TDevices.Linear(speed, duration_sec, events)
EndFunction

Int Function StartOscillate(Int deviceSelector, Float duration_sec, Int speed, String[] evts)
    String[] events = new String[1]
    If deviceSelector == 1
        events = evts
    EndIf
    return TeleDevices.Scalar("oscillate", speed, duration_sec, events)
EndFunction

Int Function StartVibration(Int deviceSelector, Float duration_sec, Int patternType, String funscript, Int speed, String[] evts)
    String[] events = new String[1]
    If deviceSelector == 1
        events = evts
    EndIf
    If patternType == 2
        String random_funscript = TDevices.GetRandomPattern(true)
        return TDevices.VibratePattern(random_funscript, speed, duration_sec, events)
    ElseIf patternType == 1
        return TDevices.VibratePattern(funscript, speed, duration_sec, events)
    Else
        return TDevices.VibrateEvents(speed, duration_sec, events)
    EndIf
EndFunction

Function MigrateToV12()
    UnregisterForUpdate()
EndFunction

Function ResetIntegrationSettings()
    TDevices.Notify("All settings reset to default")
    DeviousDevices_Vibrate = DeviousDevices_Vibrate_Default
    DeviousDevices_Vibrate_DeviceSelector = DeviousDevices_Vibrate_DeviceSelector_Default
    DeviousDevices_Vibrate_Event_Anal = DeviousDevices_Vibrate_Event_Anal_Default
    DeviousDevices_Vibrate_Event_Vaginal = DeviousDevices_Vibrate_Event_Vaginal_Default
    DeviousDevices_Vibrate_Event_Nipple = DeviousDevices_Vibrate_Event_Nipple_Default
    DeviousDevices_Vibrate_Funscript = DeviousDevices_Vibrate_Funscript_Default
    DeviousDevices_Vibrate_Pattern = DeviousDevices_Vibrate_Pattern_Default
    MilkMod_Vibrate = MilkMod_Vibrate_Default
    MilkMod_Vibrate_DeviceSelector = MilkMod_Vibrate_DeviceSelector_Default
    MilkMod_Vibrate_Event_Anal = MilkMod_Vibrate_Event_Anal_Default
    MilkMod_Vibrate_Event_Vaginal = MilkMod_Vibrate_Event_Vaginal_Default
    MilkMod_Vibrate_Event_Nipple = MilkMod_Vibrate_Event_Nipple_Default
    MilkMod_Vibrate_Funscript = MilkMod_Vibrate_Funscript_Default
    MilkMod_Vibrate_Pattern = MilkMod_Vibrate_Pattern_Default
    MilkMod_Vibrate_Strength = MilkMod_Vibrate_Strength_Default
    Sexlab_Animation = Sexlab_Animation_Default
    Sexlab_Animation_DeviceSelector = Sexlab_Animation_DeviceSelector_Default
    Sexlab_Animation_Funscript = Sexlab_Animation_Funscript_Default
    Sexlab_Animation_Pattern = Sexlab_Animation_Pattern_Default
    Sexlab_Animation_Linear_Strength = Sexlab_Animation_Linear_Strength_Default
    Sexlab_ActorOrgasm = Sexlab_ActorOrgasm_Default
    Sexlab_ActorEdge = Sexlab_ActorEdge_Default
    Sexlab_Stroker = Sexlab_Stroker_Default
    Sexlab_Oscillator = Sexlab_Oscillator_Default
    Sexlab_Stroker_DeviceSelector = Sexlab_Stroker_DeviceSelector_Default
    Sexlab_Stroker_Funscript = Sexlab_Stroker_Funscript_Default
    Sexlab_Stroker_Pattern = Sexlab_Stroker_Pattern_Default
    Sexlab_Stroker_Linear_Strength = Sexlab_Stroker_Linear_Strength_Default
    Ostim_Oscillator = Ostim_Oscillator_Default
    Ostim_Stroker = Ostim_Stroker_Default
    Ostim_Stroker_DeviceSelector = Ostim_Stroker_DeviceSelector_Default
    Ostim_Stroker_Funscript = Ostim_Stroker_Funscript_Default
    Ostim_Stroker_Speed_Control = Ostim_Stroker_Speed_Control_Default
    Ostim_Stroker_Pattern = Ostim_Stroker_Pattern_Default
    Ostim_Stroker_Funscript = Ostim_Stroker_Funscript_Default
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


;               ______                                               
;              / ____/___ ___  ___  _________ ____  ____  _______  __
;             / __/ / __ `__ \/ _ \/ ___/ __ `/ _ \/ __ \/ ___/ / / /
;            / /___/ / / / / /  __/ /  / /_/ /  __/ / / / /__/ /_/ / 
;           /_____/_/ /_/ /_/\___/_/   \__, /\___/_/ /_/\___/\__, /  
;                                     /____/                /____/   


Int _EmergencyHotkey = 211

Int Property EmergencyHotkey_Default = 211 AutoReadOnly Hidden ; del
Int Property EmergencyHotkey Hidden
    Function Set(Int keyCode)
        UnregisterForKey(_EmergencyHotkey)
        _EmergencyHotkey = keyCode
        RegisterForKey(_EmergencyHotkey)
    EndFunction
    Int Function Get()
        return _EmergencyHotkey
    EndFunction
EndProperty

Event OnKeyUp(Int keyCode, Float HoldTime)
    If keyCode == _EmergencyHotkey
        TDevices.EmergencyStop()
    Else
        TDevices.LogDebug("Unregistered keypress code: " + KeyCode)
    EndIf
EndEvent


;               __  ____ ____      __  ___          __   ______                                      
;              /  |/  (_) / /__   /  |/  /___  ____/ /  / ____/________  ____  ____  ____ ___  __  __
;             / /|_/ / / / //_/  / /|_/ / __ \/ __  /  / __/ / ___/ __ \/ __ \/ __ \/ __ `__ \/ / / /
;            / /  / / / / ,<    / /  / / /_/ / /_/ /  / /___/ /__/ /_/ / / / / /_/ / / / / / / /_/ / 
;           /_/  /_/_/_/_/|_|  /_/  /_/\____/\__,_/  /_____/\___/\____/_/ /_/\____/_/ /_/ /_/\__, /  
;                                                                                           /____/   


Int _MilkModVibrateHandle = -1
Int Property MilkMod_Vibrate_DeviceSelector = 0 Auto Hidden
Int Property MilkMod_Vibrate_DeviceSelector_Default = 0 AutoReadOnly Hidden
String Property MilkMod_Vibrate_Event_Anal = "Anal" Auto Hidden
String Property MilkMod_Vibrate_Event_Anal_Default = "Anal" Auto Hidden
String Property MilkMod_Vibrate_Event_Vaginal = "Vaginal" Auto Hidden
String Property MilkMod_Vibrate_Event_Vaginal_Default = "Vaginal" Auto Hidden
String Property MilkMod_Vibrate_Event_Nipple = "Nipple" Auto Hidden
String Property MilkMod_Vibrate_Event_Nipple_Default = "Nipple" Auto Hidden
Int Property MilkMod_Vibrate_Pattern = 0 Auto Hidden
Int Property MilkMod_Vibrate_Pattern_Default = 0 AutoReadOnly Hidden
Bool Property MilkMod_Vibrate_Default = true AutoReadOnly Hidden
String Property MilkMod_Vibrate_Funscript = "30_Sawtooth" Auto Hidden
String Property MilkMod_Vibrate_Funscript_Default = "30_Sawtooth" Auto Hidden
Int Property MilkMod_Vibrate_Strength = 60 Auto Hidden
Int Property MilkMod_Vibrate_Strength_Default = 60 Auto Hidden

Bool _MilkMod_Vibrate = false
Bool Property MilkMod_Vibrate Hidden
    Function Set(Bool enable)
        _MilkMod_Vibrate = enable
    EndFunction
    Bool Function Get()
        return _MilkMod_Vibrate
    EndFunction
EndProperty

Function InitMilkModHandlers()
	RegisterForModEvent("MilkQuest.StopMilkingMachine",  "OnMilkModVibrateEffectStop")	
    RegisterForModEvent("MilkQuest.StartMilkingMachine", "OnStartMilkingMachine")
	RegisterForModEvent("MilkQuest.FeedingStage", "OnFeedingStage")
	RegisterForModEvent("MilkQuest.MilkingStage",  "OnMilkingStage")
	RegisterForModEvent("MilkQuest.FuckMachineStage", "OnFuckMachineStage")
EndFunction

Event OnStartMilkingMachine(Form Who, Int mpas, Int MilkingType)
    Debug.Trace("[Tele] Milk Mod Handler vibrating on milking", 0)
    If !_MilkMod_Vibrate || Who != playerref
        return
    EndIf

    String[] events = new String[3]
    events[1] = MilkMod_Vibrate_Event_Anal
    events[2] = MilkMod_Vibrate_Event_Nipple

    TDevices.StopHandle(_MilkModVibrateHandle)
    Int StartMilkingStrength = Math.Floor(MilkMod_Vibrate_Strength * 0.6)
    _MilkModVibrateHandle = StartVibration(MilkMod_Vibrate_DeviceSelector, -1, MilkMod_Vibrate_Pattern, MilkMod_Vibrate_Funscript, StartMilkingStrength, events)
EndEvent

Event OnFuckMachineStage(Form Who, Int mpas, Int MilkingType)
    Debug.Trace("[Tele] Milk Mod Handler vibrating on Fuck Machine", 0)
    If !_MilkMod_Vibrate || Who != playerref
        return
    EndIf

    String[] events = new String[3]
    events[0] = MilkMod_Vibrate_Event_Vaginal
    events[1] = MilkMod_Vibrate_Event_Anal

    TDevices.StopHandle(_MilkModVibrateHandle)
    _MilkModVibrateHandle = StartVibration(MilkMod_Vibrate_DeviceSelector, -1, MilkMod_Vibrate_Pattern, MilkMod_Vibrate_Funscript, MilkMod_Vibrate_Strength, events)
EndEvent

Event OnMilkingStage(Form Who, Int mpas, Int MilkingType)
    Debug.Trace("[Tele] Milk Mod Handler vibrating on milking", 0)
    If !_MilkMod_Vibrate || Who != playerref
        return
    EndIf

    String[] events = new String[3]
    events[1] = MilkMod_Vibrate_Event_Anal
    events[2] = MilkMod_Vibrate_Event_Nipple

    TDevices.StopHandle(_MilkModVibrateHandle)
    _MilkModVibrateHandle = StartVibration(MilkMod_Vibrate_DeviceSelector, -1, MilkMod_Vibrate_Pattern, MilkMod_Vibrate_Funscript, MilkMod_Vibrate_Strength, events)
EndEvent

Event OnFeedingStage(Form Who, Int mpas, Int MilkingType)
    Debug.Trace("[Tele] Milk Mod Handler vibrating on feeding", 0)
    If !_MilkMod_Vibrate || Who != playerref
        return
    EndIf

    String[] events = new String[3]
    events[1] = MilkMod_Vibrate_Event_Anal
    events[2] = MilkMod_Vibrate_Event_Nipple

    TDevices.StopHandle(_MilkModVibrateHandle)
    Int StartMilkingStrength = Math.Floor(MilkMod_Vibrate_Strength * 0.6)
    _MilkModVibrateHandle = StartVibration(MilkMod_Vibrate_DeviceSelector, -1, MilkMod_Vibrate_Pattern, MilkMod_Vibrate_Funscript, StartMilkingStrength, events)
EndEvent

Event OnMilkModVibrateEffectStop(Form Who, Int mpas, Int MilkingType)
    If !_MilkMod_Vibrate || Who != playerref
        return
    EndIf

    TDevices.StopHandle(_MilkModVibrateHandle)
EndEvent



;               ____            _                      ____            _               
;              / __ \___ _   __(_)___  __  _______    / __ \___ _   __(_)_______  _____
;             / / / / _ \ | / / / __ \/ / / / ___/   / / / / _ \ | / / / ___/ _ \/ ___/
;            / /_/ /  __/ |/ / / /_/ / /_/ (__  )   / /_/ /  __/ |/ / / /__/  __(__  ) 
;           /_____/\___/|___/_/\____/\__,_/____/   /_____/\___/|___/_/\___/\___/____/  


Int _DeviousDevicesVibrateHandle = -1
Int Property DeviousDevices_Vibrate_DeviceSelector = 0 Auto Hidden
Int Property DeviousDevices_Vibrate_DeviceSelector_Default = 0 AutoReadOnly Hidden
String Property DeviousDevices_Vibrate_Event_Anal = "Anal" Auto Hidden
String Property DeviousDevices_Vibrate_Event_Anal_Default = "Anal" Auto Hidden
String Property DeviousDevices_Vibrate_Event_Vaginal = "Vaginal" Auto Hidden
String Property DeviousDevices_Vibrate_Event_Vaginal_Default = "Vaginal" Auto Hidden
String Property DeviousDevices_Vibrate_Event_Nipple = "Nipple" Auto Hidden
String Property DeviousDevices_Vibrate_Event_Nipple_Default = "Nipple" Auto Hidden
Int Property DeviousDevices_Vibrate_Pattern = 0 Auto Hidden
Int Property DeviousDevices_Vibrate_Pattern_Default = 0 AutoReadOnly Hidden
Bool Property DeviousDevices_Vibrate_Default = true AutoReadOnly Hidden
String Property DeviousDevices_Vibrate_Funscript = "30_Sawtooth" Auto Hidden
String Property DeviousDevices_Vibrate_Funscript_Default = "30_Sawtooth" Auto Hidden

Function InitDeviousDevicesHandlers()
    RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
    RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
EndFunction

Bool _DeviousDevices_Vibrate = false
Bool Property DeviousDevices_Vibrate Hidden
    Function Set(Bool enable)
        _DeviousDevices_Vibrate = enable
    EndFunction
    Bool Function Get()
        return _DeviousDevices_Vibrate
    EndFunction
EndProperty

Event OnVibrateEffectStart(String eventName, String actorName, Float vibrationStrength, Form sender)
    If !_DeviousDevices_Vibrate || ZadLib == None
        return
    EndIf
    If PlayerRef.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    ; TDevices.LogDebug("OnVibrateEffectStart"+ eventName + "," + actorName + "," + vibrationStrength + "," + sender)

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
EndEvent

Event OnVibrateEffectStop(String eventName, String actorName, Float argNum, form sender)
    If !_DeviousDevices_Vibrate || ZadLib == None
        return
    EndIf
    If PlayerRef.GetLeveledActorBase().GetName() != actorName
        return ; Not the player
    EndIf
    ; TDevices.LogDebug("OnVibrateEffectStop" + eventName + "," + actorName + "," + argNum + "," + sender)
    TDevices.StopHandle(_DeviousDevicesVibrateHandle)
EndEvent


;              _____           __      __  
;             / ___/___  _  __/ /___ _/ /_ 
;             \__ \/ _ \| |/_/ / __ `/ __ \
;            ___/ /  __/>  </ / /_/ / /_/ /
;           /____/\___/_/|_/_/\__,_/_.___/ 



String Property Sexlab_Animation_Funscript = "" Auto Hidden
String Property Sexlab_Animation_Funscript_Default = "" Auto Hidden
Int Property Sexlab_Animation_DeviceSelector = 0 Auto Hidden
Int Property Sexlab_Animation_DeviceSelector_Default = 0 AutoReadOnly Hidden
Bool Property Sexlab_Animation_Rousing = False Auto Hidden
Bool Property Sexlab_Animation_Rousing_Default = False AutoReadOnly Hidden
Int Property Sexlab_Animation_Pattern = 0 Auto Hidden
Int Property Sexlab_Animation_Pattern_Default = 0 AutoReadOnly Hidden

; TODO Unused?
Int Property Sexlab_Animation_Linear_Strength = 80 Auto Hidden
Int Property Sexlab_Animation_Linear_Strength_Default = 80 AutoReadOnly Hidden
; <<

String Property Sexlab_Stroker_Funscript = "" Auto Hidden
String Property Sexlab_Stroker_Funscript_Default = "" Auto Hidden
Int Property Sexlab_Stroker_DeviceSelector = 0 Auto Hidden
Int Property Sexlab_Stroker_DeviceSelector_Default = 0 AutoReadOnly Hidden
Bool Property Sexlab_Stroker_Rousing = False Auto Hidden
Bool Property Sexlab_Stroker_Rousing_Default = False AutoReadOnly Hidden
Int Property Sexlab_Stroker_Pattern = 0 Auto Hidden
Int Property Sexlab_Stroker_Pattern_Default = 0 AutoReadOnly Hidden

; TODO Unused?
Int Property Sexlab_Stroker_Linear_Strength = 80 Auto Hidden
Int Property Sexlab_Stroker_Linear_Strength_Default = 80 AutoReadOnly Hidden
; <<

Function InitSexlabHandlers()
    RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
    RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
EndFunction

Bool _Sexlab_Animation = false
Bool Property Sexlab_Animation_Default = true AutoReadOnly Hidden
Bool Property Sexlab_Animation Hidden
    Function Set(Bool enable)
        _Sexlab_Animation = enable
        If !enable
            _InSexlabScene = False
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_Animation
    EndFunction
EndProperty

Bool _Sexlab_Stroker = false
Bool Property Sexlab_Stroker_Default = true AutoReadOnly Hidden
Bool Property Sexlab_Stroker Hidden
    Function Set(Bool enable)
        _Sexlab_Stroker = enable
        If !enable && !_Sexlab_Oscillator
            _InSexlabScene = False
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_Stroker
    EndFunction
EndProperty

Bool _Sexlab_Oscillator = false
Bool Property Sexlab_Oscillator_Default = true AutoReadOnly Hidden
Bool Property Sexlab_Oscillator Hidden
    Function Set(Bool enable)
        _Sexlab_Oscillator = enable
        If !enable && !_Sexlab_Stroker
            _InSexlabScene = False
        EndIf
    EndFunction
    Bool Function Get()
        return _Sexlab_Oscillator
    EndFunction
EndProperty

Bool _Sexlab_ActorOrgasm = false
Bool Property Sexlab_ActorOrgasm_Default = false AutoReadOnly Hidden
Bool Property Sexlab_ActorOrgasm Hidden
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

Bool _Sexlab_ActorEdge = false
Bool Property Sexlab_ActorEdge_Default = false AutoReadOnly Hidden
Bool Property Sexlab_ActorEdge Hidden
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

Event OnSexlabAnimationStart(Int threadID, bool hasPlayer)
	If !hasPlayer
		return
	EndIf
    If !Sexlab_Animation && !Sexlab_Stroker && !Sexlab_Oscillator
        return
    EndIf
    ; TDevices.LogDebug("OnSexlabAnimationStart")
    sslThreadController controller = (Sexlab as SexLabFramework).GetController(threadID)
    sslBaseAnimation animation = controller.Animation

    Int speed = 80 ; this is not configurable: Sexlab_Stroker_Linear_Strength
    If Sexlab_Stroker_Rousing
        speed = (SexLabAroused as slaFrameworkScr).GetActorArousal(PlayerRef)
    EndIf
    
    If Sexlab_Animation
        _SexlabSceneVibrationHandle = StartVibration(Sexlab_Animation_DeviceSelector, -1, Sexlab_Animation_Pattern, Sexlab_Animation_Funscript, speed, animation.GetTags())
    EndIf
    If Sexlab_Stroker
        _SexlabSceneStrokerHandle = StartStroke(Sexlab_Stroker_DeviceSelector, -1, Sexlab_Stroker_Pattern, Sexlab_Stroker_Funscript, speed, animation.GetTags())
    EndIf
    If Sexlab_Oscillator
        _SexlabSceneOscillatorHandle = StartOscillate(Sexlab_Stroker_DeviceSelector, -1, speed, animation.GetTags())
    EndIf
    If Sexlab_Animation_Rousing || Sexlab_Stroker_Rousing
        _InSexlabScene = True
        UnregisterForUpdate()
        UpdateRousingControlledSexScene()
    EndIf
EndEvent

Event OnSexlabAnimationEnd(Int _, bool hasPlayer)
	If !hasPlayer
		return
	EndIf
    ; TDevices.LogDebug("OnSexlabAnimationEnd")
	_InSexlabScene = False
    If _SexlabSceneVibrationHandle != -1
        TDevices.StopHandle(_SexlabSceneVibrationHandle)
        _SexlabSceneVibrationHandle = -1
    EndIf
    If _SexlabSceneStrokerHandle != -1
        TDevices.StopHandle(_SexlabSceneStrokerHandle)
        _SexlabSceneStrokerHandle = -1
    EndIf
    If _SexlabSceneOscillatorHandle != -1
        TDevices.StopHandle(_SexlabSceneOscillatorHandle)
        _SexlabSceneOscillatorHandle = -1
    EndIf
    UnregisterForUpdate()
EndEvent

Event OnDeviceActorOrgasm(String eventName, String strArg, Float numArg, Form sender)
	TDevices.Vibrate(Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0))
EndEvent

Event OnDeviceEdgedActor(String eventName, String strArg, Float numArg, Form sender)
	TDevices.Vibrate(Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0))
EndEvent


;              ____       __  _         
;             / __ \_____/ /_(_)___ ___ 
;            / / / / ___/ __/ / __ `__ \
;           / /_/ (__  ) /_/ / / / / / /
;           \____/____/\__/_/_/ /_/ /_/ 


Int _OstimMaxSpeed = 4

String Property Ostim_Animation_Funscript = "" Auto Hidden
String Property Ostim_Animation_Funscript_Default = "" Auto Hidden
Int Property Ostim_Animation_DeviceSelector = 0 Auto Hidden
Int Property Ostim_Animation_DeviceSelector_Default = 0 AutoReadOnly Hidden
Int Property Ostim_Animation_Speed_Control = 1 Auto Hidden
Int Property Ostim_Animation_Speed_Control_Default = 1 AutoReadOnly Hidden
Int Property Ostim_Animation_Pattern = 0 Auto Hidden
Int Property Ostim_Animation_Pattern_Default = 0 AutoReadOnly Hidden
String Property Ostim_Animation_Event_Anal = "Anal" Auto Hidden
String Property Ostim_Animation_Event_Anal_Default = "Anal" Auto Hidden
String Property Ostim_Animation_Event_Vaginal = "Vaginal" Auto Hidden
String Property Ostim_Animation_Event_Vaginal_Default = "Vaginal" Auto Hidden
String Property Ostim_Animation_Event_Nipple = "Nipple" Auto Hidden
String Property Ostim_Animation_Event_Nipple_Default = "Nipple" Auto Hidden
String Property Ostim_Animation_Event_Penetration = "Penetration" Auto Hidden
String Property Ostim_Animation_Event_Penetration_Default = "Penetration" Auto Hidden
String Property Ostim_Animation_Event_Penis = "Penis" Auto Hidden
String Property Ostim_Animation_Event_Penis_Default = "Penis" Auto Hidden
Int Property Ostim_Stroker_DeviceSelector = 0 Auto Hidden
Int Property Ostim_Stroker_DeviceSelector_Default = 0 AutoReadOnly Hidden
String Property Ostim_Stroker_Funscript = "" Auto Hidden
String Property Ostim_Stroker_Funscript_Default = "" AutoReadOnly Hidden
Int Property Ostim_Stroker_Speed_Control = 1 Auto Hidden
Int Property Ostim_Stroker_Speed_Control_Default = 1 AutoReadOnly Hidden
Int Property Ostim_Stroker_Pattern = 0 Auto Hidden
Int Property Ostim_Stroker_Pattern_Default = 0 AutoReadOnly Hidden

Function InitOstimHandlers()
    RegisterForModEvent("OStim_Start", "OnOStimStart")
    RegisterForModEvent("OStim_SceneChanged", "OnOStimSceneChanged")
    RegisterForModEvent("OStim_End", "OnOstimEnd") 
EndFunction

Bool _Ostim_Animation = false
Bool Property Ostim_Animation_Default = true AutoReadOnly Hidden
Bool Property Ostim_Animation Hidden
    Function Set(Bool enable)
        _Ostim_Animation = enable
        If !enable
            If _OstimSceneVibrationHandle != -1
                TDevices.StopHandle(_OstimSceneVibrationHandle)
            EndIf
            _OstimSceneVibrationHandle = -1
        EndIf
    EndFunction
    Bool Function Get()
        return _Ostim_Animation
    EndFunction
EndProperty

Bool _Ostim_Stroker = false
Bool Property Ostim_Stroker_Default = false AutoReadOnly Hidden
Bool Property Ostim_Stroker Hidden
    Function Set(Bool enable)
        _Ostim_Stroker = enable
    EndFunction
    Bool Function Get()
        return _Ostim_Stroker
    EndFunction
EndProperty

Bool _Ostim_Oscillator = false
Bool Property Ostim_Oscillator_Default = false AutoReadOnly Hidden
Bool Property Ostim_Oscillator Hidden
    Function Set(Bool enable)
        _Ostim_Oscillator = enable
    EndFunction
    Bool Function Get()
        return _Ostim_Oscillator
    EndFunction
EndProperty

Event OnOStimStart(String eventName, String strArg, Float numArg, Form sender)
    ; TDevices.LogDebug("OnOStimStart")
    UpdateRousingControlledSexScene()
EndEvent

Event OnOstimEnd(String eventName, String sceneID, Float numArg, Form sender)
    ; TDevices.LogDebug("OnOstimEnd")
    If _OstimSceneVibrationHandle != -1
        TDevices.StopHandle(_OstimSceneVibrationHandle)
        _OstimSceneVibrationHandle = -1
    EndIf
    If _OstimSceneStrokerHandle != -1
        TDevices.StopHandle(_OstimSceneStrokerHandle)
        _OstimSceneStrokerHandle = -1
    EndIf
    If _OstimSceneOscillatorHandle != -1
        TDevices.StopHandle(_OstimSceneOscillatorHandle)
        _OstimSceneOscillatorHandle = -1
    EndIf
    UnregisterForUpdate()
EndEvent

Event OnOStimSceneChanged(String eventName, String sceneID, Float numArg, Form sender)
    If OThread.GetScene(0) != sceneID
        return
    EndIf
    If ! _Ostim_Animation && ! _Ostim_Stroker && ! _Ostim_Oscillator
        return
    EndIf

    ; TDevices.LogDebug("OnOStimSceneChanged " + sceneID  + " | " + OMetadata.GetAllActionsTags(sceneID))
    Int playerActorIndex = -1
    Int playerTargetIndex = -1

    Int[] sceneActors = OMetadata.GetActionActors(sceneID)
    Int i = sceneActors.Length
    While i > 0
        i -= 1
        Int actorIndex = sceneActors[i]
        If (HasOStim)
            If (OThread.GetActor(0, actorIndex) == PlayerRef)
                playerActorIndex = actorIndex
            EndIf
        EndIf
    EndWhile

    Int[] sceneTargets = OMetadata.GetActionTargets(sceneID)
    Int j = sceneTargets.Length
    While j > 0
        j -= 1
        Int actorIndex = sceneTargets[j]
        If (HasOStim)
            If (OThread.GetActor(0, actorIndex) == PlayerRef)
                playerTargetIndex = actorIndex
            EndIf
        EndIf
    EndWhile

    Bool isSexual = OMetadata.FindAnyActionCSV(sceneID, "sexual") > -1
    Bool hasVaginalStim = OstimPlayerHasVaginalStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasAnalStim = OstimPlayerHasAnalStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasNippleStim = OstimPlayerHasNippleStimulation(sceneID, playerTargetIndex, playerActorIndex)
    Bool isPenetrated = OstimPlayerIsPenetrated(sceneID, playerTargetIndex, playerActorIndex)
    Bool hasPenisStim = OstimSceneHasPenisStimulation(sceneID, playerTargetIndex, playerActorIndex)
       String[] evts = new String[6]
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
    If isSexual
        evts[5] = "Sexual"
    EndIf
    _OstimMaxSpeed = OMetadata.GetMaxSpeed(sceneID)

    Int oldHandle = _OstimSceneVibrationHandle
    Int oldStrokerHandle = _OstimSceneStrokerHandle

    If isSexual || hasVaginalStim || hasAnalStim || hasNippleStim || hasPenisStim || isPenetrated
        If Ostim_Animation
            _OstimSceneVibrationHandle = StartVibration(Ostim_Animation_DeviceSelector, -1, Ostim_Animation_Pattern, Ostim_Animation_Funscript, GetOStimSpeed(Ostim_Animation_Speed_Control), evts)
        EndIf
        If Ostim_Stroker
            Int speed = GetOStimSpeed(Ostim_Stroker_Speed_Control)
            _OstimSceneStrokerHandle = StartStroke(Ostim_Stroker_DeviceSelector, -1, Ostim_Stroker_Pattern, Ostim_Stroker_Funscript, speed, evts)
        EndIf
        If Ostim_Oscillator
            Int speed = GetOStimSpeed(Ostim_Stroker_Speed_Control)
            _OstimSceneOscillatorHandle = StartOscillate(Ostim_Stroker_DeviceSelector, -1, speed, evts)
        EndIf
    Else
        _OstimSceneStrokerHandle = - 1
        _OstimSceneVibrationHandle = -1
    EndIf
    If oldHandle != -1
        TDevices.StopHandle(oldHandle)
    EndIf
    If oldStrokerHandle != -1
        TDevices.StopHandle(oldStrokerHandle)
    EndIf
EndEvent

Int Function GetOStimSpeed(Int controlMode)
    If controlMode  == 1
        Float speed = OThread.GetSpeed(0) as Float
        If speed == 0.0
            speed = 0.5
        EndIf
        Float factor = speed / _OstimMaxSpeed as Float
        ; TDevices.LogDebug("GetOstimSpeed(1) " + factor + " speed: " + speed + " _OstimMaxSpeed: " + _OstimMaxSpeed)
        return (100 * factor) as Int
    EndIf

    If controlMode == 2
        Float excitement = OActor.GetExcitement(PlayerRef)
        ; TDevices.LogDebug("GetOstimSpeed(2) " + excitement)
        return excitement as Int
    EndIf

    If controlMode == 3
        Float speed = OThread.GetSpeed(0) as Float
        If speed == 0.0
            speed = 0.5
        EndIf
        Float speedFactor = speed / (_OstimMaxSpeed as Float)
        Float excitement = OActor.GetExcitement(PlayerRef)
        ; TDevices.LogDebug("GetOstimSpeed(3) speed: " + speed + " speedFactor" + speedFactor + " excitement: " + excitement + " _OstimMaxSpeed: " + _OstimMaxSpeed)
        return (excitement * speedFactor) as Int
    EndIf

    return 100
EndFunction

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

Bool Function OstimSceneHasPenisStimulation(String sceneID, Int playerTarget, Int playerActor)
    ; Int passivePenisStim = OMetadata.FindAnyActionForActorCSV( sceneID, playerTarget, "deepthroat,lickingpenis,grindingpenis,thighjob,handjob")
    ; Int activePenisStim = OMetadata.FindAnyActionForActorCSV( sceneID, playerActor, "analsex,malemasturbation,vaginalsex" )
    Int passivePenisStim = OMetadata.FindAnyActionCSV( sceneID, "deepthroat,lickingpenis,grindingpenis,thighjob,handjob")
    Int activePenisStim = OMetadata.FindAnyActionCSV( sceneID, "analsex,malemasturbation,vaginalsex" )
    return passivePenisStim != -1 || activePenisStim != -1
EndFunction

Bool Function OstimPlayerIsPenetrated(String sceneID, Int playerTarget, Int playerActor)
    Int passiveAction = OMetadata.FindAnyActionForTargetCSV( sceneID, playerTarget, "analsex,analfisting,analfingering,deepthroat,lickingpenis,vaginalfisting" )
    return passiveAction != -1
EndFunction


;             ______                    ___          __                  
;            /_  __/___  __  _______   ( _ )        / /   ____ _   _____ 
;             / / / __ \/ / / / ___/  / __ \/|     / /   / __ \ | / / _ \
;            / / / /_/ / /_/ (__  )  / /_/  <     / /___/ /_/ / |/ /  __/
;           /_/  \____/\__, /____/   \____/\/    /_____/\____/|___/\___/ 
;                     /____/                                             



Bool _Toys_Vaginal_Penetration = false
Bool Property Toys_Vaginal_Penetration_Default = false AutoReadOnly Hidden
Bool Property Toys_Vaginal_Penetration Hidden
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

Bool _Toys_Anal_Penetration = false
Bool Property Toys_Anal_Penetration_Default = false AutoReadOnly Hidden
Bool Property Toys_Anal_Penetration Hidden
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

Bool _Toys_Oral_Penetration = false
Bool Property Toys_Oral_Penetration_Default = false AutoReadOnly Hidden
Bool Property Toys_Oral_Penetration Hidden
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

Bool _Toys_Fondle = false
Bool Property Toys_Fondle_Default = false AutoReadOnly Hidden
Bool Property Toys_Fondle Hidden
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

Bool _Toys_Squirt = false
Bool Property Toys_Squirt_Default = false AutoReadOnly Hidden
Bool Property Toys_Squirt Hidden
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

Bool _Toys_Climax = false
Bool Property Toys_Climax_Default = false AutoReadOnly Hidden
Bool Property Toys_Climax Hidden
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

String Property Toys_Vibrate_Funscript = "" Auto Hidden
String Property Toys_Vibrate_Funscript_Default = "" Auto Hidden
Int Property Toys_Vibrate_DeviceSelector = 0 Auto Hidden
Int Property Toys_Vibrate_DeviceSelector_Default = 0 AutoReadOnly Hidden
String Property Toys_Vibrate_Event = "Vaginal" Auto Hidden
String Property Toys_Vibrate_Event_Default = "Vaginal" AutoReadOnly Hidden
Int Property Toys_Vibrate_Pattern = 0 Auto Hidden
Int Property Toys_Vibrate_Pattern_Default = 0 AutoReadOnly Hidden
Int Property Toys_Vibrate_Linear_Strength = 80 Auto Hidden
Int Property Toys_Vibrate_Linear_Strength_Default = 80 AutoReadOnly Hidden

Bool _Toys_Vibrate = false
Bool Property Toys_Vibrate_Default = true AutoReadOnly Hidden
Bool Property Toys_Vibrate Hidden
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

String Property Toys_Animation_Funscript = "" Auto Hidden
String Property Toys_Animation_Funscript_Default = "" Auto Hidden
Int Property Toys_Animation_DeviceSelector = 0 Auto Hidden
Int Property Toys_Animation_DeviceSelector_Default = 0 AutoReadOnly Hidden
Bool Property Toys_Animation_Match_Tags = false Auto Hidden
Bool Property Toys_Animation_Match_Tags_Default = false AutoReadOnly Hidden
String Property Toys_Animation_Event_Vaginal = "Vaginal" Auto Hidden
String Property Toys_Animation_Event_Vaginal_Default = "Vaginal" AutoReadOnly Hidden
String Property Toys_Animation_Event_Oral = "Oral" Auto Hidden
String Property Toys_Animation_Event_Oral_Default = "Oral" AutoReadOnly Hidden
String Property Toys_Animation_Event_Anal = "Anal" Auto Hidden
String Property Toys_Animation_Event_Anal_Default = "Anal" AutoReadOnly Hidden
String Property Toys_Animation_Event_Nipple = "Nipple" Auto Hidden
String Property Toys_Animation_Event_Nipple_Default = "Nipple" AutoReadOnly Hidden
Bool Property Toys_Animation_Rousing = true Auto Hidden
Bool Property Toys_Animation_Rousing_Default = true AutoReadOnly Hidden
Int Property Toys_Animation_Pattern = 0 Auto Hidden
Int Property Toys_Animation_Pattern_Default = 0 AutoReadOnly Hidden
Int Property Toys_Animation_Linear_Strength = 80 Auto Hidden
Int Property Toys_Animation_Linear_Strength_Default = 80 AutoReadOnly Hidden
Bool Property Toys_Animation_Default = true AutoReadOnly Hidden

Bool _Toys_Animation = false
Bool Property Toys_Animation Hidden
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
 
Bool _Toys_Caressed = false
Bool Property Toys_Caressed_Default = false AutoReadOnly Hidden
Bool Property Toys_Caressed Hidden
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

Bool _Toys_Denial = false
Bool Property Toys_Denial_Default = false AutoReadOnly Hidden
Bool Property Toys_Denial Hidden
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

Event OnToysPulsate(String eventName, String argString, Float argNum, form sender)
    ; Duration is random lasting from approx. 12 to 35 seconds
    Int duration = Utility.RandomInt(12,35)
    String[] events = new String[1]
    events[0] = Toys_Vibrate_Event
    StartVibration(Toys_Vibrate_DeviceSelector, duration, Toys_Vibrate_Pattern, Toys_Vibrate_Funscript, Toys_Vibrate_Linear_Strength, events)
EndEvent

Int _ToysFondleHandle = -1
Event OnToysFondleStart(String eventName, String argString, Float argNum, form sender)
    ; Fondle started - successfully increased rousing
	_ToysFondleHandle = TDevices.Vibrate(40, -1)
EndEvent

Event OnToysFondleEnd(String eventName, String argString, Float argNum, form sender)
    ; Fondle animation has ended (no player controls locking). Anim duration is 10 to 18 seconds.
	TDevices.StopHandle(_ToysFondleHandle)
EndEvent

Event OnToysSquirt(String eventName, String argString, Float argNum, form sender)
    ; SquirtingEffect has started. There can be numerous in a single scene. Is not sent if turned off in MCM. Duration is 12 seconds
	TDevices.Vibrate(100, 12.0)
EndEvent

Event OnToysLoveSceneInfo(String loveName, Bool playerInScene, Int numStages, Bool playerConsent, Form actInPos1, Form actInPos2, Form actInPos3, Form actInPos4, Form actInPos5)
    ; - ToysLoveSceneInfo - Dual purpose event: 1) Get Scene Info. 2) Event indicates start of animating. It's the moment actors are in place and the first animation has started. Scene Info includes:
    ; 	- LoveName, PlayerInScene, NumStages, PlayerConsent, ActInPos1.. Pos2.. Pos3.. Pos4.. Pos5
    ; 	- Actors as Form, given in scene position. The Player will always be in Position 1 or 2
    ; 	- event is sent for Player-less scenes. The param PlayerInScene will be false
    ; 	**Custom Parameters** Event <callbackName>(String LoveName, Bool PlayerInScene, Int NumStages, Bool PlayerConsent, Form ActInPos1, Form ActInPos2, Form ActInPos3, Form ActInPos4, Form ActInPos5)
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

Event OnToysSceneEnd(String eventName, String argString, Float argNum, form sender)
    _InToysScene = false
    TDevices.StopHandle(_ToysSceneVibrationHandle)
EndEvent

Event OnToysClimax(String eventName, String argString, Float argNum, form sender)
    ; Simultaneous Orgasm. Both player & NPC have climaxed. This can happen multiple times. Sent in addition to other climax events. This event always first
	TDevices.Vibrate(80, 5)
EndEvent

Event OnToysClimaxSimultaneous(String eventName, String argString, Float argNum, form sender)
	TDevices.Vibrate(100, 7)
EndEvent

Event OnToysDenied(String eventName, String argString, Float argNum, form sender)
	TDevices.Vibrate(0, 7)
EndEvent

Event OnToysVaginalPenetration(String eventName, String argString, Float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Vaginal"
    TDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
EndEvent

Event OnToysAnalPenetration(String eventName, String argString, Float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Anal"
    TDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
EndEvent

Event OnToysOralPenetration(String eventName, String argString, Float argNum, form sender)
    String[] events = new String[1]
    events[0] = "Oral"
    TDevices.VibrateEvents(Utility.RandomInt(80, 100), 12, events)
EndEvent

            
;              ________          _       __                    __      
;             / ____/ /_  ____ _(_)___  / /_  ___  ____ ______/ /______
;            / /   / __ \/ __ `/ / __ \/ __ \/ _ \/ __ `/ ___/ __/ ___/
;           / /___/ / / / /_/ / / / / / /_/ /  __/ /_/ (__  ) /_(__  ) 
;           \____/_/ /_/\__,_/_/_/ /_/_.___/\___/\__,_/____/\__/____/  


Int Property Chainbeasts_Vibrate_DeviceSelector = 0 Auto Hidden
Int Property Chainbeasts_Vibrate_DeviceSelector_Default = 0 AutoReadOnly Hidden
String Property Chainbeasts_Vibrate_Event = "Vaginal" Auto Hidden
String Property Chainbeasts_Vibrate_Event_Default = "Vaginal" AutoReadOnly Hidden
Int Property Chainbeasts_Vibrate_Pattern = 1 Auto Hidden
Int Property Chainbeasts_Vibrate_Pattern_Default = 1 AutoReadOnly Hidden
String Property Chainbeasts_Vibrate_Funscript = "03_Wub-Wub-Wub" Auto Hidden
String Property Chainbeasts_Vibrate_Funscript_Default = "03_Wub-Wub-Wub" Auto Hidden
Int Property Chainbeasts_Vibrate_Linear_Strength = 80 Auto Hidden
Int Property Chainbeasts_Vibrate_Linear_Strength_Default = 80 AutoReadOnly Hidden

Bool _Chainbeasts_Vibrate = false
Bool Property Chainbeasts_Vibrate_Default = true AutoReadOnly Hidden
Bool Property Chainbeasts_Vibrate Hidden
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

Event OnSCB_VibeEvent(String eventName, String strArg, Float numArg, Form sender)
    String[] evts = new String[1]
    evts[0] = Chainbeasts_Vibrate_Event
    StartVibration(Chainbeasts_Vibrate_DeviceSelector, 3, Chainbeasts_Vibrate_Pattern, Chainbeasts_Vibrate_Funscript, Chainbeasts_Vibrate_Linear_Strength, evts)
	; TDevices.LogDebug("OnSCB_VibeEvent")
EndEvent

; Depracted

Tele_Devices Property TeleDevices Auto
Quest Property OStim Auto Hidden
