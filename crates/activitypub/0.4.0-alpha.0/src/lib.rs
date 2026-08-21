/*
 * This file is part of ActivityPub.
 *
 * Copyright © 2018 Riley Trautman
 *
 * ActivityPub is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ActivityPub is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ActivityPub.  If not, see <http://www.gnu.org/licenses/>.
 */

//! ActivityPub
//!
//! This crate defines the base set of types from the ActivityPub specification.
//!
//! ## Example Usage
//! ```rust
//! use activitypub::{
//!     context,
//!     object::{
//!         properties::{
//!             ApObjectProperties,
//!             ObjectProperties
//!         },
//!         Video,
//!     },
//! };
//! use anyhow::Error;
//!
//! fn init_video_oprops<T>(mut t: T) -> Result<T, Error>
//! where
//!     T: AsMut<ObjectProperties>,
//! {
//!     t.as_mut().set_context_xsd_any_uri(context())?;
//!     Ok(t)
//! }
//!
//! fn init_video_aoprops<T>(mut t: T) -> Result<T, Error>
//! where
//!     T: AsMut<ApObjectProperties>,
//! {
//!     t.as_mut().set_likes("https://my-instance.com/likes")?;
//!     Ok(t)
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let mut video = Video::default();
//!
//!     init_video_oprops(&mut video)?;
//!     init_video_aoprops(&mut video)?;
//!
//!     let video_string = serde_json::to_string(&video)?;
//!
//!     let video: Video = serde_json::from_str(&video_string)?;
//!
//!     Ok(())
//! }
//! ```
pub mod activity;
pub mod actor;
pub mod collection;
mod endpoint;
pub mod link;
pub mod object;

pub use self::{
    activity::{Activity, IntransitiveActivity},
    actor::Actor,
    collection::{Collection, CollectionPage},
    endpoint::EndpointProperties,
    link::Link,
    object::Object,
};
pub use activitystreams::{context, properties, PropRefs, UnitString};
