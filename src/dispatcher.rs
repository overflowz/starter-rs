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

pub struct DispatcherRx<M, E, R> {
    rx: crossfire::MAsyncRx<Message<M, E, R>>,
}

impl<M, E, R> DispatcherRx<M, E, R> {
    pub async fn recv(&self) -> Option<Message<M, E, R>> {
        self.rx.recv().await.ok()
    }
}

pub struct DispatcherTx<M, E, R>
where
    M: Unpin,
{
    tx: crossfire::MAsyncTx<Message<M, E, R>>,
}

impl<T, E, R> Clone for DispatcherTx<T, E, R>
where
    T: Unpin,
    E: Unpin,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<M, E, R> DispatcherTx<M, E, R>
where
    M: Unpin + Send + 'static,
    E: Unpin + Send + 'static,
    R: Unpin + Send + 'static,
{
    pub async fn send<A>(&self, message: A) -> SendResult<E, R>
    where
        A: Into<M>,
    {
        let (tx, rx) = oneshot::channel::<Result<R, E>>();

        if self
            .tx
            .send(Message::Action(message.into(), Some(tx)))
            .await
            .is_err()
        {
            return SendResult::SendChannelClosed;
        }

        match rx.await {
            Ok(Ok(response)) => SendResult::Response(response),
            Ok(Err(err)) => SendResult::EnablingConditionErr(err),
            Err(_) => SendResult::NoResponse,
        }
    }

    /// Fire-and-forget. Awaits only while the mailbox is full (backpressure).
    /// Returns `false` when the store loop is gone (mailbox closed).
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
}

pub struct Dispatcher;

impl Dispatcher {
    pub fn bounded<M, E, R>(size: usize) -> (DispatcherTx<M, E, R>, DispatcherRx<M, E, R>)
    where
        M: Unpin,
        E: Unpin,
    {
        let (tx, rx) = crossfire::mpmc::bounded_async(size);

        (DispatcherTx { tx }, DispatcherRx { rx })
    }
}
