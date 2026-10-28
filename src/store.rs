use enum_dispatch::enum_dispatch;

use crate::reply::ReplySender;

pub type Reducer<State, Action> = fn(&mut State, &Action);

pub type Effect<State, Action, EnablingConditionErr, Context = ()> =
    fn(&mut Store<State, Action, EnablingConditionErr, Context>, &Action, &mut Option<ReplySender>);

#[enum_dispatch]
pub trait EnablingCondition<State, EnablingConditionErr> {
    fn is_enabled(&self, _state: &State) -> Result<(), EnablingConditionErr> {
        Ok(())
    }
}

pub struct Store<State, Action, EnablingConditionErr, Context = ()> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Context>,
    context: Context,
}

impl<State, Action, EnablingConditionErr, Context>
    Store<State, Action, EnablingConditionErr, Context>
{
    pub fn new(
        state: State,
        reducer: Reducer<State, Action>,
        effect: Effect<State, Action, EnablingConditionErr, Context>,
        context: Context,
    ) -> Self {
        Self {
            state,
            reducer,
            effect,
            context,
        }
    }

    #[inline]
    pub fn dispatch(
        &mut self,
        action: &Action,
        reply: &mut Option<ReplySender>,
    ) -> Result<(), EnablingConditionErr>
    where
        Action: EnablingCondition<State, EnablingConditionErr>,
    {
        action.is_enabled(&self.state)?;

        (self.reducer)(&mut self.state, action);
        (self.effect)(self, action, reply);

        Ok(())
    }

    #[inline(always)]
    pub fn state(&self) -> &State {
        &self.state
    }

    #[inline(always)]
    pub fn context(&self) -> &Context {
        &self.context
    }

    #[inline(always)]
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

pub struct StoreBuilder<State, Action, EnablingConditionErr, Context = ()> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Context>,
    context: Context,
}

impl<State, Action, EnablingConditionErr, Context>
    StoreBuilder<State, Action, EnablingConditionErr, Context>
{
    pub fn new(
        state: State,
        reducer: Reducer<State, Action>,
        effect: Effect<State, Action, EnablingConditionErr, Context>,
        context: Context,
    ) -> Self {
        Self {
            state,
            reducer,
            effect,
            context,
        }
    }

    pub fn build(self) -> Store<State, Action, EnablingConditionErr, Context> {
        Store {
            state: self.state,
            reducer: self.reducer,
            effect: self.effect,
            context: self.context,
        }
    }
}
