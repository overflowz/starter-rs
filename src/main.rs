use starter_rs::{
    Action, Context, State, Store, StoreBuilder, chain_effects, chain_reducers,
    dispatcher::{Dispatcher, DispatcherRx, Message},
    modules,
};

fn root_reducer(state: &mut State, action: &Action) {
    chain_reducers!(state, action, modules::dummy::dummy_reducer);
}

fn root_effect(store: &mut Store, action: &Action) {
    chain_effects!(store, action, modules::dummy::dummy_effect);
}

fn main_loop(rx: DispatcherRx<Action>, mut store: Store) {
    while let Some(msg) = rx.recv_blocking() {
        match msg {
            Message::Action(action) => {
                if let Err(err) = store.dispatch(action) {
                    eprintln!("action rejected: {err}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    crossfire::detect_backoff_cfg();

    let (tx, rx) = Dispatcher::bounded::<Action>(u16::MAX as usize);

    let _main_loop_handle = {
        let tx = tx.clone();
        let runtime_handle = tokio::runtime::Handle::current();

        std::thread::Builder::new()
            .name("store".to_owned())
            .spawn(move || {
                let store = StoreBuilder::new(
                    State::default(),
                    root_reducer,
                    root_effect,
                    Context {
                        runtime: runtime_handle,
                        tx,
                    },
                )
                .build();

                main_loop(rx, store)
            })
            .unwrap()
    };

    // the core waits forever; the process lives until it is terminated.
    std::future::pending::<()>().await;
}
