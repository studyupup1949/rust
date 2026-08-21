use actix::{Handler, Supervisor, System};
use actix_derive::Message;
use actix_handler_macro::{Actor, Supervised};

#[derive(Message)]
#[rtype(result = "()")]
struct Sum;

#[derive(Actor, Supervised, Default)]
#[actor(context = "::actix::SyncContext")]
struct Adder;

impl Handler<Sum> for Adder {
    type Result = <Sum as actix::Message>::Result;
    fn handle(&mut self, _: Sum, _: &mut Self::Context) -> () {
        ()
    }
}

#[test]
fn test_message() {
    let mut sys = System::new("actix-test-runtime");
    let addr = Supervisor::start(|_| Adder);
    let _ = sys.block_on(addr.send(Sum)).unwrap();
}
