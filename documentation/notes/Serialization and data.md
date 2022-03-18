# About deluge data structures

I think sounds are different for each SoundType.  
Probably there are common stuff, I do not want to reproduce the real
object hierarchy the deluge uses, but:  
    - i suspect they have a base class "Sound" (I guess they use C++ and classic object oriented approch)  
    - RingMod, Subtractive and Fm are derived classes from Sound  
This is why some parameter are available and other not depending of the patch.  

Example:  
 - ringmod SYNT177.XML
 - subtractive SYNT179.XML
 - fm SYNT176.XML

# Ideas - Patch instead of write everything?
I think for writing, I must implement some kind of patching to be sure to never loose any informations even if
DKE does not support them. The idea is to have an in memory representation of the XML file, then modify this in memory representation to write it later. This sounds complex, but this way even if DKE does not support a field for example, the field will simply never be modified by DKE and the data in the file will never be modified as well.  
This sounds complex but I think this is the only way to avoid loosing patch information.
