// use crate::{Actor, SystemEvent, Address};
// use mio::{SetReadiness};
// use std::sync::mpsc::{Receiver,Sender};
// use std::io;

// #[derive(Debug, Clone)]
// pub struct SystemHandle{
//     tx: Sender<SystemEvent>,
//     set_readiness: SetReadiness,
// }
// impl SystemHandle{
//     pub fn new(tx: Sender<SystemEvent>, set_readiness: SetReadiness) -> Self{
//         SystemHandle{tx, set_readiness}
//     }
//     pub fn fork(actor: impl Actor) -> Result<Address, io::Error>{
//         //send message to actual system to start an actor.
//         //create context, and then send the receiving hald of that context in the message
//         //return the sending half which is Address here.
//     }
//     pub fn kill(address: usize) -> Result<(), io::Error>{
//         Ok(())
//     }
// }
