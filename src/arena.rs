use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

pub struct Id<T> {
    index: u32,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        let _phantom = PhantomData;
        let index = self.index;
        Id { index, _phantom }
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.index.hash(h);
    }
}

pub struct Arena<T>(Vec<T>);

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena(vec![])
    }

    pub fn alloc(&mut self, value: T) -> Id<T> {
        let index = self.next_id();
        self.0.push(value);
        index
    }

    pub fn next_id(&self) -> Id<T> {
        let _phantom = PhantomData;
        let index = self.len() as u32;
        Id { index, _phantom }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> Index<Id<T>> for Arena<T> {
    type Output = T;
    fn index(&self, id: Id<T>) -> &Self::Output {
        &self.0[id.index as usize]
    }
}

impl<T> IndexMut<Id<T>> for Arena<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut Self::Output {
        &mut self.0[id.index as usize]
    }
}
