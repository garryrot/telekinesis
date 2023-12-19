# Feature Backlock

## Bugfixes / Improvements

- Improve device status
    + Keep track of device status in tk struct
        + Erroring (after each error, propagate error from player)

    + Loop patterns for duration

    + Unique device names

    + Logging
        + Debug-Level: Print settings and tags on each command
        + Remove Warning telekinesis_plug::pattern: Speed 93 overrids provided speed 93

    + Events
        + Do not log "Stop" events


- Get rid of RegisterForUpdate (see)[https://wiki.bethesda.net/wiki/creationkit/Skyrim/RegisterForUpdate_-_Form/]

## Multi-Motor Support

- Instead of each device, each motor is a distinct endpoint
    - Associated Events/TAgs
    - Enabled/Disabled

## G.I.F.T Support

1. Create PP with custom patterns 

## Strokers

**TODO:**

1. Implement stroker function analog to Vibrate
    - `Int LinearPattern(String pattern, Float duration)`
    - `Int Linear(Int pos (0-100), Float duration)`, is this useful?
2. Add new backend action to propagate linear commands to bp io
3. Add support for linear patterns to pattern reader
4. Add at least one linear pattern
5. Add sexlab support for strokers
    - Stroker movement Enable/Disable
    - Filter All/Match Events
    - Pattern selection 
6. Find someone that can actually test it

## OStim Integration

**Implementation hint from OStim Discord**

There are several events you can listen for in OStim.

For starters, you will want to listen for the ostim_start event. This is what tells you when an OStim scene starts. You can do any sort of initialisations that you need on this event.

Afterwards, you want to know when a scene changes, so you know which animation is currently being played. For this you have the ostim_scenechanged event, which sends the ID of the scene being played.

Now, with the ID of the scene being played, you can call the functions in OMetadata.psc to check the tags, actions and other info about the animation, so you can do all the magic you want. OMetadata is decently documented. I will also leave a link below with common actions used in OStim scenes.

When the OStim scene ends, you have the ostim_end event telling you so.

https://github.com/VersuchDrei/OStimNG/blob/main/data/Scripts/Source/OMetadata.psc

https://github.com/VersuchDrei/OStimNG/tree/main/data/SKSE/Plugins/OStim/actions

To get arousal levels, you can use functions from the OActor.psc script: https://github.com/VersuchDrei/OStimNG/blob/main/data/Scripts/Source/OActor.psc

If you want to see OStim events being used as a practical example, you can check this script from OCum: https://github.com/Aietos/OCum-Ascended/blob/master/Source/Scripts/OCumMaleScript.psc
