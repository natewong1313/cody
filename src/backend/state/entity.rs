use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tokio::sync::watch;

use crate::backend::state::StateError;

pub(crate) struct EntityState<K, V>
where
    K: Eq + Hash + Copy,
    V: Clone,
{
    name: &'static str,

    /// The in-memory state
    items: RwLock<HashMap<K, V>>,

    /// All entities, e.g. a subscriber wants to listen to all projects
    all_sender: watch::Sender<Vec<V>>,
    /// We also need to handle subscribing to a specific entity
    item_senders: Mutex<HashMap<K, watch::Sender<Option<V>>>>,

    /// Given an entity, get its corresponding key. e.g. Project -> Project.id
    key_fn: fn(&V) -> K,
    /// TODO: should improve declaring this, don't want to stray from DB
    sort_fn: fn(&V, &V) -> Ordering,
}

impl<K, V> EntityState<K, V>
where
    K: Eq + Hash + Copy,
    V: Clone,
{
    pub(crate) fn new(
        name: &'static str,
        initial: Vec<V>,
        key_fn: fn(&V) -> K,
        sort_fn: fn(&V, &V) -> Ordering,
    ) -> Self {
        let mut items = HashMap::with_capacity(initial.len());
        for value in initial.iter() {
            items.insert(key_fn(value), value.clone());
        }

        let mut snapshot = initial;
        snapshot.sort_by(sort_fn);
        let (all_sender, _) = watch::channel(snapshot);

        Self {
            name,
            items: RwLock::new(items),
            all_sender,
            item_senders: Mutex::new(HashMap::new()),
            key_fn,
            sort_fn,
        }
    }

    pub(crate) fn subscribe_all(&self) -> watch::Receiver<Vec<V>> {
        self.all_sender.subscribe()
    }

    pub(crate) fn subscribe_one(&self, key: K) -> Result<watch::Receiver<Option<V>>, StateError> {
        let mut senders = self.item_senders_lock()?;

        let initial = self.items_read()?.get(&key).cloned();

        let sender = senders
            .entry(key)
            .or_insert_with(|| {
                let (sender, _) = watch::channel(initial);
                sender
            })
            .clone();

        Ok(sender.subscribe())
    }

    pub(crate) fn list(&self) -> Result<Vec<V>, StateError> {
        let mut snapshot: Vec<V> = self.items_read()?.values().cloned().collect();
        snapshot.sort_by(self.sort_fn);
        Ok(snapshot)
    }

    pub(crate) fn get(&self, key: K) -> Result<Option<V>, StateError> {
        Ok(self.items_read()?.get(&key).cloned())
    }

    pub(crate) fn upsert(&self, value: V) -> Result<(), StateError> {
        let key = (self.key_fn)(&value);

        self.items_write()?.insert(key, value.clone());

        if let Some(sender) = self.item_senders_lock()?.get(&key).cloned() {
            let _ = sender.send(Some(value));
        }

        self.publish_all()
    }

    pub(crate) fn remove(&self, key: K) -> Result<(), StateError> {
        self.items_write()?.remove(&key);

        let sender = self.item_senders_lock()?.remove(&key);
        if let Some(sender) = sender {
            let _ = sender.send(None);
        }

        self.publish_all()
    }

    fn publish_all(&self) -> Result<(), StateError> {
        let _ = self.all_sender.send(self.list()?);
        Ok(())
    }

    fn items_read(&self) -> Result<RwLockReadGuard<'_, HashMap<K, V>>, StateError> {
        self.items.read().map_err(|_| StateError::LockPoisoned {
            state: self.name,
            lock: "items_read",
        })
    }

    fn items_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<K, V>>, StateError> {
        self.items.write().map_err(|_| StateError::LockPoisoned {
            state: self.name,
            lock: "items_write",
        })
    }

    fn item_senders_lock(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<K, watch::Sender<Option<V>>>>, StateError> {
        self.item_senders
            .lock()
            .map_err(|_| StateError::LockPoisoned {
                state: self.name,
                lock: "item_senders",
            })
    }
}
