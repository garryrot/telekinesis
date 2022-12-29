# Telekinesis (Bluetooth Toy Control for Papyrus)

This is my humble attempt at creating Papyrus bindings for the famous buttplug.io toy control framework. 

**In a Nutshell:** This SKSE64 plugin allowes modders to control bluetooth toys (Vibrators, etc.) from within Papyrus scripts. This does not provide actual game content. If you are not a Skyrim Mod Developer it will do nothing for you, unless you also run a Mod that is making use of it.

## Features
 * Very fast and reactive native implementation
 * Toy control directly from within Papyrus
 * No dependency on external processes, just install the ESP

## Usage

>:warning: **EARLY TEST PROTOTYPE: THIS API WILL CHANGE** :warning:

### 1. Install Telekinesis

**Depdendencies**

 - SKSE64
 - Skyrim SE/VR/AE
 - Address Library

### 2 Connect and Scan

Start Telekinesis and scan for devices. This must be done once on every game startup (actually
once for every game process). You most likely want to do this `OnInit` and `OnPlayerLoadGame`.

```cs
Actor property Player auto
Event OnInit()
    TK_Telekinesis.TK_ScanForDevices()
    RegisterForUpdate(5) // for displaying updates (see section 3)
EndEvent
```

If Telekinesis wasn't started, the other functions will not have any effect.

### 2. Device Control

Call `TK_StartVibrateAll(speed)` to vibrate all devices.

```cs
int vibrated = TK_Telekinesis.TK_StartVibrateAll(1.0) // speed can be any float from 0 to (1.0=full speed)
Debug.Notification( "Vibrating" + vibrated + " device(s)..." )
```

Call `TK_StartVibrateAll(0)` to stop all devices

```cs
Util.Wait(5);
int stopped = TK_Telekinesis.TK_StartVibrateAll(0) // 0 = stop vibrating
Debug.Notification( "Stopping" + stopped + " device(s)..." )
```


If no devices are connected or the connection was not established, this will simply do nothing.

#### 3. Monitoring Connected Devices

You can poll `Tk_AwaitNextEvent` to see if any device connected or disconnected. This
will return a message or the default string `""` (if nothing happened).

```cs
Event OnUpdate()
    String evt = TK_Telekinesis.Tk_AwaitNextEvent()
    If (evt != "")
        Debug.Notification(evt) // If it says "Device XY connected" you are ready to go
    EndIf
EndEvent
```


### 4. Shutting Down

At one point the user will close the game or load a different safe. If possible, you should
call `TK_StopVibrateAll` to stop all devices.

In the worst case (i.e. if the user kills the game process while a vibration is running)
the `Stop Event` will be lost and the vibrating toys might need to be turned off manually.

I don't know if there is any reliable event or hook to do this. Please tell me, if you know.

```cs
TK_Telekinesis.TK_StopVibrateAll() // stop all devices
TK_Telekinesis.TK_Close() // destroy the connection 
```

`TK_Close` will free up the associated memory resources. If the process dies, this happens
anyways. After this, you can call `TK_ScanAndConnect` again to start all over again.

## Demo

Showcase of the sample Mod (Triggers vibration based on DD `OnVibrateEffectStart` and `OnVibrateEffectStart` events)

*You can already run this mod if you want, but its just a proof of concept and lacks any settings (and fun)*

## Caveats & Known Issues

 * Only BluetoothLE Vibrators are activated right now (*)
    - [List of Devices that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)
 * I only tested this on SE (v1.5.97.0)
 * If you close or reload the game during a vibration event it may not stop until you turn off your device manually

(*) More connection-managers be activated in later versions 


## API

[See TK_Telekinesis.psc]()

#### 

```cs
scriptName TK_Telekinesis hidden
bool function TK_ScanForDevices() global native
int function TK_StartVibrateAll(Float speed) global nativ
int function TK_StopVibrateAll() global native
string function Tk_AwaitNextEvent() global native


bool function Tk_Close() global native
```

## Why...

### 

### This has been done before, why do it again?

There have been several efforts to control toys with Skyrim in the past. Most of them uread Papyrus log to control Vibration events. This projects tries a different approach to solve the problem from ground up by extending Papyrus:

## Troubleshooting

If anything fails or behaves in an unexpected way, check the error log in th SKSE logs of this plugin `Telekinesis.SKSE.log` and `Telekinesis.Plug.log` 

You can adapt logging level by editing `Telekinesis.yaml` (in `Data/SKSE/Plugins` next to your `Telekinesis.dll`)

You can probably find them in `%USERPROFILE%/My Games/Sykrim Special Edition/SKSE/...`


## Changelog

### 0.1.0

- Initial Version


## License

This if free software you can use it however you like under the Terms of the Apache2.0 Open Source License, see [LICENSE](LICENSE).