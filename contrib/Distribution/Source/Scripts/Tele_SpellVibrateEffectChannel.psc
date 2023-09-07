Scriptname Tele_SpellVibrateEffectChannel extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

Int _VibrateEffectHandle = -1
Event OnEffectStart(Actor target, Actor caster)
	_VibrateEffectHandle = TeleDevices.Vibrate(Math.Floor(GetMagnitude()), -1)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
	TeleDevices.StopHandle(_VibrateEffectHandle)
EndEvent
