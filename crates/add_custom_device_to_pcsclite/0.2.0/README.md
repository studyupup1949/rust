> OpenSC relies on PCSClite driver, which reads a list (Info.plist) that contains a pair of VID/PID of supported readers.

When apps like gpg don't detect your custom OpenPGP card, you can flash VID:PID of any popular commercial and sometimes proprietary OpenPGP card. This also offers the developer of Pico Keys. But it's not clear for me as a perfectionist. It's horrible when I see in the terminal that my own OpenPGP key is named as Yubikey, it's not funny. Maybe another software when will see it, try to use some features of Yubikey and cause errors. It's not good! So I want to flash test VID:PID (FEFF:FCFD) with my own name and say pcsclite to use it.

This tool adds your unofficial VID:PID entry to pcsclite's Info.plist file. In the first tool wrote for Pico OpenPGP, but you can find another use.

Author: Орест Сметрний (foresle)

License: MIT