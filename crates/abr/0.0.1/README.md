# ADR

## Android Device Resizer

ARD is a cli tool made with Rust. This tool allows to resize the screen of a device connected to one of the following preset dimensions:

- ldpi => Galaxy Y => (240x320,133)
- mdpi => iphone => (320x480,165)
- hdpi => Android One => (480x854,218)
- xhdpi => Moto x => (720x1280,312)
- xxhdpi => Google Pixel => (1080x1920,441)
- xxxhdpi => Google Pixel XL => (1440x2560,534)

Example:
this will reisze the android device to the mdpi size

```sh
adr mdpi
```

# Installation!

- [crates] The package can be found on

## License

MIT

**Free Software, Hell Yeah!**

[//]: # "These are reference links used in the body of this note and get stripped out when the markdown processor does its job. There is no need to format nicely because it shouldn't be seen"
[crates]: https://crates.io/
