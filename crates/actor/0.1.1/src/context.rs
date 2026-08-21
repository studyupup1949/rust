pub trait Context: Sized + 'static{
    fn fork();
    fn kill();
    fn send();
}
// use std::sync::mpsc::{
//     channel, 
//     Sender, 
//     Receiver,
//     };
// use crate::{Actor, Address, Message};
// use std::io;
// use mio::{
//     Registration, 
//     Evented, 
//     Poll, 
//     PollOpt, 
//     Ready, 
//     Token, 
//     SetReadiness,
//     };

// #[derive(Debug)]
// pub enum ContextEvent<M: Message>{
//     OnCreated,
//     OnTerminated,
//     OnMessage(Address, M),
// }

// #[derive(Debug)]
// pub struct Context<M: Message, A: Actor>{
//     id: usize,
//     actor: A,
//     tx: Sender<ContextEvent<M>>,
//     rx: Receiver<ContextEvent<M>>,
//     pendingMessages: u64,
//     registration: Registration,
//     set_readiness: SetReadiness,
// }

// impl<A: Actor, M: Message> Context<M, A>{
//     pub fn new(id: usize, actor: A) -> Self {
//         let (tx, rx) = channel();
//         let (registration, set_readiness) = Registration::new2();
//         let pendingMessages = 0;
//         Context{tx,rx,actor, registration, set_readiness, id, pendingMessages}
//     }
//     pub fn get_id(&self) -> usize{self.id}
//     pub fn address(&self) -> Address{pub fn execute_event(&mut self) -> Result<(), io::Error>{
//         let _ = match self.rx.recv().unwrap(){
//             ContextEvent::OnCreated => self.actor.on_init(),
//             ContextEvent::OnTerminated => self.actor.on_terminate(),
//             ContextEvent::OnMessage(from, message) => self.actor.on_message(from, message),
//         };
//         Ok(())
//     }
// }

// impl<A: Actor> Evented for Context<A>{
//     fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
//         -> io::Result<()>{
//         self.registration.register(poll, token, interest, opts)
//     }
//     fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
//         -> io::Result<()>{
//         self.registration.reregister(poll, token, interest, opts)
//     }
//     fn deregister(&self, poll: &Poll) -> io::Result<()> {
//         self.registration.deregister(poll)
//     }
// }
