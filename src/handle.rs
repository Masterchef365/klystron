use std::collections::HashMap;

// TODO Use the type system to further protect me from myself.

pub type Id = u64;

pub struct HandleMap<T> {
    // TODO: Use a more appropriate data structure?
    inner: HashMap<Id, T>, 
    next_id: Id,
}

impl<T> HandleMap<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        if self.inner.insert(id, value).is_some() {
            panic!("Duplicate keys!");
        }
        id
    }

    pub fn remove(&mut self, id: &Id) {
        let _ = self.inner.remove(id);
        if *id >= self.next_id {
            panic!("Id did not originate from this map")
        }
    }
}

impl<T> Default for HandleMap<T> {
    fn default() -> Self {
        Self::new()
    }
}
