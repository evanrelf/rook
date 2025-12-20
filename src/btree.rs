// TODO: Remove
#![allow(unused)]

use arrayvec::ArrayVec;
use std::{mem, sync::Arc};

pub struct BTreeMap<K, V> {
    root: Node<K, V>,
}

impl<K, V> BTreeMap<K, V> {
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.root.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.root.remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        self.root.get_mut(key)
    }

    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        self.root.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    fn assert_invariants(&self) {
        self.root.assert_invariants(0);
    }
}

impl<K, V> Default for BTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximum number of children a node can have
// TODO: Choose a real value
const M: usize = 42;

/// Minimum possible height of the tree
///
/// [Wikipedia > Tree (abstract data type) > Terminology][1] says "The height of a node is the
/// length of the longest downward path to a leaf from that node. The height of the root is the
/// height of the tree," and therefore "leaf nodes have height zero."
///
/// [1]: https://en.wikipedia.org/wiki/Tree_(abstract_data_type)#Terminology
const H_MIN: usize = 0;

/// Maximum possible height of the tree
///
/// I'm making an educated guess that the height will will never exceed 38 based on these sources:
///
/// - [Wikipedia > B-tree > Best case and worst case heights][1]
/// - [WolframAlpha > "limit of (floor(log base 2 of ((n + 1) / 2))) as n approaches 1 trillion"][2]
///
/// [1]: https://en.wikipedia.org/wiki/B-tree#Best_case_and_worst_case_heights
/// [2]: https://www.wolframalpha.com/input?i=limit+of+%28floor%28log+base+2+of+%28%28n+%2B+1%29+%2F+2%29%29%29+as+n+approaches+1+trillion
const H_MAX: usize = 38;

#[expect(clippy::large_enum_variant)]
enum Node<K, V> {
    Internal(NodeInternal<K, V>),
    Leaf(NodeLeaf<K, V>),
}

impl<K, V> Node<K, V> {
    fn new() -> Self {
        Self::Leaf(NodeLeaf {
            keys: ArrayVec::new(),
            values: ArrayVec::new(),
        })
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        todo!()
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        match self {
            Node::Internal(internal) => internal.get(key),
            Node::Leaf(leaf) => leaf.get(key),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        match self {
            Node::Internal(internal) => internal.get_mut(key),
            Node::Leaf(leaf) => leaf.get_mut(key),
        }
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        match self {
            Node::Internal(internal) => internal.contains_key(key),
            Node::Leaf(leaf) => leaf.contains_key(key),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Node::Internal(internal) => internal.is_empty(),
            Node::Leaf(leaf) => leaf.is_empty(),
        }
    }

    fn is_internal(&self) -> bool {
        matches!(self, Node::Internal(_))
    }

    fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    fn assert_invariants(&self, depth: u8) {
        match self {
            Node::Internal(internal) => internal.assert_invariants(depth),
            Node::Leaf(leaf) => leaf.assert_invariants(depth),
        }
    }
}

struct NodeInternal<K, V> {
    keys: ArrayVec<K, { M - 1 }>,
    children: ArrayVec<Arc<Node<K, V>>, M>,
}

impl<K, V> NodeInternal<K, V> {
    fn search(&self, key: &K) -> SearchResult
    where
        K: Ord,
    {
        todo!()
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        todo!()
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        todo!()
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        self.keys.binary_search(key).is_ok()
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn assert_invariants(&self, depth: u8) {
        // https://en.wikipedia.org/wiki/B-tree#Definition
        assert!(
            self.children.len() <= M,
            "1. Every node has at most m children."
        );
        let is_root = depth == 0;
        let is_leaf = false;
        assert!(
            is_root || is_leaf || self.children.len() >= M.div_ceil(2),
            "2. Every node, except for the root and the leaves, has at least ⌈m/2⌉ children."
        );
        assert!(
            !is_root || self.children.len() >= 2,
            "3. The root node has at least two children unless it is a leaf."
        );
        let leaf_count = self.children.iter().filter(|child| child.is_leaf()).count();
        assert!(
            leaf_count == 0 || leaf_count == self.children.len(),
            "4. All leaves appear on the same level."
        );
        assert!(
            self.children.len() == self.keys.len() - 1,
            "5. A non-leaf node with k children contains_key k−1 keys."
        );
        for child in &self.children {
            child.assert_invariants(depth + 1);
        }
    }
}

struct NodeLeaf<K, V> {
    keys: ArrayVec<K, M>,
    values: ArrayVec<V, M>,
}

impl<K, V> NodeLeaf<K, V> {
    fn search(&self, key: &K) -> SearchResult
    where
        K: Ord,
    {
        match self.keys.binary_search(key) {
            Ok(index) => SearchResult::Found(index),
            Err(index) if self.keys.is_full() => SearchResult::MissingFull,
            Err(index) => SearchResult::MissingCanInsert(index),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)>
    where
        K: Ord,
    {
        match self.search(&key) {
            SearchResult::Found(index) => {
                let previous_value = mem::replace(&mut self.values[index], value);
                Ok(Some(previous_value))
            }
            SearchResult::MissingCanInsert(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                Ok(None)
            }
            SearchResult::MissingFull => Err((key, value)),
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        match self.search(key) {
            SearchResult::Found(index) => {
                self.keys.remove(index);
                let value = self.values.remove(index);
                Some(value)
            }
            _ => None,
        }
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&self.values[index]),
            Err(_) => None,
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&mut self.values[index]),
            Err(_) => None,
        }
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        self.keys.binary_search(key).is_ok()
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn assert_invariants(&self, depth: u8) {
        assert!(self.keys.len() == self.values.len());
    }
}

enum SearchResult {
    Found(usize),
    MissingCanInsert(usize),
    MissingFull,
}
