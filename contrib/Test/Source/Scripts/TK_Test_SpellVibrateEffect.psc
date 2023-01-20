Scriptname TK_Test_SpellVibrateEffect extends ActiveMagicEffect  
Event OnEffectStart(Actor target, Actor caster)
	Log("Vibrate Spell");
	Bool vibrated = Tele.VibrateAll(1.0)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
    Log("Spell Stop")
	Bool vibrated = Tele.VibrateAll(0)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction