use enum_dispatch::enum_dispatch;
use tokio::sync::oneshot;

pub type Reducer<State, Action> = fn(&mut State, &Action);
pub type Effect<State, Action, EnablingConditionErr, Response, Context = ()> = fn(
    &mut Store<State, Action, EnablingConditionErr, Response, Context>,
    &Action,
    &mut Option<oneshot::Sender<Result<Response, EnablingConditionErr>>>,
);

#[enum_dispatch]
pub trait EnablingCondition<State, EnablingConditionErr> {
    fn is_enabled(&self, _state: &State) -> Result<(), EnablingConditionErr> {
        Ok(())
    }
}

pub struct Store<State, Action, EnablingConditionErr, Response, Context = ()> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Response, Context>,
    context: Context,
}

impl<State, Action, EnablingConditionErr, Response, Context>
    Store<State, Action, EnablingConditionErr, Response, Context>
{
    pub fn new(
        state: State,
        reducer: Reducer<State, Action>,
        effect: Effect<State, Action, EnablingConditionErr, Response, Context>,
        context: Context,
    ) -> Self {
        Self {
            state,
            reducer,
            effect,
            context,
        }
    }

    pub fn dispatch<A>(
        &mut self,
        action: A,
        responder: &mut Option<oneshot::Sender<Result<Response, EnablingConditionErr>>>,
    ) -> Result<(), EnablingConditionErr>
    where
        A: EnablingCondition<State, EnablingConditionErr> + Into<Action>,
    {
        action.is_enabled(&self.state)?;

        let action = action.into();

        (self.reducer)(&mut self.state, &action);
        (self.effect)(self, &action, responder);

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

pub struct StoreBuilder<State, Action, EnablingConditionErr, Response, Context = ()> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Response, Context>,
    context: Context,
}

impl<State, Action, EnablingConditionErr, Response, Context>
    StoreBuilder<State, Action, EnablingConditionErr, Response, Context>
{
    pub fn new(
        state: State,
        reducer: Reducer<State, Action>,
        effect: Effect<State, Action, EnablingConditionErr, Response, Context>,
        context: Context,
    ) -> Self {
        Self {
            state,
            reducer,
            effect,
            context,
        }
    }

    pub fn build(self) -> Store<State, Action, EnablingConditionErr, Response, Context> {
        Store {
            state: self.state,
            reducer: self.reducer,
            effect: self.effect,
            context: self.context,
        }
    }
}
