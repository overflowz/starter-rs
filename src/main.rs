use starter_rs::{
    Action, EnablingConditionErr, Response, State, Store, StoreBuilder, chain_effects,
    chain_reducers,
    dispatcher::{Dispatcher, Message},
    modules,
};

fn root_reducer(state: &mut State, action: &Action) {
    chain_reducers!(state, action, modules::dummy::dummy_reducer);
}

fn root_effect(
    store: &mut Store,
    action: &Action,
    responder: &mut Option<crossfire::MAsyncTx<Response>>,
) {
    chain_effects!(store, action, responder, modules::dummy::dummy_effect);
}

#[tokio::main]
async fn main() {
    let (tx, rx) = Dispatcher::bounded::<Action, EnablingConditionErr, Response>(u16::MAX as usize);

    let mut store = StoreBuilder::new(State::default(), root_reducer, root_effect)
        .with_context(tx.clone())
        .build();

    while let Some(msg) = rx.recv().await {
        match msg {
            Message::Action(action, tx1, mut tx2) => {
                if let Err(err) = store.dispatch(action, &mut tx2)
                    && let Some(tx1) = tx1
                {
                    let _ = tx1.send(err).await;
                }
            }
        }
    }
}
