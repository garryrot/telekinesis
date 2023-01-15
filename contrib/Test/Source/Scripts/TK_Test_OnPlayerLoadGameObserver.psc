Scriptname TK_Test_OnPlayerLoadGameObserver extends ReferenceAlias

Actor property Player auto
ReferenceAlias property PlayerRef auto

Event OnPlayerLoadGame()
	Log("OnPlayerLoadGame")
	Log("TK_ScanForDevices...")
	TK_Telekinesis.TK_ScanForDevices();
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction

Event OnHit(ObjectReference aggressor, Form source, Projectile projectile, bool powerAttack, bool sneakAttack, bool bashAttack, bool blocked)
	Actor actorRef = PlayerRef.GetActorRef()
	Float strength = 1 - (actorRef.GetActorValue("Health") / actorRef.GetBaseActorValue("Health"))
	Float duration = strength * 2
	if (duration < 0.5)
		duration = 0.5
	endif
	TK_Telekinesis.TK_VibrateAllFor(strength, duration)
EndEvent