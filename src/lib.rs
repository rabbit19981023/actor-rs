use tokio::sync::{mpsc, oneshot};

pub enum ActorSignal {
    Continue,
    Stop,
}

pub trait Actor: Sized + 'static {
    type State: Send;
    type Msg: Send;

    fn handle(state: &mut Self::State, msg: Self::Msg) -> ActorSignal;

    fn spawn(init_state: Self::State, buffer: usize) -> ActorHandle<Self> {
        let (sender, mut receiver) = mpsc::channel(buffer);

        tokio::spawn(async move {
            let mut state = init_state;

            while let Some(msg) = receiver.recv().await {
                if let ActorSignal::Stop = Self::handle(&mut state, msg) {
                    break;
                }
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

    pub async fn ask<F, R>(&self, into_msg: F) -> R
    where
        F: FnOnce(oneshot::Sender<R>) -> A::Msg,
        R: Send,
    {
        let (sender, receiver) = oneshot::channel();
        let msg = into_msg(sender);
        self.send(msg).await;
        receiver.await.unwrap()
    }
}
