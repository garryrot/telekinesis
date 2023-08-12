Scriptname Tele_SpellEffectStop extends ActiveMagicEffect

Tele_Devices Property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	Tele_Api.StopAll()
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction
