use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel(10);
    let mut counter = Counter::new(0);
    let counter_handle = CounterHandle { sender };

    tokio::spawn(async move {
        counter.run(receiver).await;
    });

    counter_handle.incr().await;
    counter_handle.incr().await;
    let count = counter_handle.get().await;
    println!("count: {}", count);
}

trait Actor {
    type State;
    type Msg;

    fn new(state: Self::State) -> Self;
    async fn run(&mut self, receiver: mpsc::Receiver<Self::Msg>);
}

trait ActorHandle {
    type State;

    async fn get(&self) -> Self::State;
}

struct Counter {
    count: u32,
}

struct CounterHandle {
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

    fn new(state: u32) -> Self {
        Counter { count: state }
    }

    async fn run(&mut self, mut receiver: mpsc::Receiver<CounterMsg>) {
        while let Some(msg) = receiver.recv().await {
            match msg {
                CounterMsg::Get { reply_to } => {
                    reply_to.send(self.count).unwrap();
                }
                CounterMsg::Incr => {
                    self.count += 1;
                }
                CounterMsg::Decr => {
                    self.count -= 1;
                }
            }
        }
    }
}

impl ActorHandle for CounterHandle {
    type State = u32;

    async fn get(&self) -> u32 {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(CounterMsg::Get { reply_to: sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
}

impl CounterHandle {
    async fn incr(&self) {
        self.sender.send(CounterMsg::Incr).await.unwrap();
    }

    async fn decr(&self) {
        self.sender.send(CounterMsg::Decr).await.unwrap();
    }
}
