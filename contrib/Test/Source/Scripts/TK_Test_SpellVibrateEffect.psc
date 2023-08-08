Scriptname TK_Test_SpellVibrateEffect extends ActiveMagicEffect

Tele_Devices property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	Bool vibrated = Tele.Vibrate(100, 30)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
	Bool vibrated = Tele.Vibrate(0, 0.1)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction
