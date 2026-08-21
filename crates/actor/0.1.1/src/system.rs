// use crate::{Actor, Context, ContextEvent, SystemHandle, SystemEvent, Message};
// use std::collections::{HashMap};
// use mio::{
//     Poll,
//     SetReadiness,
//     Registration,
//     Ready,
//     Token,
//     PollOpt,
//     Events,
//     Evented,
//     };
// use std::sync::mpsc::{
//     Receiver,
//     Sender,
//     channel,
//     };
// use std::io;

// const SYSTEM_TOKEN: Token = Token(0);
// const EVENT_CAPACITY: usize = 1024;

// pub struct System<A, M> 
//     where A: Actor, M: Message{
//         actors: HashMap<usize, Context<M,A>>, //Token => Receiving part of the context
//         poll: Poll,
//         rx: Receiver<SystemEvent>,
//         tx: Sender<SystemEvent>,
//         set_readiness: SetReadiness,
//         registration: Registration,
//         events: Events,
//         next_id: u64,
// }
// impl<A, M> System<A, M> 
//     where A: Actor, M: Message{
//        pub fn new() -> Self {
//         let (tx, rx) = channel();
//         let (registration, set_readiness) = Registration::new2();
//         System{
//             actors: HashMap::new(),
//             poll: Poll::new().unwrap(), //handle error case,
//             events: Events::with_capacity(EVENT_CAPACITY),
//             next_id: 1,
//             tx,
//             rx,
//             registration,
//             set_readiness,
//           }
//     }
//     pub fn handle(&self) -> SystemHandle {
//         SystemHandle::new(self.tx, self.set_readiness)
//     }
//     pub fn run(mut self) {
//         loop{
//             self.poll.poll(&mut self.events, None);
//             self.poll.register(&self,SYSTEM_TOKEN, Ready::readable(), PollOpt::edge());
//             for event in self.events.iter() {}
//         }
//     }
//     fn new_id(&mut self) -> usize{
//         let id = self.next_id;
//         self.next_id += 1;
//         id as usize
//     }
// }


// impl<A: Actor, M:Message> Evented for System<A,M>{
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