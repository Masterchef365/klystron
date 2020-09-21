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

    pub fn get(&self, id: &Id) -> Option<&T> {
        self.inner.get(id)
    }

    pub fn remove(&mut self, id: &Id) -> Option<T> {
        self.inner.remove(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Id, &T)> {
        self.inner.iter()
    }

    #[allow(unused)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Id, &mut T)> {
        self.inner.iter_mut()
    }

    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = (Id, T)> + 'a {
        self.inner.drain()
    }
}

impl<T> Default for HandleMap<T> {
    fn default() -> Self {
        Self::new()
    }
}
