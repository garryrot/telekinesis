Scriptname TK_Test_SpellVibrateEffect extends ActiveMagicEffect  
Event OnEffectStart(Actor target, Actor caster)
	Log("Vibrate Spell");
	int vibrated = TK_Telekinesis.TK_StartVibrateAll(1.0)
EndEvent

Event OnEffectFinish(Actor akTarget, Actor akCaster)
    Log("Spell Stop")
  int vibrated = TK_Telekinesis.TK_StartVibrateAll(0)
endEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction