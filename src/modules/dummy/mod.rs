use crate::{Action, EnablingConditionErr, State, Store, store::EnablingCondition};

#[derive(Debug)]
pub enum DummyAction {
    Noop,
}

#[derive(Debug, thiserror::Error)]
pub enum DummyEnablingConditionErr {
    #[error("dummy error: {0}")]
    Err(&'static str),
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct DummyState {}

pub fn dummy_reducer(_state: &mut State, _action: &Action) {}

pub fn dummy_effect(_store: &mut Store, _action: &Action) {}

impl EnablingCondition<State, EnablingConditionErr> for DummyAction {}
