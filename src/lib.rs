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
    ($store: ident, $action: ident, $responder: ident) => {};
    ($store: ident, $action: ident, $responder: ident, $($handler: expr),+) => {
        $( $handler($store, $action, $responder); )+
    };
}

#[derive(Debug, thiserror::Error)]
pub enum EnablingConditionErr {
    #[error("dummy error: {0}")]
    Dummy(#[from] modules::dummy::DummyEnablingConditionErr),
}

#[derive(Debug, derive_more::From)]
pub enum Response {
    Dummy(Result<modules::dummy::DummyResponseOk, modules::dummy::DummyResponseErr>),
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

pub type Store = store::Store<State, Action, EnablingConditionErr, Response>;
pub type StoreBuilder = store::StoreBuilder<State, Action, EnablingConditionErr, Response>;
