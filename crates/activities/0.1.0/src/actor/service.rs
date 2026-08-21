//
//
// use crate::actor::Actor;
// use crate::TypedType;
// use serde::{Deserialize, Serialize};
// use std::ops::Deref;
//
// #[derive(Serialize, Deserialize)]
// pub struct Service(Actor);
//
// #[typetag::serde]
// impl TypedType for Service {}
//
// impl Deref for Service {
//     type Target = Actor;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
