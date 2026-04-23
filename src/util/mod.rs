use std::collections::{HashMap, hash_map};

use slotmap::{Key, SlotMap};

pub struct NamedContainer<K: Key, V> {
    by_name: HashMap<String, K>,
    entries: SlotMap<K, V>,
}

impl<K: Key, V> NamedContainer<K, V> {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            entries: SlotMap::with_key(),
        }
    }

    pub fn define<T: FnOnce() -> V>(&mut self, name: String, func: T) -> Result<K, K> {
        let entry = self.by_name.entry(name.to_owned());

        match entry {
            hash_map::Entry::Occupied(occupied_entry) => Err(*occupied_entry.get()),
            hash_map::Entry::Vacant(vacant_entry) => {
                let k = self.entries.insert(func());
                vacant_entry.insert(k);
                Ok(k)
            }
        }
    }

    pub fn define_unnamed(&mut self, ent: V) -> K {
        let k = self.entries.insert(ent);
        k
    }

    pub fn by_name(&self, name: &str) -> Option<K> {
        self.by_name.get(name).copied()
    }

    pub fn by_key(&self, key: K) -> Option<&V> {
        self.entries.get(key)
    }

    pub fn by_key_mut(&mut self, key: K) -> Option<&mut V> {
        self.entries.get_mut(key)
    }
}
