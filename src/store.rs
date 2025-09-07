use enum_dispatch::enum_dispatch;

pub type Reducer<State, Action> = fn(&mut State, &Action);
pub type Effect<State, Action, EnablingConditionErr, Response> = fn(
    &mut Store<State, Action, EnablingConditionErr, Response>,
    &Action,
    &crossfire::MAsyncTx<Response>,
);

#[enum_dispatch]
pub trait EnablingCondition<State, EnablingConditionErr> {
    fn is_enabled(&self, _state: &State) -> Result<(), EnablingConditionErr> {
        Ok(())
    }
}

pub struct Store<State, Action, EnablingConditionErr, Response> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Response>,
    contexts: anymap3::AnyMap,
}

impl<State, Action, EnablingConditionErr, Response>
    Store<State, Action, EnablingConditionErr, Response>
{
    pub fn dispatch<A>(
        &mut self,
        action: A,
        responder: &crossfire::MAsyncTx<Response>,
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
    pub fn context<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.contexts.get_mut()
    }
}

pub struct StoreBuilder<State, Action, EnablingConditionErr, Response> {
    state: State,
    reducer: Reducer<State, Action>,
    effect: Effect<State, Action, EnablingConditionErr, Response>,
    contexts: anymap3::AnyMap,
}

impl<State, Action, EnablingConditionErr, Response>
    StoreBuilder<State, Action, EnablingConditionErr, Response>
{
    pub fn new(
        state: State,
        reducer: Reducer<State, Action>,
        effect: Effect<State, Action, EnablingConditionErr, Response>,
    ) -> Self {
        Self {
            state,
            reducer,
            effect,
            contexts: anymap3::AnyMap::new(),
        }
    }

    pub fn with_context<C>(mut self, cx: C) -> Self
    where
        C: 'static,
    {
        self.contexts.insert(cx);
        self
    }

    pub fn build(self) -> Store<State, Action, EnablingConditionErr, Response> {
        Store {
            state: self.state,
            reducer: self.reducer,
            effect: self.effect,
            contexts: self.contexts,
        }
    }
}
