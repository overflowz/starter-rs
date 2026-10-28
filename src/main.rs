use starter_rs::{
    Action, Context, State, Store, StoreBuilder,
    dispatcher::{Dispatcher, DispatcherRx, Message},
    modules,
    reply::ReplySender,
    subscriptions,
};

fn root_reducer(state: &mut State, action: &Action) {
    subscriptions!(action, state => {
        Action::Dummy(_) => [modules::dummy::dummy_reducer],
    });
}

fn root_effect(store: &mut Store, action: &Action, reply: &mut Option<ReplySender>) {
    subscriptions!(action, store, reply => {
        Action::Dummy(_) => [modules::dummy::dummy_effect],
    });
}

fn main_loop(rx: DispatcherRx<Action>, mut store: Store) {
    while let Some(Message { action, mut reply }) = rx.recv_blocking() {
        let Err(err) = store.dispatch(&action, &mut reply) else {
            continue;
        };

        if let Some(reply) = reply {
            let _ = reply.reject(err);
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
