use crate::reply::ReplySender;
use crate::{Action, EnablingConditionErr, State, Store, store::EnablingCondition};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DummyAction {
    Noop,
    Ping,
    PingAsync,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DummyResponse {
    Pong,
}

#[derive(Debug, thiserror::Error)]
pub enum DummyEnablingConditionErr {
    #[error("dummy error: {0}")]
    Err(&'static str),
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct DummyState {}

pub fn dummy_reducer(_state: &mut State, _action: &Action) {}

pub fn dummy_effect(store: &mut Store, action: &Action, reply: &mut Option<ReplySender>) {
    match action {
        Action::Dummy(DummyAction::Ping) => {
            if let Some(reply) = reply.take() {
                let _ = reply.respond(DummyResponse::Pong);
            }
        }
        Action::Dummy(DummyAction::PingAsync) => {
            let Some(reply) = reply.take() else {
                return;
            };

            store.context().runtime.spawn(async move {
                tokio::task::yield_now().await;
                let _ = reply.respond(DummyResponse::Pong);
            });
        }
        _ => {}
    }
}

impl EnablingCondition<State, EnablingConditionErr> for DummyAction {
    fn is_enabled(&self, _state: &State) -> Result<(), EnablingConditionErr> {
        match self {
            Self::Reject => Err(DummyEnablingConditionErr::Err("dummy action rejected").into()),
            Self::Noop | Self::Ping | Self::PingAsync => Ok(()),
        }
    }
}
