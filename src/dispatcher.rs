pub enum Message<M> {
    Action(M),
}

pub struct DispatcherTx<M>
where
    M: Unpin,
{
    tx: crossfire::MAsyncTx<Message<M>>,
    tx_blocking: crossfire::MTx<Message<M>>,
}

impl<M> Clone for DispatcherTx<M>
where
    M: Unpin,
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

pub struct DispatcherRx<M> {
    rx: crossfire::MAsyncRx<Message<M>>,
    rx_blocking: crossfire::MRx<Message<M>>,
}

impl<M> DispatcherRx<M> {
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
        M: Unpin,
    {
        let (tx, rx) = crossfire::mpmc::bounded_async(size);

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
