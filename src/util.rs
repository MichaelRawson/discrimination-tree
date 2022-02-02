use std::fmt;

#[derive(Clone)]
pub(crate) struct SortedMap<K, V> {
    items: Vec<(K, V)>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for SortedMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.items.fmt(f)
    }
}

impl<K, V> Default for SortedMap<K, V> {
    fn default() -> Self {
        Self { items: vec![] }
    }
}

impl<K: Ord, V> SortedMap<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        match self.items.binary_search_by(|(k, _)| k.cmp(key)) {
            Ok(index) => Some(&self.items[index].1),
            Err(_) => None,
        }
    }

    pub(crate) fn get_or_insert_with<F: FnOnce() -> V>(
        &mut self,
        key: K,
        insert: F,
    ) -> &mut V {
        let index = match self.items.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(index) => index,
            Err(index) => {
                self.items.insert(index, (key, insert()));
                index
            }
        };
        &mut self.items[index].1
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.items.iter()
    }
}
