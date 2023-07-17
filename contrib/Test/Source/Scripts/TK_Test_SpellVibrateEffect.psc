Scriptname TK_Test_SpellVibrateEffect extends ActiveMagicEffect

Tele_Devices property TeleDevices Auto

Event OnEffectStart(Actor target, Actor caster)
	String[] names = TeleDevices.GetUsedDevices()
	Bool vibrated = Tele.Vibrate(100, 30, names)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
	String[] names = TeleDevices.GetUsedDevices()
	Bool vibrated = Tele.Vibrate(0, 0.1, names)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction
