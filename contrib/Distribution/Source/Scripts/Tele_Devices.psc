ScriptName Tele_Devices extends Quest

Spell Property Tele_VibrateSpellStrong auto
Spell Property Tele_VibrateSpellMedium auto
Spell Property Tele_VibrateSpellWeak auto
Spell Property Tele_Stop auto

Int Property MajorVersion = 1 autoReadOnly
Int Property MinorVersion = 0 autoReadOnly
String Property Revsision = "RC1" autoReadOnly

Int Property ScanTime = 30 auto

Bool Property LogDeviceConnects = true auto
Bool Property LogDeviceEvents = false auto
Bool Property LogDebugEvents = false auto

Bool Property ScanningForDevices = false auto
Int Property ConnectionType = 0 auto

Event OnInit()
    Notify("Telekinesis v" + MajorVersion + "." + MinorVersion + Revsision + ": Enable connected devices in MCM for usage...")
    ConnectAndScanForDevices()
    RegisterForUpdate(5)
EndEvent

Event OnUpdate()
    String[] evts = Tele_Api.PollEvents()
    Int i = 0
    While (i < evts.Length)
        String evt = evts[i]
        If StringUtil.Find(evt, "connected") != -1 || StringUtil.Find(evt, "removed") != -1
            LogConnection(evt)
        ElseIf StringUtil.Find( evt, "Vibrated") != -1
            LogEvent(evt)
        Else
            LogDebug(evt)
        EndIf
        i += 1
    EndWhile

    UpdateSexScene()
EndEvent

; Private

Function ConnectAndScanForDevices()
    Tele_Api.Connect()
    Tele_Api.ScanForDevices()
    ScanningForDevices = true
EndFunction

Function Disconnect() 
    Tele_Api.Close()
    ScanningForDevices = false
EndFunction

; Public

; Vibrate all specified devices for the given duration
; - speed (Percentage from 0=off to 100=full power)
; - duration_sec (Duratation in seconds. You can specify split seconds)
; - events (Vibrate devices that match the specified events)
Function Vibrate(Int speed, Float duration_sec)
    Tele_Api.Vibrate(speed, duration_sec)
    Trace("(Vibrate) speed='" + speed + "' duration='" + duration_sec + "' all")
EndFunction

; See Vibrate
Function VibrateEvents(Int speed, Float duration_sec, String[] events)
    Tele_Api.VibrateEvents(speed, duration_sec, events)
    Trace("(Vibrate) events speed='" + speed + " duration=" + duration_sec + " events=" + events)
EndFunction

; Stop all vibrators.
; - events (If events are specified, stop vibrators associated with the given event)
Function StopVibrate(String[] events = None)
    If events == None
        Tele_Api.Vibrate(0, 0.1)
        Trace("(Vibrate) stop all")
    Else
        Tele_Api.VibrateEvents(0, 0.1, events)
        Trace("(Vibrate) stop events=" + events)
    EndIf
EndFunction

; Logging

Function Notify(string msg)
    Debug.Notification("[Tele] " + msg)
EndFunction

Function Trace(string msg, Int level = 0)
    Debug.Trace("[Tele] " + msg, level)
EndFunction

Function LogError(string msg)
    Notify(msg)
    Trace(msg, 2)
EndFunction

Function LogConnection(string msg)
    Trace(msg)
    If LogDeviceConnects
        Notify(msg)
    EndIf
EndFunction

Function LogEvent(string msg)
    Trace(msg + " LogDeviceEvents " + LogDeviceEvents)
    If LogDeviceEvents
        Notify(msg)
    EndIf
EndFunction

Function LogDebug(string msg)
    Trace(msg)
    If LogDebugEvents
        Notify(msg)
    EndIf
EndFunction

Bool property Devious_VibrateEffect
    Function Set(Bool enable)
        If enable
            RegisterForModEvent("DeviceVibrateEffectStart", "OnVibrateEffectStart")
            RegisterForModEvent("DeviceVibrateEffectStop", "OnVibrateEffectStop")
        Else
            UnregisterForModEvent("DeviceVibrateEffectStart")
            UnregisterForModEvent("DeviceVibrateEffectStop")
        EndIf
    EndFunction
endProperty

Event OnDeviceActorOrgasm(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(10, 100), Utility.RandomFloat(5.0, 20.0) )
    LogDebug("DD OnDeviceActorOrgasm")
EndEvent

Event OnDeviceEdgedActor(string eventName, string strArg, float numArg, Form sender)
	Tele_Api.Vibrate( Utility.RandomInt(1, 20), Utility.RandomFloat(3.0, 8.0) )
    LogDebug("DD OnDeviceEdgedActor")
