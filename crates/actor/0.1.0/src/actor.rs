use std::io::Error;
use crate::{Message,Address};

pub trait Actor<A, M>: Sized 
    where  M: Message, A: Address<M> {
    fn on_init(&mut self) -> Result<(), Error>;
    fn on_message(&mut self, from: A, message: M) -> Result<(), Error>;
    fn on_terminate(&mut self) -> Result<(), Error>;
}
