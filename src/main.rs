use actor_rs::Actor;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let counter = Counter::spawn(0);

    counter.incr().await;
    counter.incr().await;
    counter.incr().await;
    counter.decr().await;

    println!("count: {}", counter.get().await);
}

struct Counter {
    sender: mpsc::Sender<CounterMsg>,
}

enum CounterMsg {
    Get { reply_to: oneshot::Sender<u32> },
    Incr,
    Decr,
}

impl Actor for Counter {
    type State = u32;
    type Msg = CounterMsg;

    fn from_sender(sender: mpsc::Sender<CounterMsg>) -> Self {
        Self { sender }
    }

    fn sender(&self) -> &mpsc::Sender<CounterMsg> {
        &self.sender
    }

    fn get_fn(&self) -> fn(oneshot::Sender<u32>) -> CounterMsg {
        |sender| CounterMsg::Get { reply_to: sender }
    }

    fn handle(state: &mut u32, msg: CounterMsg) {
        match msg {
            CounterMsg::Get { reply_to } => reply_to.send(*state).unwrap(),
            CounterMsg::Incr => *state += 1,
            CounterMsg::Decr => *state -= 1,
        }
    }
}

impl Counter {
    async fn incr(&self) {
        self.send(CounterMsg::Incr).await;
    }

    async fn decr(&self) {
        self.send(CounterMsg::Decr).await;
    }
}
