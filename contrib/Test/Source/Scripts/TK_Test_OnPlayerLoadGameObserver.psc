Scriptname TK_Test_OnPlayerLoadGameObserver extends ReferenceAlias

Actor property Player auto
ReferenceAlias property PlayerRef auto

Event OnHit(ObjectReference aggressor, Form source, Projectile projectile, bool powerAttack, bool sneakAttack, bool bashAttack, bool blocked)
	Actor actorRef = PlayerRef.GetActorRef()
	Float lostHealth = 1 - (actorRef.GetActorValue("Health") / actorRef.GetBaseActorValue("Health"))
	Int strength = Math.Floor(100 * lostHealth);
	Float duration = strength / 50
	If (duration < 0.5)
		duration = 0.5
	EndIf
	Tele.Vibrate(strength, duration)
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[Tele] " + textToPrint)
	Debug.Notification("[Tele] " + textToPrint)
EndFunction