Scriptname Tele_SpellVibrateEffectChannel extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	TeleDevices.Vibrate(Math.Floor(GetMagnitude()), -1)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
	TeleDevices.VibrateStopAll()
EndEvent
