use actix::{Actor, Message, System};
use actix_handler_macro::{actix_handler, Actor};

#[derive(Actor)]
struct Example;

type ExampleContext = actix::Context<Example>;

#[derive(Message)]
#[rtype(result = "()")]
struct Greeting;

#[derive(Message)]
#[rtype(result = "()")]
struct Hello;

#[actix_handler]
impl Example {
    fn greet(&self, _message: Greeting, _ctx: &ExampleContext) {
        println!("Received greeting!");
    }

    fn say_hello(&self, _message: Hello, _ctx: &ExampleContext) {
        println!("Received hello!");
    }
}

#[derive(Actor)]
struct GreeterImpl;

#[actix_handler(trait_name = "Greeter", use_recipient)]
impl GreeterImpl {
    fn greet(&self, _message: Greeting, _ctx: &actix::Context<Self>) {
        println!("Received greeting!");
    }
}

#[test]
fn test_message() {
    let mut sys = System::new("actix-test-runtime");
    let addr = Example {}.start();
    sys.block_on(async move {
        let _result = addr.send(Greeting {}).await.ok().unwrap();
        let _result = addr.send(Hello {}).await.ok().unwrap();
    });
}

#[test]
fn test_addr_trait() {
    let mut sys = System::new("actix-test-runtime");
    let addr = Example {}.start();
    sys.block_on(async move {
        let _result = addr.greet(Greeting {}).await.ok().unwrap();
        let _result = addr.say_hello(Hello {}).await.ok().unwrap();
    });
}
