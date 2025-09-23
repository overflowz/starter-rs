#[derive(Debug)]
#[must_use]
pub enum DispatchResult<E> {
    SendChannelClosed,
    EnablingConditionErr(E),
    Sent,
}

#[derive(Debug)]
#[must_use]
pub enum SendResult<E, R> {
    SendChannelClosed,
    NoResponse,
    EnablingConditionErr(E),
    Response(R),
}

pub enum Message<M, E, R> {
    Action(
        M,
        Option<crossfire::AsyncTx<E>>,
        Option<crossfire::MAsyncTx<R>>,
    ),
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
        let ((tx1, rx1), (tx2, rx2)) = (
            crossfire::spsc::bounded_async::<E>(1),
            crossfire::mpsc::bounded_async::<R>(1),
        );

        if self
            .tx
            .send(Message::Action(message.into(), Some(tx1), Some(tx2)))
            .await
            .is_err()
        {
            return SendResult::SendChannelClosed;
        }

        if let Ok(res) = rx1.recv().await {
            return SendResult::EnablingConditionErr::<E, R>(res);
        }

        if let Ok(res) = rx2.recv().await {
            return SendResult::Response::<E, R>(res);
        }

        SendResult::NoResponse
    }

    pub async fn dispatch<A>(&self, action: A) -> DispatchResult<E>
    where
        A: Into<M>,
    {
        if self
            .tx
            .send(Message::Action(action.into(), None, None))
            .await
            .is_err()
        {
            return DispatchResult::SendChannelClosed;
        }

        DispatchResult::Sent
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
