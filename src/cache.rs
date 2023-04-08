use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    time::{Duration, Instant},
};

pub struct CacheItem<T> {
    expiration: Instant,
    value: T,
}

pub struct Cache<K: Hash + Eq, V> {
    inner: HashMap<K, CacheItem<V>>,
    ttl: Duration,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            ttl: Duration::from_secs(15 * 60),
        }
    }

    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let item = self.inner.get(key);
        match item {
            Some(cache_item) => {
                let now = Instant::now();

                if now < cache_item.expiration {
                    Some(&cache_item.value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        let cache_item = CacheItem {
            expiration: Instant::now() + self.ttl,
            value,
        };

        self.inner.insert(key, cache_item);
    }
}
