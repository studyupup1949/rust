use std::io::Error;
use crate::{Message,Address, Context};

pub trait Actor<A, M, C>: Sized 
    where  M: Message, 
           A: Address<M>, 
           C: Context {
    fn on_init(&mut self, context: &mut C) -> Result<(), Error>;
    fn on_message(&mut self, context: &mut C, message: (A,M)) -> Result<(), Error>;
    fn on_terminate(&mut self, context: &mut C) -> Result<(), Error>;
}
