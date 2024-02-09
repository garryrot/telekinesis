# Set up Body Parts

Lets assume a `VibrateEffectStart` occurs on the player character, based on the following settings, these scnarios can happen:

- In the first case `All` filter was selected. This will select all devices that can vibrate, even if they have no tags associated.
- In the second case the in-game character wears a vibrating nipple piercing and one Vibrator is tagged with `Nipple`
- In the third case the in-game charecter wears a vibrating anal plug and a vibrating nipple piercing, this will result in both devices being vibrated.


| **Device Filter**   | In-Game Devices       |       IRL Plug, Body Parts=Anal,Vaginal | IRL Nipple Vibrator, Body Parts=Nipple | IRL Device 3 Body Parts=       |
|-------------------------|---------------------  |--------------------------------------|----------------------           |--------------------------------|
| All          |             *                    | **Vibrates**                         | **Vibrates**                    |         **Vibrates**
| Match Body | Vibrating Plug                   | **Vibrates**                         |       ---                       |            --- 
| Match Body | Vibrating Plug, Nipple Piercing  | **Vibrates**                         | **Vibrates**                    |            --- 
