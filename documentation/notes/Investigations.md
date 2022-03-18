# Investigations

This file contains my notes for my ongoing investigations about the Deluge's file format.

## notes about specific values
 - sound.modulator1.toModulator2: I don't understand what is this parameter. I can't find anything in the Deluge's manual. 
 => Je crois que c'est une option en mode FM, on peut envoyer le modulateur 1 soit dans le carrier soit dans modulateur 2.

It seems to be a OnOff value.  
 - timeStretchEnable: peut etre 0 ou 1, et 1 correspond a Linked, 0 a Independent  [Done] -> PitchSpeed
 - timeStretchAmount: -48 a +48 , display label: SPEEd   [Done] -> TimeStretchAmount
 - defaultParams: the values in this part of the XML files are the values that can be automated.  
The reason why it's called "defaultParams" is because thoses values are used when there are no automation. 
I suppose they are storing all the values in a big array of integer, but this is not important to me as I do not want to edit songs.  

About the pulsewidth, the Deluge's manual says there is no pulse width in FM mode.

All parameter modulation are stored in a "patchCables" structure.
Each patchCable contains:
 - source (string)
 - destination (string)
 - amount (HexU50)
 I think I should just accept any string for source and destination, and let the user decide how to use that.

 arpeggiator.mode =>
 Off, Up, Down, Both, Rand