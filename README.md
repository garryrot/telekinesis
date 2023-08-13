# Telekinesis (Bluetooth Toy Control for Skyrim) 1.0.0 Beta

## Installation

1. Install `Telekinesis.7z` with a mod manager

**Depdendencies**: `SKSE64`, `Skyrim SE`, `SkyUI`, `Address Library for SKSE Plugins`
**Incompatibilities**:  Other mods that control Intiface/Or Buttplug.io in one way or the other might cause undefined behavior. Note that there is a compatibility option for G.I.F.T. so you can run it.

## Migration

Migrating from the early alpha versions is not supported, start a new game, or try to fix on your own.

- Uninstall `TelekinesisTest.esp` and delete it forever (it won't be needed again)

## Usage

1. Make sure that your bluetooth devices are coupled in your system control and connected
2. The dll will constantly scan for Bluetooth LE devices and connect them when they are available. Connected devices will show up in your Notifications
3. Open the MCM, go to Page `Devices` and enable the devices you want to use (you only need to do this once, this choice is persisted between different save games)
4. Use spells in `Debug` to test device vibrations (Don't worry, deselecting the option will remove them from your character again)
5. Remember the Emergency Stop hotkey (default `DEL`) in case anything goes wrong

## Caveats & Known Issues

 * Only BluetoothLE is activated right now: [List of toys that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)
 * Tested on Skyrim SE (v1.5.97.0) and AE (1.6.640.0)

## Screenshots

<img src="doc/scr1.png" width="500"/>
<img src="doc/scr2.png" width="500"/>
<img src="doc/scr3.png" width="500"/>
<img src="doc/scr4.png" width="500"/>
<img src="doc/scr5.png" width="500"/>

## Troubleshooting

### Devices don't connect

Please check that:

1. First, make sure that your device is couple correctly
2. Your device has enough battery
3. Your device is supported by buttplug.io, see [List of toys that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)
4. Test it with [Intiface Central Desktop App](https://intiface.com/central), if a vibrator works in that app, and not in this plugin, its an issue with the mod.

### Devices connects but doesn't vibrate

1. Make sure that your device is enabled in Page `Devices`
2. Make sure it has full battery (with low battery it might still be able to connect but not move)

### Bug Reports

If anything fails or behaves in an unexpected way, include the Papyrus logs `Pyprus.0.log` and the Logs of this plugin (`Telekinesis.SKSE.log` and `Telekinesis.Plug.log`)

* You will find them in `%USERPROFILE%/My Games/Sykrim Special Edition/SKSE/...`
* If you can reproduce the issue, adapt the debug level by changing `Telekinesis.yaml` (in `Data/SKSE/Plugins` next to your `Telekinesis.dll`) and set everything to `trace`.
## Why yet another bluetooth control?


## License

This if free software. If you want to change this, redistribute it, or integrate it into your mod, you are free to whatever you like, as long as it is permitted by the [Apache License](LICENSE)


## Changelog

### 1.0.0

- Complete rework of everything
- Mod now comes with an MCM and lots of mod integration

### 0.3.0

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

### 0.2.0

- Support message queuing to reduce mini lags
- More consistent naming of API functions

### 0.1.0

- Initial Version
