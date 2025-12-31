// TODO: Remove
#![allow(private_interfaces)]
#![allow(unused)]

use crate::disk::{Disk, Id};
use arrayvec::ArrayVec;
use std::marker::PhantomData;

pub struct BTreeMap<K, V> {
    root: Id,
    length: usize,
    marker: PhantomData<fn() -> Node<K, V>>,
}

impl<K, V> BTreeMap<K, V> {
    pub fn new(disk: &mut Disk<Node<K, V>>) -> Self {
        Self {
            root: disk.alloc(Node::Leaf(NodeLeaf {
                keys: ArrayVec::new(),
                values: ArrayVec::new(),
            })),
            length: 0,
            marker: PhantomData,
        }
    }

    pub fn insert(&mut self, disk: &mut Disk<Node<K, V>>, key: K, value: V) -> Option<V> {
        todo!()
    }

    pub fn remove(&mut self, disk: &mut Disk<Node<K, V>>, key: &K) -> Option<V> {
        todo!()
    }

    pub fn get(&self, disk: &Disk<Node<K, V>>, key: &K) -> Option<&V> {
        todo!()
    }

    pub fn get_mut(&mut self, disk: &mut Disk<Node<K, V>>, key: &K) -> Option<&mut V> {
        todo!()
    }

    pub fn contains_key(&self, disk: &Disk<Node<K, V>>, key: &K) -> bool {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(not(test))]
const D: usize = 42; // TODO: Choose a real value
#[cfg(test)]
const D: usize = 2;

#[cfg_attr(not(test), expect(clippy::large_enum_variant))]
enum Node<K, V> {
    Branch(NodeBranch<K, V>),
    Leaf(NodeLeaf<K, V>),
}

impl<K, V> Node<K, V> {
    //
}

struct NodeBranch<K, V> {
    keys: ArrayVec<K, { (2 * D) + 1 }>,
    children: ArrayVec<Id, { (2 * D) + 2 }>,
    marker: PhantomData<fn() -> Node<K, V>>,
}

impl<K, V> NodeBranch<K, V> {
    //
}

struct NodeLeaf<K, V> {
    keys: ArrayVec<K, { (2 * D) + 1 }>,
    values: ArrayVec<V, { (2 * D) + 1 }>,
}

impl<K, V> NodeLeaf<K, V> {
    //
}
