Scriptname Tele_SpellEffectStop extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	Tele_Api.StopAll()
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
EndEvent