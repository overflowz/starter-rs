use tokio::sync::oneshot;

use starter_rs::{
    Action, EnablingConditionErr, Response, State, Store, StoreBuilder, chain_effects,
    chain_reducers,
    dispatcher::{Dispatcher, DispatcherRx, Message},
    modules,
};

fn root_reducer(state: &mut State, action: &Action) {
    chain_reducers!(state, action, modules::dummy::dummy_reducer);
}

fn root_effect(
    store: &mut Store,
    action: &Action,
    responder: &mut Option<oneshot::Sender<Result<Response, EnablingConditionErr>>>,
) {
    chain_effects!(store, action, responder, modules::dummy::dummy_effect);
}

fn main_loop(rx: DispatcherRx<Action, EnablingConditionErr, Response>, mut store: Store) {
    while let Some(msg) = rx.recv_blocking() {
        match msg {
            Message::Action(action, mut reply) => {
                if let Err(err) = store.dispatch(action, &mut reply)
                    && let Some(tx) = reply
                {
                    let _ = tx.send(Err(err));
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = Dispatcher::bounded::<Action, EnablingConditionErr, Response>(u16::MAX as usize);

    let _main_loop_handle = {
        let tx = tx.clone();
        let runtime_handle = tokio::runtime::Handle::current();

        std::thread::Builder::new()
            .name("store".to_owned())
            .spawn(move || {
                let store = StoreBuilder::new(State::default(), root_reducer, root_effect)
                    .with_context(runtime_handle)
                    .with_context(tx)
                    .build();

                main_loop(rx, store)
            })
            .unwrap()
    };

    // the core waits forever; the process lives until it is terminated.
    std::future::pending::<()>().await;
}

