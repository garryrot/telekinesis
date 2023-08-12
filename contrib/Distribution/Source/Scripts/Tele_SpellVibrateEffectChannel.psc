Scriptname Tele_SpellVibrateEffectChannel extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	Bool vibrated = TeleDevices.Vibrate(Math.Floor(GetMagnitude()), 120)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
	Bool vibrated = TeleDevices.Vibrate(0, 0.1)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction
