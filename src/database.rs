use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use bytes::Bytes;
use tokio::{
    sync::{Mutex, Notify, broadcast},
    time::{self, Duration, Instant},
};
use tracing::debug;

const CAPACITY: usize = 1024;

#[derive(Debug)]
struct Entry {
    data: Bytes,
    expires_at: Option<Instant>,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>,
    pub_sub: HashMap<String, broadcast::Sender<bytes::Bytes>>,
    expirations: BTreeSet<(Instant, String)>,
    shutdown: bool,
}
impl State {
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .iter()
            .next()
            .map(|expiration| expiration.0)
    }
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    background_task: Notify,
}
impl Shared {
    async fn is_shutdown(&self) -> bool {
        self.state.lock().await.shutdown
    }

    async fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().await;
        if state.shutdown {
            return None;
        }
        let state = &mut *state;
        let now = Instant::now();
        while let Some(&(when, ref key)) = state.expirations.iter().next() {
            if when > now {
                return Some(when);
            }
            state.entries.remove(key);
            state.expirations.remove(&(when, key.clone()));
        }
        None
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Database {
    shared: Arc<Shared>,
}
impl Database {
    pub(crate) fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                pub_sub: HashMap::new(),
                expirations: BTreeSet::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        });
        tokio::spawn(purge_expired_tasks(shared.clone()));
        Database { shared }
    }

    pub(crate) async fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().await;
        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub(crate) async fn set(&self, key: String, data: Bytes, expire: Option<Duration>) {
        let mut state = self.shared.state.lock().await;
        let mut notify = false;
        let expires_at = expire.map(|duration| {
            let when = Instant::now() + duration;
            notify = state.next_expiration().map(|e| e > when).unwrap_or(true);
            when
        });
        let prev = state
            .entries
            .insert(key.clone(), Entry { data, expires_at });
        if let Some(prev) = prev {
            if let Some(when) = prev.expires_at {
                state.expirations.remove(&(when, key.clone()));
            }
        }

        if let Some(when) = expires_at {
            state.expirations.insert((when, key));
        }
        drop(state);
        if notify {
            self.shared.background_task.notify_one();
        }
    }

    pub(crate) async fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;
        let mut state = self.shared.state.lock().await;
        match state.pub_sub.entry(key) {
            Entry::Occupied(e) => e.get().subscribe(),
            Entry::Vacant(e) => {
                let (tx, rx) = broadcast::channel(CAPACITY);
                e.insert(tx);
                rx
            }
        }
    }
    pub(crate) async fn publish(&self, key: &str, value: Bytes) -> usize {
        let state = self.shared.state.lock().await;
        state
            .pub_sub
            .get(key)
            .and_then(|tx| tx.send(value).ok())
            .unwrap_or(0)
    }

    async fn shutdown_task(&self) {
        let mut state = self.shared.state.lock().await;
        state.shutdown = true;
        self.shared.background_task.notify_one();
    }
}

#[derive(Debug)]
pub(crate) struct DatabaseGuard {
    database: Database,
}
impl DatabaseGuard {
    pub(crate) fn new() -> Self {
        Self {
            database: Database::new(),
        }
    }
    pub(crate) fn db(&self) -> Database {
        self.database.clone()
    }
    pub(crate) async fn close(&self) {
        self.database.shutdown_task().await;
    }
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
    while !shared.is_shutdown().await {
        if let Some(when) = shared.purge_expired_keys().await {
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }

    debug!("Purge background task shut down")
}
