use enum_dispatch::enum_dispatch;

use crate::store::EnablingCondition;

pub mod dispatcher;
pub mod modules;
pub mod store;

#[macro_export]
macro_rules! chain_reducers {
    ($state: ident, $action: ident) => {};
    ($state: ident, $action: ident, $($handler: expr),+) => {
        $( $handler($state, $action); )+
    };
}

#[macro_export]
macro_rules! chain_effects {
    ($store: ident, $action: ident) => {};
    ($store: ident, $action: ident, $($handler: expr),+) => {
        $( $handler($store, $action); )+
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

#[derive(Debug)]
#[enum_dispatch(EnablingCondition<State, EnablingConditionErr>)]
pub enum Action {
    Dummy(modules::dummy::DummyAction),
}

pub struct Context {
    pub runtime: tokio::runtime::Handle,
    pub tx: dispatcher::DispatcherTx<Action>,
}

pub type Store = store::Store<State, Action, EnablingConditionErr, Context>;
pub type StoreBuilder = store::StoreBuilder<State, Action, EnablingConditionErr, Context>;
