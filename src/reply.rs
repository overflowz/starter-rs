use tokio::sync::oneshot;

use crate::{EnablingConditionErr, Response};

type ReplyResult = Result<Response, EnablingConditionErr>;

pub struct ReplySender {
    tx: oneshot::Sender<ReplyResult>,
}

impl ReplySender {
    pub fn channel() -> (Self, ReplyReceiver) {
        let (tx, rx) = oneshot::channel();

        (Self { tx }, ReplyReceiver { rx })
    }

    pub fn respond(self, response: impl Into<Response>) -> Result<(), ReplyError> {
        self.tx
            .send(Ok(response.into()))
            .map_err(|_| ReplyError::ReceiverDropped)
    }

    pub fn reject(self, error: EnablingConditionErr) -> Result<(), ReplyError> {
        self.tx
            .send(Err(error))
            .map_err(|_| ReplyError::ReceiverDropped)
    }
}

pub struct ReplyReceiver {
    rx: oneshot::Receiver<ReplyResult>,
}

impl ReplyReceiver {
    pub async fn recv(self) -> Result<Response, ReplyError> {
        match self.rx.await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(ReplyError::Rejected(error)),
            Err(_) => Err(ReplyError::NoReply),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReplyError {
    #[error("the store is not running")]
    StoreClosed,
    #[error("the action was rejected: {0}")]
    Rejected(EnablingConditionErr),
    #[error("the effect produced no reply")]
    NoReply,
    #[error("the reply receiver was dropped")]
    ReceiverDropped,
}
