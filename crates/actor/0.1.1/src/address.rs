use crate::Message;

pub trait Address<M>: Sized 
    where M: Message {
    fn send(&mut self, from: Self, message: M);
}

// #[derive(Debug, Clone)]
// pub struct Address{
//     tx: Sender<ContextEvent>,
//     set_readiness: SetReadiness,
// }

// impl Address{
//     pub fn new(tx: Sender<ContextEvent>, set_readiness: SetReadiness) -> Self {
//         Address{tx, set_readiness}
//     }
//     pub fn send(&self,from: Address, message: impl Message){
//         self.tx.send(ContextEvent::OnMessage(from,message));
//         self.set_readiness.set_readiness(Ready::readable());
//     }
// }