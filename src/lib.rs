use tokio::sync::{mpsc, oneshot};

pub trait Actor: Sized + 'static {
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

pub struct ActorHandle<A: Actor> {
    sender: mpsc::Sender<A::Msg>,
}

impl<A: Actor> ActorHandle<A> {
    pub async fn send(&self, msg: A::Msg) {
        self.sender.send(msg).await.unwrap()
    }

    pub async fn get(&self) -> A::State
    where
        A::Msg: From<oneshot::Sender<A::State>>,
    {
        let (sender, receiver) = oneshot::channel();
        self.send(A::Msg::from(sender)).await;
        receiver.await.unwrap()
    }
}
