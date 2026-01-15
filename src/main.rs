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

struct Counter;

enum CounterMsg {
    Get { reply_to: oneshot::Sender<u32> },
    Incr(u32),
    Decr(u32),
}

impl From<oneshot::Sender<u32>> for CounterMsg {
    fn from(sender: oneshot::Sender<u32>) -> Self {
        CounterMsg::Get { reply_to: sender }
    }
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

impl ActorHandle<Counter> {
    pub async fn incr(&self, num: u32) {
        self.send(CounterMsg::Incr(num)).await;
    }

    pub async fn decr(&self, num: u32) {
        self.send(CounterMsg::Decr(num)).await;
    }
}

trait Actor: Sized + 'static {
    type State: Send;
    type Msg: Send;

    fn handle(state: &mut Self::State, msg: Self::Msg);

    fn spawn(init_state: Self::State, buffer: usize) -> ActorHandle<Self> {
        let (sender, mut receiver) = mpsc::channel(buffer);

        tokio::spawn(async move {
            let mut state = init_state;

            while let Some(msg) = receiver.recv().await {
                Self::handle(&mut state, msg)
            }
        });

        ActorHandle { sender }
    }
}

struct ActorHandle<A: Actor> {
    sender: mpsc::Sender<A::Msg>,
}

impl<A: Actor> ActorHandle<A> {
    async fn send(&self, msg: A::Msg) {
        self.sender.send(msg).await.unwrap()
    }

    async fn get(&self) -> A::State
    where
        A::Msg: From<oneshot::Sender<A::State>>,
    {
        let (sender, receiver) = oneshot::channel();
        self.send(A::Msg::from(sender)).await;
        receiver.await.unwrap()
    }
}
