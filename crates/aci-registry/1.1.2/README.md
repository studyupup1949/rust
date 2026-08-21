# aci-registry-rs

This crate provides programmatic interfaces to represent the types of Device Classes, Device Subclasses, Device Vendors, and Device Products. for the ACI Specification.

Additionally, this crate provides information about device classes, vendors, and well-known subclasses

## Features

The following crate features are provided:
* `fixed-repr`: Causes each well-known subclass enum, [`DeviceClass`], [`DeviceVendor`], [`SubclassId`], and [`ProductId`] to all be represented as a native-endian `u16`,
 which is suitable for encoding/decoding for transport, specifically as the Device Class/Subclass and Device Vendor/Product fields
* `extended-info`: Provides information that is generally unneeded by drivers or firmware for ACI Devices for well-known subclass enums, [`DeviceClass`], and [`DeviceVendor`]
* `non-authorative`: Provides enums in the `non_authorative` module for known product ID and subclass ID registrations. 

## Use
There are three primary uses for this crate:
* The development of firmware for ACI Compliant Devices, or low-level components of an ACI Host
* The development of drivers for ACI Complaint Devices, or generic portions of ACI Host System Software
* The development of tools which provide information about ACI devices

To Faciliate the first two use cases, the crate is `no_std` and does not require the `alloc` crate. This allows it to be used in bare metal freestanding systems without issue

## Other Subclasses/Product Ids

Subclasses of non-well known classes, and Product IDs are provided in the `non_authorative` module, which is enabled by the `non-authorative` feature. 

This information is not owned by the Clever-ISA Project, nor is its updates coordinated, so it's accuracy is not warrantied and the types are provided on an AS IS basis only. 

Enums provided in the module are affected as usual by the `fixed-repr` and `extended-info` features.

## Out of Date Registries

The registry associated with this crate is kept in sync by periodic updates within a few hours of modifications to the registry.
If newer registriations are not available, you may need to use a later version of this crate.

It is considered a semver patch to update the copy of the registry associated with a version of this crate.


## License

Copyright (C) 2024 Clever-ISA Project

This project is released under the terms of the Apache 2.0 or MIT License, at your option. 

Unless you state otherwise, any contribution intentionally submitted by you to this repository will be released under the above dual license. 