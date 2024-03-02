## 1.2.1

Fix issue with PlayerRef that cause Telekinesis to not load in certain scenarios

## 1.2.0

**IMPORTANT**: All device settings have been reset to allow for multi-motor support. You need to re-enable all your devices after installing this patch.
	
## Features

* Add support for OStim
  * Controls the vibrators whenever a sexual scene runs
  * Support speed control based on in-game animation speed (the thing that is controlled with -/+) and/or player rousing (the bar)

* Support dynamic speed during funscript patterns
  * In the MCM, linear speed and arousal sync are no longer disabled when choosing `Funscript` or `Random Funscript`

* Support actuator (motor-specific) device control
  * If a device has multiple motors, each motor will now show up individually in the seettings, and can be assigned custom body parts

## Improvements / Fixes

* Fix an issue that sometimes caused vibrations to linger after arousal-controlled sex scenes

* Device errors are now displayed in-game (red) and will cause devices to go into error state

* Support setting speed of running task handles with new native call `Tele_Update` 

* Device connects/disconnects and actions are now dispatched as SKSE_Events to reduce script load: `Tele_Connected`, `Tele_ConnectionError`, `Tele_DeviceAdded`, `Tele_DeviceRemoved`, `Tele_DeviceActionStarted`, `Tele_DeviceActionDone`, `Tele_DeviceError`


## 1.1.0

- Migrating from Beta will reset your MCM settings

- Add support for funscript patterns
  * Only works with vibrator files `vibration.funscript` files for now
  * Other patterns are still being displayed

- Add support for events (device tags)
  * This allows associating devices with certain events that correlate to body parts (see manual)

- Improve integration for Sexlab, Devious Devices, Toys & Love:
  * Introduced a lot of new generic vibration options that are available for almost all of the vibration events
    * Strength can be regulated linearly or with a funscript pattern
    * Use random patterns
    * Support matching devices with events (body parts)

  * Devious Devices
    * Uses actual DD vibration strength (device vibrated strongly, very strongly etc.) instead of a random speed value.
    * Tag/Event support to match equipped dd stimulation devices with body parts (Nipple, Anal, Vaginal)

  * Sexlab
    * Match devices with animation tags
    * Control Strength through sexlab arousal
    * Support for denial

  * Toys&Love
    * Match with animation tags
    * Control strength through rousing
    * Support Denial, Body Part Penetration and Fondling events

  * Skyrim Chain Beast
    * Support Gemmed Beast Vibrations (`SCB_VibeEvent`)
    * Disclaimer: Seems to not work with Chainbeasts v7.0.0, unless SCB_VibeEffect.psc is recompiled
      from source and the psx was replaced in Script folder

- Technical Improvements
  * Add support for simultaneous and overlapping vibration events and patterns. 
    * Previously every new device action aborted all running tasks
    * Technical requirement for long running patterns and to assure a seamless
      experience with mods that do a lof of different things at the same time.
    * Papyrus API had to be reworked to use task handles
  * WebSocket (Intiface) connection now works
    * This allows to use Intiface App as backend control instead of the default in-process backend

## 1.0.0

- Complete rework of everything
- Devious Devices Integration
- Toys & Love Integration
- Sexlab integration
- Add emergency stop hotkey

## 0.3.0

**Features**:

- Add `Tele.VibrateAllFor` to vibrate for a specific duration and then stop
- Reworked/broke entire API
    - Vibration speed is now value between 0 and 100
    - Shorter functions i.e. `Tele.VibrateAll` instead of `Tk_Telekinesis.Tk_VibrateAll`

**Fixes**:
- Now loads on AE (as intended)
- More stability/stutter fixes
    - Not a single possibly blocking call left in papyrus thread
    - Actually link against updated rust lib, so the fix from 0.2.0 is now correctly included

## 0.2.0

- Support message queuing to reduce mini lags
- More consistent naming of API functions

## 0.1.0

- Initial Version
