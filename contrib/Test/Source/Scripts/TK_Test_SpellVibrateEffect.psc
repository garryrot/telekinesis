Scriptname TK_Test_SpellVibrateEffect extends ActiveMagicEffect  
Event OnEffectStart(Actor target, Actor caster)
	Log("Vibrate Spell");
	Bool vibrated = Tele.VibrateAll(100)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
    Log("Spell Stop")
	Bool vibrated = Tele.VibrateAll(0)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction