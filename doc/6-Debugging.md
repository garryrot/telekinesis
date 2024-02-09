# Debugging & In-Game Messages


### 1. In-Game Debugging Tools

Varius tools and Settings for debugging issues or just toying around...

- **Spells**: The player character learns a set of useful debug spells to test the device vibration, or stop vibrations. The spells will vibrate the toy at various strength (low=1, mid=10, full=100). Don't worry, these spells will disappear from the player if you unselect it.

- **Logging**: This controls which types of message are shown as an in-game notification (top left). 
  - **Devices connects**: 'Device XY has connected' etc. events are shown
  - **Device events**: 'N Device(s) have vibrated at M%' events are shown
  - **Other messages**: For debugging

<img src="scr5.png" width="450"/>


### Logs

To diagnose problems, please always include the following log files:

##### Telekinesis.log

This is a log in `C:\Users\YOUR_USER\Documents\My Games\Skyrim Special Edition\SKSE\Telekinesis.log` that contains the logs of the native library.

If you have device errors, or issues with funscript playback, this log might contain information on the root cause.

