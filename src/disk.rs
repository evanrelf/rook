use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Id(usize);

#[derive(Debug)]
pub struct Disk<T> {
    items: Vec<T>,
    free: BTreeSet<Id>,
}

impl<T> Disk<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            free: BTreeSet::new(),
        }
    }

    pub fn alloc(&mut self, item: T) -> Id {
        if let Some(id) = self.free.pop_first() {
            self.items[id.0] = item;
            id
        } else {
            self.items.push(item);
            Id(self.items.len() - 1)
        }
    }

    pub fn free(&mut self, id: Id) {
        self.free.insert(id);
    }

    pub fn get(&self, id: Id) -> &T {
        &self.items[id.0]
    }

    pub fn get_mut(&mut self, id: Id) -> &mut T {
        &mut self.items[id.0]
    }
}

impl<T> Default for Disk<T> {
    fn default() -> Self {
        Self::new()
    }
}
