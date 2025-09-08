#[derive(Debug)]
pub enum DispatchResult<E, R> {
    SendChannelClosed,
    NoResponse,
    EnablingConditionErr(E),
    Response(R),
}

pub enum Message<M, E, R> {
    Action(M, crossfire::AsyncTx<E>, crossfire::MAsyncTx<R>),
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
    pub async fn dispatch<A>(&self, message: A) -> DispatchResult<E, R>
    where
        A: Into<M>,
    {
        let ((tx1, rx1), (tx2, rx2)) = (
            crossfire::spsc::bounded_async::<E>(1),
            crossfire::mpsc::bounded_async::<R>(1),
        );

        if self
            .tx
            .send(Message::Action(message.into(), tx1, tx2))
            .await
            .is_err()
        {
            return DispatchResult::SendChannelClosed;
        }

        if let Ok(res) = rx1.recv().await {
            return DispatchResult::EnablingConditionErr::<E, R>(res);
        }

        if let Ok(res) = rx2.recv().await {
            return DispatchResult::Response::<E, R>(res);
        }

        DispatchResult::NoResponse
    }
}

pub struct Dispatcher;

impl Dispatcher {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<M, E, R>(size: usize) -> (DispatcherTx<M, E, R>, DispatcherRx<M, E, R>)
    where
        M: Unpin,
        E: Unpin,
    {
        let (tx, rx) = crossfire::mpmc::bounded_async(size);

        (DispatcherTx { tx }, DispatcherRx { rx })
    }
}
