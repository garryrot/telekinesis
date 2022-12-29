Scriptname TK_Test_OnPlayerLoadGameObserver extends ReferenceAlias

Actor property Player auto

Event OnPlayerLoadGame()
	Log("OnPlayerLoadGame")
	Log("TK_ScanForDevices...")
	TK_Telekinesis.TK_ScanForDevices();
EndEvent

Function Log(string textToPrint)
	Debug.Trace("[TK] " + textToPrint)
	Debug.Notification("[TK] " + textToPrint)
EndFunction