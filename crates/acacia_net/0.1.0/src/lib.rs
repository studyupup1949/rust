//! # acacia_net
//! `acacia_net` is a library for the Minecraft: Java Edition network protocol.
//! It currently supports Protocol 404 (Minecraft 1.13.2) only.
//! 
//! You are strongly advised to use [this wiki](wiki.vg/Protocol) for reference as it provides information on what some of the packets actually do.
//! 
//! # Invariants
//! This library doesn't check some packets for correctness before serialising them because it would be really hard, so for some packets you must make sure that you don't break any invariants unless you want to confuse the client. Here are the invariants.
//! 
//!   * Any packets with an `Option` in them probably have a related field of type `bool`. This must be set to `true` for `Some(_)` and `false` for `None`.
//!   * The clientbound `PlayerListItem` packet is a tough beast. At the beginning is a field named `action` and this must match the variant of each `PlayerListAction` enum used in the packet. The values are 0-4 inclusively for the variant in the order they are in in the source. If you want to perform the same action type on multiple players, go ahead, but if you want to perform different actions on different players you'll have to put each action type in a separate packet.
//! 
//! If you want this to change, contact Mojang. (except the first one - I could probably fix that myself)

#![allow(clippy::cast_lossless, clippy::large_enum_variant)]

extern crate byteorder;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate nbt;

pub mod serialise;
pub mod deserialise;
pub mod packets;
pub mod types;

#[cfg(test)]
mod tests;
