use actix::{Actor, Addr, Message, MessageResponse, System, SystemRunner};
use actix_handler_macro::{actix_handler, Actor};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Actor)]
struct Example;

type ExampleContext = actix::Context<Example>;

#[derive(Message, Clone)]
#[rtype(result = "Hello")]
struct Greeting;

#[derive(Message, MessageResponse)]
#[rtype(result = "()")]
struct Hello;

#[actix_handler]
impl Example {
    fn greet(&self, _message: Greeting, _ctx: &ExampleContext) -> Hello {
        Hello {}
    }

    fn say_hello(&self, _message: Hello, _ctx: &ExampleContext) {}
}

fn bench_send_msg_normal(sys: &mut SystemRunner, addr: Addr<Example>, greeting: Greeting) -> Hello {
    sys.block_on(async move {
        for _ in 0..1000u32 {
            addr.send(greeting.clone()).await.ok().unwrap();
        }
        addr.send(greeting).await.ok().unwrap()
    })
}

fn bench_send_msg_trait(sys: &mut SystemRunner, addr: Addr<Example>, greeting: Greeting) -> Hello {
    sys.block_on(async move {
        for _ in 0..1000u32 {
            addr.send(greeting.clone()).await.ok().unwrap();
        }
        addr.greet(greeting).await.ok().unwrap()
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut sys = System::new("actix-test-runtime");
    let addr = Example {}.start();

    c.bench_function("bench send msg trait", |b| {
        b.iter(|| bench_send_msg_trait(&mut sys, addr.clone(), black_box(Greeting {})))
    });
    c.bench_function("bench send msg normal", |b| {
        b.iter(|| bench_send_msg_normal(&mut sys, addr.clone(), black_box(Greeting {})))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
