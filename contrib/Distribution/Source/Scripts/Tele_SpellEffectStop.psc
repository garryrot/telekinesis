Scriptname Tele_SpellEffectStop extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

; TODO This should be removed

Event OnEffectStart(Actor target, Actor caster)
	TeleDevices.EmergencyStop()
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
EndEvent