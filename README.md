# Telekinesis

This is my humble attempt at creating Papyrus bindings for the famous buttplug.io toy control. 

**In a Nutshell:** This resource allows modders to control bluetooth toys (Vibrators, etc.) from within papyrus functions in Skyrim. This does not provide actual game content. If you are not a Skyrim Mod Developer it will do nothing for you, unless you also run a Mod that is making use of it.

## Features

There have been several efforts to control toys from within Skyrim Mods in the past, most of which use the Papyrus log to control Vibration events. This projects tries to solve the problem from ground up by extending Papyrus:

 * Very fast and reactive native implementation
 * Device control directly from within Papyrus Scripts scripts
 * No dependency on external processes or applications

## Demo

Showcase of the sample Mod (Triggers vibration based on DD `OnVibrateEffectStart` and `OnVibrateEffectStart` events)

*You can already run this mod if you want, but its just a proof of concept and lacks any settings (and fun)*

## Caveats & Known Issues

 * Only BluetoothLE Vibrators are activated right now (*)
    - List of devices that might work: [IoST Index of Bluetooth Vibrators with Buttplug IO support](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)

 * I only tested this on SE (v1.5.97.0) with the newest version of 

 * If you close or reload the game during a vibration event it may not stop until you turn off your device manually

(*) More connection-managers be activated in later versions 


## Installation (Mod Developers)

 - Download the latest Telekinesis.Version.7z and install it with your mod manager
 - This is also a dependency for your mod users

**Depdendencies**

 - SKSE64
 - Skyrim SE/VR/AE
 - Address Library


## API

ATTENTION: This is my first attempt at a prototype and is very likely to change in fundamental ways. 

#### TK_Telekinesis.psc

```cs
scriptName TK_Telekinesis hidden

// Sets up a new connection and starts scanning for devices. 
// This will automatically connect to every single bluetooth
// toy Buttplug.io knows about, and that is turned on and coupled to
// you PCs bluetooth connection. Right now
// the scanning will continue indefinitely, so new devices might be added at
// any point in time automatically
bool function TK_ScanForDevices() global native

// Vibrate all devices that are currently connected.
// Speed is any float between 0.0(=off) and 1.0 (=full power)
// TK_StartVibrateAll( 0 ) should also be used for stopping the vibration,
// as it provides a smoother experience than TK_StopVibrateAll
// TODO: Rename to Tk_SetVibrationSpeed
int function TK_StartVibrateAll(Float speed) global native

// Immediately stops all connected devices
// This should be used for shutdown, before calling Tk_Close.
// TODO: Rename to Tk_StopAll
int function TK_StopVibrateAll() global native

// Returns a stream of messages that describe
// the status devices of devices, and whether they are
// connecting or disconnecting.
//
// - Returns a new Event every time this is called.
//  EXAMPLE:
//  * "" (Empty string, if no new messages available)  
//  * "Device YOUR_DEVICE connected" (This device is connected and will be controlled)
//  * "Device YOUR_DEVICE disconnected" (This device should no longer get vibrated)
//  
// DISCLAIMER: The implementation is a really shitty hackjob right now and will
// only return one event at a time (and even drop some). When multiple
// Mods consume this, they will steal each others events
string function Tk_AwaitNextEvent() global native

// Close the connection and dispose all structures. Until you run
// TK_ScanForDevices again to set up a new connection no controls
// will have any effect after calling this
bool function Tk_Close() global native
```

## Troubleshooting

If anything fails or behaves in an unexpected way, check the error log in th SKSE logs of this plugin `Telekinesis.SKSE.log` and `Telekinesis.Plug.log` 

You can adapt logging level by editing `Telekinesis.yaml` (in `Data/SKSE/Plugins` next to your `Telekinesis.dll`)

You can probably find them in `%USERPROFILE%/My Games/Sykrim Special Edition/SKSE/...`


## Changelog

### 0.1.0

- Initial Version


## License

This if free software you can use it however you like under the Terms of the Apache2.0 Open Source License, see [LICENSE](LICENSE).