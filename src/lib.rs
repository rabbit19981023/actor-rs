use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

#[async_trait]
pub trait Actor: Sized + 'static {
    type State: Send;
    type Msg: Send;

    fn from_sender(sender: mpsc::Sender<Self::Msg>) -> Self;
    fn sender(&self) -> &mpsc::Sender<Self::Msg>;
    fn handle(state: &mut Self::State, msg: Self::Msg);

    fn get_fn(&self) -> fn(oneshot::Sender<Self::State>) -> Self::Msg;

    fn spawn(init_state: Self::State) -> Self {
        let (sender, mut receiver) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut state = init_state;

            while let Some(msg) = receiver.recv().await {
                Self::handle(&mut state, msg);
            }
        });

        Self::from_sender(sender)
    }

    async fn send(&self, msg: Self::Msg) {
        self.sender().send(msg).await.unwrap()
    }

    async fn get(&self) -> Self::State {
        let (sender, receiver) = oneshot::channel();
        let msg = self.get_fn()(sender);
        self.send(msg).await;
        receiver.await.unwrap()
    }
}
