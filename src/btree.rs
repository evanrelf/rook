#![allow(unused)] // TODO: Remove

use arrayvec::ArrayVec;
use std::{mem, sync::Arc};

/// Ordered map based on a copy-on-write B+ tree
#[derive(Clone)]
pub struct BTreeMap<K, V> {
    root: Arc<Node<K, V>>,
}

impl<K, V> BTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            root: Arc::new(Node::new()),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        Arc::make_mut(&mut self.root).insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        Arc::make_mut(&mut self.root).remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        Arc::make_mut(&mut self.root).get_mut(key)
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

/// Branching factor of the tree
///
/// Also represents the maximum number of children a node can have.
#[cfg(not(test))]
const M: usize = 42; // TODO: Choose a real value
#[cfg(test)]
const M: usize = 4;

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
const H_MAX: usize = {
    assert!(M >= 4, "`H_MAX` assumes branching factor >= 4");
    38
};

#[cfg_attr(not(test), expect(clippy::large_enum_variant))]
#[derive(Clone)]
enum Node<K, V> {
    Branch(NodeBranch<K, V>),
    Leaf(NodeLeaf<K, V>),
}

impl<K, V> Node<K, V> {
    fn new() -> Self {
        Self::Leaf(NodeLeaf::new())
    }

    fn search(&self, key: &K) -> NodeSearchResult
    where
        K: Ord,
    {
        let mut node = self;
        let mut branch_indices = ArrayVec::new();
        loop {
            match node {
                Node::Branch(branch) => {
                    let index = branch.search(key);
                    branch_indices.push(index);
                    node = &branch.children[index];
                    continue;
                }
                Node::Leaf(leaf) => {
                    break NodeSearchResult {
                        branch_indices,
                        leaf_index: leaf.search(key),
                    };
                }
            }
        }
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
            Node::Branch(branch) => branch.get(key),
            Node::Leaf(leaf) => leaf.get(key),
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        match self {
            Node::Branch(branch) => branch.get_mut(key),
            Node::Leaf(leaf) => leaf.get_mut(key),
        }
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        match self {
            Node::Branch(branch) => branch.contains_key(key),
            Node::Leaf(leaf) => leaf.contains_key(key),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Node::Branch(branch) => branch.is_empty(),
            Node::Leaf(leaf) => leaf.is_empty(),
        }
    }

    fn is_full(&self) -> bool {
        match self {
            Node::Branch(branch) => branch.is_full(),
            Node::Leaf(leaf) => leaf.is_full(),
        }
    }

    fn is_branch(&self) -> bool {
        matches!(self, Node::Branch(_))
    }

    fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    fn assert_invariants(&self, depth: u8) {
        match self {
            Node::Branch(branch) => branch.assert_invariants(depth),
            Node::Leaf(leaf) => leaf.assert_invariants(depth),
        }
    }
}

impl<K, V> Default for Node<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

struct NodeSearchResult {
    branch_indices: ArrayVec<usize, H_MAX>,
    leaf_index: LeafSearchResult,
}

#[derive(Clone)]
struct NodeBranch<K, V> {
    keys: ArrayVec<K, { M - 1 }>,
    children: ArrayVec<Arc<Node<K, V>>, M>,
}

impl<K, V> NodeBranch<K, V> {
    fn search(&self, key: &K) -> usize
    where
        K: Ord,
    {
        self.keys
            .iter()
            .position(|k| k > key)
            .unwrap_or(self.keys.len() - 1)
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
        let index = self.search(key);
        self.children[index].get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        let index = self.search(key);
        Arc::make_mut(&mut self.children[index]).get_mut(key)
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        let index = self.search(key);
        self.children[index].contains_key(key)
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn is_full(&self) -> bool {
        self.keys.is_full()
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

#[derive(Clone)]
struct NodeLeaf<K, V> {
    keys: ArrayVec<K, M>,
    values: ArrayVec<V, M>,
}

impl<K, V> NodeLeaf<K, V> {
    fn new() -> Self {
        Self {
            keys: ArrayVec::new(),
            values: ArrayVec::new(),
        }
    }

    fn search(&self, key: &K) -> LeafSearchResult
    where
        K: Ord,
    {
        match self.keys.binary_search(key) {
            Ok(index) => LeafSearchResult::Found(index),
            Err(index) if self.is_full() => LeafSearchResult::MissingFull,
            Err(index) => LeafSearchResult::MissingCanInsert(index),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)>
    where
        K: Ord,
    {
        match self.search(&key) {
            LeafSearchResult::Found(index) => {
                let previous_value = mem::replace(&mut self.values[index], value);
                Ok(Some(previous_value))
            }
            LeafSearchResult::MissingCanInsert(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                Ok(None)
            }
            LeafSearchResult::MissingFull => Err((key, value)),
        }
    }

    fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        match self.search(key) {
            LeafSearchResult::Found(index) => {
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
        match self.search(key) {
            LeafSearchResult::Found(index) => Some(&self.values[index]),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        match self.search(key) {
            LeafSearchResult::Found(index) => Some(&mut self.values[index]),
            _ => None,
        }
    }

    fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        matches!(self.search(key), LeafSearchResult::Found(_))
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn is_full(&self) -> bool {
        self.keys.is_full()
    }

    fn assert_invariants(&self, depth: u8) {
        assert!(self.keys.len() == self.values.len());
    }
}

impl<K, V> Default for NodeLeaf<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

enum LeafSearchResult {
    Found(usize),
    MissingCanInsert(usize),
    MissingFull,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf() {
        let mut leaf = NodeLeaf::new();

        assert!(leaf.insert(1, 11).is_ok());
        assert!(leaf.insert(2, 22).is_ok());
        assert!(leaf.insert(3, 33).is_ok());
        assert!(leaf.insert(4, 44).is_ok());
        assert!(leaf.insert(5, 55).is_err());

        assert_eq!(leaf.get(&1), Some(&11));
        assert_eq!(leaf.get(&2), Some(&22));
        assert!(leaf.contains_key(&3));

        assert_eq!(leaf.remove(&3), Some(33));
        assert!(!leaf.contains_key(&3));
        assert_eq!(leaf.get(&3), None);
    }
}
