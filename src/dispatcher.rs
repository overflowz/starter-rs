use crossfire::mpmc::Array;
use crossfire::{MAsyncRx, MAsyncTx, MRx, MTx};

pub enum Message<M> {
    Action(M),
}

pub struct DispatcherTx<M>
where
    M: Unpin + Send + 'static,
{
    tx: MAsyncTx<Array<Message<M>>>,
    tx_blocking: MTx<Array<Message<M>>>,
}

impl<M> Clone for DispatcherTx<M>
where
    M: Unpin + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            tx_blocking: self.tx_blocking.clone(),
        }
    }
}

impl<M> DispatcherTx<M>
where
    M: Unpin + Send + 'static,
{
    #[must_use = "the action is silently dropped if the store loop is gone"]
    pub async fn dispatch<A>(&self, action: A) -> bool
    where
        A: Into<M>,
    {
        self.tx.send(Message::Action(action.into())).await.is_ok()
    }

    #[must_use = "the action is silently dropped if the store loop is gone"]
    pub fn dispatch_blocking<A>(&self, action: A) -> bool
    where
        A: Into<M>,
    {
        self.tx_blocking
            .send(Message::Action(action.into()))
            .is_ok()
    }
}

pub struct DispatcherRx<M>
where
    M: Unpin + Send + 'static,
{
    rx: MAsyncRx<Array<Message<M>>>,
    rx_blocking: MRx<Array<Message<M>>>,
}

impl<M> DispatcherRx<M>
where
    M: Unpin + Send + 'static,
{
    pub async fn recv(&self) -> Option<Message<M>> {
        self.rx.recv().await.ok()
    }

    pub fn recv_blocking(&self) -> Option<Message<M>> {
        self.rx_blocking.recv().ok()
    }
}

pub struct Dispatcher;

impl Dispatcher {
    pub fn bounded<M>(size: usize) -> (DispatcherTx<M>, DispatcherRx<M>)
    where
        M: Unpin + Send + 'static,
    {
        let (tx, rx) = crossfire::mpmc::bounded_async::<Message<M>>(size);

        (
            DispatcherTx {
                tx: tx.clone(),
                tx_blocking: tx.into_blocking(),
            },
            DispatcherRx {
                rx: rx.clone(),
                rx_blocking: rx.into_blocking(),
            },
        )
    }
}
