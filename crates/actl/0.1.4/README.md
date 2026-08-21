# actl

actl is a simple but powerful system maintenance tool for Archlinux.

Note: The documentation is a work in progress and may be outdated or incorrect. If this is the case, I as the maintainer of this project would greatly appreciate if you could update it. Otherwise just raise an issue and I or someone else can fix it when we time.

## Dependencies

- sudo

- trizen

- rkhunter

## Construction of valid commands

Valid commands are constructed with the below formula.

actl -[CATEGORY SWITCH][OPERATION SWITCHES] [ADDITIONAL ARGUMENTS]

Where [CATEGORY SWITCH] is the switch for the category the operation you want to access resides in.

That is followed by [OPERATION SWITCHES] which are one or multiple

If some operation requires additional arguments. `-Sp` for example. They are to be supplied here.

## Operation categories

Since this program has a wide range of features, I've decided to split every "operation" into groups. These can be listed by typing `actl -h`.
Currently single operations that don't fit in a group are given their own category with no parameters to them. This is intended to be changed in the future when there are more of these operations. All current categories are listed below.

- `pctls` for package related stuff.

- `ethct` for internet connectivity related stuff.

- `sysec` for system security stuff.

## pctls

This category contains operations related to packages. All possible operations and their respective switches are listed below.

| Switch          | Description     |
| --------------- | --------------- |
| P | The category switch. |
| y | Update all packages on the system. Including AUR packages. |
| o | Optimize the pacman database. |
| k | Clean the package cache. |
| u | Display a list of orphan packages. |

## ethct

This category contains operations related to internet connectivity. All possible operations and their respective switches are listed below.

| Switch          | Description     |
| --------------- | --------------- |
| E | The category switch. Connects to the internet via ethernet using the adapter configured in the program config. |

## sysec

This category contains operations related to system security. All possible operations and their respective switches are listed below.

| Switch          | Description     |
| --------------- | --------------- |
| S | The category switch. |
| p | Generate a cryptographically random string of ASCII characters for passwords using the length supplied as an additional argument. |
