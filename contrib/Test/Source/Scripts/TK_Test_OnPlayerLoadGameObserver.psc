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
	Float health = actorRef.GetActorValue("Health")
	Float maxHealth = actorRef.GetBaseActorValue("Health")
	Float strength = 1 - (health / maxHealth)
	Log("OnHit " + strength + " health: " + health + " maxHealth: " + maxHealth)
	TK_Telekinesis.TK_VibrateAllFor(strength, 1.5)
EndEvent