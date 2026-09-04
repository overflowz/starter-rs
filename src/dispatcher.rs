use tokio::sync::oneshot;

#[derive(Debug)]
#[must_use]
pub enum SendResult<E, R> {
    SendChannelClosed,
    NoResponse,
    EnablingConditionErr(E),
    Response(R),
}

pub enum Message<M, E, R> {
    Action(M, Option<oneshot::Sender<Result<R, E>>>),
}

pub struct DispatcherTx<M, E, R>
where
    M: Unpin,
{
    tx: crossfire::MAsyncTx<Message<M, E, R>>,
    tx_blocking: crossfire::MTx<Message<M, E, R>>,
}

impl<M, E, R> Clone for DispatcherTx<M, E, R>
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

impl<M, E, R> DispatcherTx<M, E, R>
where
    M: Unpin + Send + 'static,
    E: Unpin + Send + 'static,
    R: Unpin + Send + 'static,
{
    /// RPC round trip: awaits the reply channel.
    pub async fn send<A>(&self, message: A) -> SendResult<E, R>
    where
        A: Into<M>,
    {
        let (rtx, rrx) = oneshot::channel::<Result<R, E>>();

        if self
            .tx
            .send(Message::Action(message.into(), Some(rtx)))
            .await
            .is_err()
        {
            return SendResult::SendChannelClosed;
        }

        match rrx.await {
            Ok(Ok(response)) => SendResult::Response(response),
            Ok(Err(err)) => SendResult::EnablingConditionErr(err),
            Err(_) => SendResult::NoResponse,
        }
    }

    #[must_use = "the action is silently dropped if the store loop is gone"]
    pub async fn dispatch<A>(&self, action: A) -> bool
    where
        A: Into<M>,
    {
        self.tx
            .send(Message::Action(action.into(), None))
            .await
            .is_ok()
    }

    pub fn send_blocking<A>(&self, message: A) -> SendResult<E, R>
    where
        A: Into<M>,
    {
        let (rtx, rrx) = oneshot::channel::<Result<R, E>>();

        if self
            .tx_blocking
            .send(Message::Action(message.into(), Some(rtx)))
            .is_err()
        {
            return SendResult::SendChannelClosed;
        }

        match rrx.blocking_recv() {
            Ok(Ok(response)) => SendResult::Response(response),
            Ok(Err(err)) => SendResult::EnablingConditionErr(err),
            Err(_) => SendResult::NoResponse,
        }
    }

    #[must_use = "the action is silently dropped if the store loop is gone"]
    pub fn dispatch_blocking<A>(&self, action: A) -> bool
    where
        A: Into<M>,
    {
        self.tx_blocking
            .send(Message::Action(action.into(), None))
            .is_ok()
    }
}

pub struct DispatcherRx<M, E, R> {
    rx: crossfire::MAsyncRx<Message<M, E, R>>,
    rx_blocking: crossfire::MRx<Message<M, E, R>>,
}

impl<M, E, R> DispatcherRx<M, E, R> {
    pub async fn recv(&self) -> Option<Message<M, E, R>> {
        self.rx.recv().await.ok()
    }

    pub fn recv_blocking(&self) -> Option<Message<M, E, R>> {
        self.rx_blocking.recv().ok()
    }
}

pub struct Dispatcher;

impl Dispatcher {
    pub fn bounded<M, E, R>(size: usize) -> (DispatcherTx<M, E, R>, DispatcherRx<M, E, R>)
    where
        M: Unpin,
        E: Unpin,
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

