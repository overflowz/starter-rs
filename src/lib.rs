use enum_dispatch::enum_dispatch;

use crate::store::EnablingCondition;

pub mod dispatcher;
pub mod modules;
pub mod reply;
pub mod store;

#[macro_export]
macro_rules! subscriptions {
    (
        $action:expr, $ctx:expr => {
            $(
                $pattern:pat => [ $($handler:path),* $(,)? ]
            ),* $(,)?
        }
    ) => {
        match $action {
            $(
                $pattern => {
                    $(
                        $handler($ctx, $action);
                    )*
                }
            ),*
        }
    };
    (
        $action:expr, $ctx:expr, $reply:expr => {
            $(
                $pattern:pat => [ $($handler:path),* $(,)? ]
            ),* $(,)?
        }
    ) => {
        match $action {
            $(
                $pattern => {
                    $(
                        $handler($ctx, $action, &mut *$reply);
                    )*
                }
            ),*
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub enum EnablingConditionErr {
    #[error("dummy error: {0}")]
    Dummy(#[from] modules::dummy::DummyEnablingConditionErr),
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub dummy: modules::dummy::DummyState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[enum_dispatch(EnablingCondition<State, EnablingConditionErr>)]
pub enum Action {
    Dummy(modules::dummy::DummyAction),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Response {
    Dummy(modules::dummy::DummyResponse),
}

impl From<modules::dummy::DummyResponse> for Response {
    fn from(response: modules::dummy::DummyResponse) -> Self {
        Self::Dummy(response)
    }
}

pub struct Context {
    pub runtime: tokio::runtime::Handle,
    pub tx: dispatcher::DispatcherTx<Action>,
}

pub type Store = store::Store<State, Action, EnablingConditionErr, Context>;
pub type StoreBuilder = store::StoreBuilder<State, Action, EnablingConditionErr, Context>;
