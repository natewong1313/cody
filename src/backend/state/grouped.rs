use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use tokio::sync::watch;

use crate::backend::state::StateError;

pub(crate) struct GroupedState<G, K, V>
where
    G: Eq + Hash + Copy,
    K: Eq + Hash + Copy,
    V: Clone,
{
    name: &'static str,

    /// All of this is similar to whats documented in entity.rs
    /// We just need an extra layer since this involves grouping
    /// e.g. tracking sessions for a specific project
    groups: RwLock<HashMap<G, HashMap<K, V>>>,
    item_to_group: RwLock<HashMap<K, G>>,
    group_senders: Mutex<HashMap<G, watch::Sender<Vec<V>>>>,
    group_key_fn: fn(&V) -> G,
    item_key_fn: fn(&V) -> K,
    sort_fn: fn(&V, &V) -> Ordering,
}

impl<G, K, V> GroupedState<G, K, V>
where
    G: Eq + Hash + Copy,
    K: Eq + Hash + Copy,
    V: Clone,
{
    pub(crate) fn new(
        name: &'static str,
        initial: Vec<V>,
        group_key_fn: fn(&V) -> G,
        item_key_fn: fn(&V) -> K,
        sort_fn: fn(&V, &V) -> Ordering,
    ) -> Self {
        let mut groups: HashMap<G, HashMap<K, V>> = HashMap::new();
        let mut item_to_group = HashMap::new();

        for value in initial {
            let group_key = group_key_fn(&value);
            let item_key = item_key_fn(&value);
            groups
                .entry(group_key)
                .or_default()
                .insert(item_key, value.clone());
            item_to_group.insert(item_key, group_key);
        }

        Self {
            name,
            groups: RwLock::new(groups),
            item_to_group: RwLock::new(item_to_group),
            group_senders: Mutex::new(HashMap::new()),
            group_key_fn,
            item_key_fn,
            sort_fn,
        }
    }

    pub(crate) fn subscribe_group(
        &self,
        group_key: G,
    ) -> Result<watch::Receiver<Vec<V>>, StateError> {
        let mut senders = self.group_senders_lock()?;
        let initial = self.list_group(group_key)?;

        let sender = senders
            .entry(group_key)
            .or_insert_with(|| {
                let (sender, _) = watch::channel(initial);
                sender
            })
            .clone();

        Ok(sender.subscribe())
    }

    pub(crate) fn list_group(&self, group_key: G) -> Result<Vec<V>, StateError> {
        let mut snapshot = self
            .groups_read()?
            .get(&group_key)
            .map(|items| items.values().cloned().collect::<Vec<V>>())
            .unwrap_or_default();
        snapshot.sort_by(self.sort_fn);
        Ok(snapshot)
    }

    pub(crate) fn upsert(&self, value: V) -> Result<(), StateError> {
        let group_key = (self.group_key_fn)(&value);
        let item_key = (self.item_key_fn)(&value);

        let mut publish_old = None;
        {
            let mut item_to_group = self.item_to_group_write()?;
            let old_group = item_to_group.get(&item_key).copied();

            let mut groups = self.groups_write()?;

            if let Some(old_group_key) = old_group
                && old_group_key != group_key
            {
                if let Some(old_items) = groups.get_mut(&old_group_key) {
                    old_items.remove(&item_key);
                }
                publish_old = Some(old_group_key);
            }

            groups
                .entry(group_key)
                .or_default()
                .insert(item_key, value.clone());
            item_to_group.insert(item_key, group_key);
        }

        if let Some(old_group_key) = publish_old {
            self.publish_group(old_group_key)?;
        }
        self.publish_group(group_key)
    }

    pub(crate) fn remove(&self, item_key: K) -> Result<(), StateError> {
        let group_key = self.item_to_group_write()?.remove(&item_key);

        if let Some(group_key) = group_key {
            self.groups_write()?
                .entry(group_key)
                .or_default()
                .remove(&item_key);
            self.publish_group(group_key)?;
        }

        Ok(())
    }

    pub(crate) fn remove_group(&self, group_key: G) -> Result<(), StateError> {
        let removed_items = self.groups_write()?.remove(&group_key).unwrap_or_default();

        if !removed_items.is_empty() {
            let mut item_to_group = self.item_to_group_write()?;
            for item_key in removed_items.keys() {
                item_to_group.remove(item_key);
            }
        }

        self.group_senders_lock()?.remove(&group_key);

        Ok(())
    }

    fn publish_group(&self, group_key: G) -> Result<(), StateError> {
        if let Some(sender) = self.group_senders_lock()?.get(&group_key).cloned() {
            let _ = sender.send(self.list_group(group_key)?);
        }

        Ok(())
    }

    fn groups_read(&self) -> Result<RwLockReadGuard<'_, HashMap<G, HashMap<K, V>>>, StateError> {
        self.groups.read().map_err(|_| StateError::LockPoisoned {
            state: self.name,
            lock: "groups_read",
        })
    }

    fn groups_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<G, HashMap<K, V>>>, StateError> {
        self.groups.write().map_err(|_| StateError::LockPoisoned {
            state: self.name,
            lock: "groups_write",
        })
    }

    fn item_to_group_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<K, G>>, StateError> {
        self.item_to_group
            .write()
            .map_err(|_| StateError::LockPoisoned {
                state: self.name,
                lock: "item_to_group_write",
            })
    }

    fn group_senders_lock(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<G, watch::Sender<Vec<V>>>>, StateError> {
        self.group_senders
            .lock()
            .map_err(|_| StateError::LockPoisoned {
                state: self.name,
                lock: "group_senders",
            })
    }
}
