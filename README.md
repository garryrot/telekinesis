# Telekinesis (Bluetooth Toy Control for Papyrus) V0.3.0

**In a Nutshell**

This is my humble attempt at creating Papyrus bindings for the famous buttplug.io toy control library. This SKSE64 plugin allows modders to control bluetooth toys (Vibrators, etc.) from within Papyrus scripts. It provides several new Papyrus functions that let you Scan for local devices and then control/vibrate them. This does not provide any playable content on its own, (unless you run the Sample ESP). This is pretty much a big experimental prototype and still work in progress. The API I expose right now can and will change.

## Features
 * Very fast and reactive native implementation
 * Toy control directly from within Papyrus
 * No dependency on external processes, just install the ESP

## Usage

>:warning: **EARLY TEST PROTOTYPE: THIS API WILL CHANGE** :warning:

### 1. Install

1. Install the Mod
2. Move the Main .psc file `Tele.psc` into a path where your Papyrus Compiler can find it. For Creation Kit its probably something like `data/source/scripts` but I don't get its folder structure at all tbh.
3. Connect your Toy via Bluetooth
4. Open up Creation Kit and [Use the Papyrus Functions!](https://github.com/garryrot/telekinesis/blob/master/contrib/Distribution/Source/Scripts/Tele.psc)
4. Alternatively: Install The Sample ESP (Telekinesis.Sample.7z), which demonstrates the use of those Papyrus Functions In-Game

**Depdendencies**: `SKSE64`, `Skyrim SE`, `Address Library for SKSE Plugins`
**Incompatibilities**:  Other mods that control Intiface/Or Buttplug.io in one way or the other might cause undefined behavior.

### 2. Start

Start Telekinesis and scan for devices. This must be done once on every game startup (actually
once for every game process). You most likely want to do this `OnInit` and `OnPlayerLoadGame`.

```cs
Actor property Player auto
Event OnInit()
    Tele.ScanForDevices()
    RegisterForUpdate(5) // for displaying updates (see section 3)
EndEvent
```

If Telekinesis wasn't started, the other functions will not have any effect.

### 3. Control Devices 

Call `VibrateAll(speed)` to vibrate all devices.

```cs
int vibrated = Tele.VibrateAll(1.0) // speed can be any float from 0 to (1.0=full speed)
Debug.Notification( "Vibrating" + vibrated + " device(s)..." )
```

Call `VibrateAll(0)` to stop all devices

```cs
Util.Wait(5);
int stopped = Tele.VibrateAll(0) // 0 = stop vibrating
Debug.Notification( "Stopping" + stopped + " device(s)..." )
```

If no devices are connected or Telekinesis was not started, this will simply do nothing.

#### 4. Monitoring Connections

You can poll `PollEvents` to see if any device connected or disconnected. This
will return a message or the default string `""` (if nothing happened).

```cs
Event OnUpdate()
    String evt = Tele.PollEvents()
    If (evt != "")
        Debug.Notification(evt) // If it says "Device XY connected" you are ready to go
    EndIf
EndEvent
```

### 5. Shutting Down

At one point the user will close the game or load a different safe. If possible, you should call `StopAll`
to stop all devices. In the worst case (i.e. if the user kills the game process while a vibration is running)
the `Stop Event` will be lost and the vibrating toys might need to be turned off manually.

I don't know if there is any reliable event or hook to do this. Please tell me if you do.

```cs
Tele.StopAll() // stop all devices
```

`Close` will free up the associated memory resources after everything else is done.

```cs
Tele.Close() // destroy the connection 
```

## Demo

Showcase of the Sample/Test-Mod `Telekinesis.SampleProject.7z`

- Triggers vibration based on DD `OnVibrateEffectStart` and `OnVibrateEffectStart` events
- Gives the player a "Telekinesis Vibrate Toy" spell that causes a vibration effect

*You can already run this if you want, but its just a proof of concept and lacks any settings (and fun)*

[![Watch the Video](https://i.imgur.com/QiG6p2y.jpg)](https://www.youtube.com/watch?v=_EoiLqY_6_Q)

## Caveats & Known Issues

 * Only BluetoothLE is activated right now: [List of toys that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)
 * Only tested Skyrim SE (v1.5.97.0)
    * If it works on different versions, please tell me!
 * If you close or reload the game during a vibration event it may not stop until you turn off your device manually

(*) More connection-managers be activated in later versions 

## Troubleshooting

### Devices don't connect

First, make sure that your device is couple correctly and works with Buttplug.io. You can use the [Intiface Central Desktop App](https://intiface.com/central) to test your device and verify that it actually works with Buttplug before proceeding.

### Bug Reports

If anything fails or behaves in an unexpected way, include the Papyrus logs `Pyprus.0.log` and the Logs of this plugin (`Telekinesis.SKSE.log` and `Telekinesis.Plug.log`)

* You will find them in `%USERPROFILE%/My Games/Sykrim Special Edition/SKSE/...`
* If you can reproduce the issue, adapt the debug level by changing `Telekinesis.yaml` (in `Data/SKSE/Plugins` next to your `Telekinesis.dll`) and set everything to `trace`.


## Why yet another bluetooth control?

First of all, because I can :3

Right now, this is more of a proof of concept, and far from a stable production state.

There several other toy control projects you can use. The most complete and feature-rich approach
is [GIFT](https://github.com/MinLL/GameInterfaceForToys), which processes events from the Papyrus Logs and controls
external, and also supports a bigger variety of outputs (not only buttplug.io but also E-Stim, Chaster, etc.) and
different Games.

Right now, if you are not a mod developer with very specific speed and integration requirements, you most likely 
want to use GIFT.

### When Telekinesis?

Telekinesis tries a completely different approach, trying to fit a different niche by providing native Papyrus functions.

**Upsides**:

- Fast reaction time, because everything happens in-process
- Gives device control directly to Papyrus, which should make the creation of interactive interactions really easy
- Will only add one mod dependency (this mod)

**Downsides**:

- With tighter integration and without an external supervisor process, there is one less failsafe. For example, if the User kill Skyrim with alt+f4, and then you have to turn off your device manually
- Mods might not expose Papyrus events for all the things that show up the logs
- External GUI applications might be easier to use than MCM Menus

## License

This if free software. If you want to change this, redistribute it, or integrate it into your mod, you are free to whatever you like, as long as it is permitted by the  [Apache License](LICENSE)

## Changelog

### 0.3.0

- Fix AE support
- Support vibrating for a certain duration
- Actually link against the correct rust library, so the stutter fix from 0.2.0 is now correctly included
- Reworked/Broke entire API

### 0.2.0

- Support message queuing to reduce mini lags
- More consistent naming of API functions

### 0.1.0

- Initial Version