EndEvent

Event OnVibrateEffectStart(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(100, 120)
	LogDebug("DD VibrateStart " + eventName)
EndEvent

Event OnVibrateEffectStop(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
    LogDebug("DD VibrateStop")
EndEvent

Bool property Sexlab_Animation
    Function set(Bool enable)
        If enable
            RegisterForModEvent("HookAnimationStart", "OnSexlabAnimationStart")
            RegisterForModEvent("HookAnimationEnd", "OnSexlabAnimationEnd")
        Else
            UnregisterForModEvent("HookAnimationStart")
            UnregisterForModEvent("HookAnimationEnd")
        EndIf
    EndFunction
endProperty

Bool property Sexlab_ActorOrgasm
    Function set(Bool enable)
        If enable
            RegisterForModEvent("DeviceActorOrgasm", "OnDeviceActorOrgasm")
        Else
            UnregisterForModEvent("DeviceActorOrgasm")
        EndIf
    EndFunction
endProperty

Bool property Sexlab_ActorEdge
    Function set(Bool enable)
        If enable
            RegisterForModEvent("DeviceEdgedActor", "OnDeviceEdgedActor")
        Else
            UnregisterForModEvent("DeviceEdgedActor")
        EndIf
    EndFunction
endProperty

Event OnSexlabAnimationStart(int _, bool hasPlayer)
	If !hasPlayer
		 LogDebug("Animation on Non-Player")
		 return
	EndIf
	StartSexScene()
EndEvent

Event OnSexlabAnimationEnd(int _, bool hasPlayer)
	If !hasPlayer
        LogDebug("Animation on Non-Player")
		 return
	EndIf
	StopSexScene()
EndEvent

Bool property Toys_VibrateEffect
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysPulsate", "OnToysPulsate") ; Pulsate Effect has started. Duration is random lasting from approx. 12 to 35 seconds
        Else
            UnregisterForModEvent("ToysPulsate")
        EndIf
    EndFunction
endProperty

Bool property Toys_Animation
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysStartLove", "OnToysSceneStart") ; Sex scene starts
            RegisterForModEvent("ToysLoveSceneEnd", "OnToysSceneEnd") ; Sex scene ends
        Else
            UnregisterForModEvent("ToysStartLove")
            UnregisterForModEvent("ToysLoveSceneEnd")
        EndIf
    EndFunction
endProperty

Bool property Toys_OtherEvents
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
endProperty

Bool property Toys_Denial
    Function set(Bool enable)
        If enable
            RegisterForModEvent("ToysDenied", "OnToysDenied") ; An individuall squirt has been denied
        Else
            UnregisterForModEvent("ToysDenied")
        EndIf
    EndFunction
endProperty

Event OnToysPulsate(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate( Utility.RandomInt(1, 100), 5 )
	LogDebug("ToysPulsate")
EndEvent

Event OnToysFondleStart(string eventName, string argString, float argNum, form sender) 
	Tele_Api.Vibrate(10, 30)
	LogDebug("ToysFondleStart")
EndEvent

Event OnToysFondleEnd(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
	LogDebug("ToysFondleEnd")
EndEvent

Event OnToysSquirt(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(100, 12.0)
	LogDebug("ToysSquirt")
EndEvent

Event OnToysSceneStart(string eventName, string argString, float argNum, form sender)
	StartSexScene()
	LogDebug("ToysSceneStart")
EndEvent

Event OnToysSceneEnd(string eventName, string argString, float argNum, form sender)
	StopSexScene()
	LogDebug("OnToysSceneEnd")
EndEvent

Event OnToysClimax(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(80, 5)
	LogDebug("OnToysClimax")
EndEvent

Event OnToysClimaxSimultaneous(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(100, 8)
	LogDebug("OnToysClimaxSimultaneous")
EndEvent

Event OnToysDenied(string eventName, string argString, float argNum, form sender)
	Tele_Api.Vibrate(0, 0.1)
	LogDebug("OnToysDenied")
EndEvent

; Event OnToysVaginalPenetration(string eventName, string argString, float argNum, form sender)
; 	LogDebug("OnToysVaginalPenetration")
; EndEvent

; Event OnToysAnalPenetration(string eventName, string argString, float argNum, form sender)
; 	LogDebug("OnToysAnalPenetration")
; EndEvent

; Event OnToysOralPenetration(string eventName, string argString, float argNum, form sender)
; 	LogDebug("OnToysOralPenetration")
; EndEvent

; Event OnToysCaressed(string eventName, string argString, float argNum, form sender)
; 	LogDebug("OnToysCaressed")
; EndEvent

; --------------- TODO DEPRACTE -------------

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

