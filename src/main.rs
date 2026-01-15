use actor_rs::{Actor, ActorHandle};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let counter = Counter::spawn(0, 32);

    counter.incr(2).await;
    counter.incr(2).await;
    counter.incr(4).await;
    counter.decr(3).await;

    println!("count: {}", counter.get().await);
}

struct Counter;

trait CounterApi {
    async fn get(&self) -> u32;
    async fn incr(&self, num: u32);
    async fn decr(&self, num: u32);
}

enum CounterMsg {
    Get { reply_to: oneshot::Sender<u32> },
    Incr(u32),
    Decr(u32),
}

impl Actor for Counter {
    type State = u32;
    type Msg = CounterMsg;

    fn handle(state: &mut u32, msg: CounterMsg) {
        match msg {
            CounterMsg::Get { reply_to } => reply_to.send(*state).unwrap(),
            CounterMsg::Incr(num) => *state += num,
            CounterMsg::Decr(num) => *state -= num,
        }
    }
}

impl CounterApi for ActorHandle<Counter> {
    async fn get(&self) -> u32 {
        self.ask(|reply_to| CounterMsg::Get { reply_to }).await
    }

    async fn incr(&self, num: u32) {
        self.send(CounterMsg::Incr(num)).await;
    }

    async fn decr(&self, num: u32) {
        self.send(CounterMsg::Decr(num)).await;
    }
}
