use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let counter = Counter::spawn(0, 32);

    counter.incr(2).await;
    counter.incr(2).await;
    counter.incr(4).await;
    counter.decr(3).await;

    println!("count: {}", counter.get().await);
}

struct Counter {
    sender: mpsc::Sender<CounterMsg>,
}

enum CounterMsg {
    Get { reply_to: oneshot::Sender<u32> },
    Incr(u32),
    Decr(u32),
}

impl Counter {
    fn spawn(init_state: u32, buffer: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel(buffer);

        tokio::spawn(async move {
            let mut state = init_state;

            while let Some(msg) = receiver.recv().await {
                match msg {
                    CounterMsg::Get { reply_to } => {
                        reply_to.send(state).unwrap();
                    }
                    CounterMsg::Incr(num) => {
                        state += num;
                    }
                    CounterMsg::Decr(num) => {
                        state -= num;
                    }
                }
            }
        });

        Self { sender }
    }

    async fn send(&self, msg: CounterMsg) {
        self.sender.send(msg).await.unwrap()
    }

    async fn get(&self) -> u32 {
        let (sender, receiver) = oneshot::channel();
        self.send(CounterMsg::Get { reply_to: sender }).await;
        receiver.await.unwrap()
    }

    async fn incr(&self, num: u32) {
        self.send(CounterMsg::Incr(num)).await
    }

    async fn decr(&self, num: u32) {
        self.send(CounterMsg::Decr(num)).await
    }
}
