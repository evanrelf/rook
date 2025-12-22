#![allow(unused)] // TODO: Remove
#![allow(clippy::manual_div_ceil)] // Erroneous lint

use arrayvec::ArrayVec;
use std::{iter, mem, sync::Arc};

/// Ordered map based on an in-memory copy-on-write B+ tree
#[derive(Clone)]
pub struct BTreeMap<K, V> {
    root: Arc<Node<K, V>>,
}

impl<K, V> BTreeMap<K, V> {
    /// Create a new, empty map
    pub fn new() -> Self {
        Self {
            root: Arc::new(Node::new()),
        }
    }

    /// Insert an entry, returning the existing value if present
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        let root = Arc::make_mut(&mut self.root);
        match root.insert(key, value) {
            NodeInsertResult::Replaced(previous_value) => Some(previous_value),
            NodeInsertResult::Inserted => None,
            NodeInsertResult::Split(parent_key, child_right) => {
                let child_left = {
                    let parent = Node::Branch(NodeBranch {
                        keys: ArrayVec::new(),
                        children: ArrayVec::new(),
                    });
                    mem::replace(root, parent)
                };
                let child_left_max = &child_left.keys()[child_left.keys().len() - 1];
                let child_right_min = &child_right.keys()[0];
                assert!(child_left_max < child_right_min, "Left child < right child");
                assert!(*child_left_max < parent_key, "Left child max < parent key");
                assert!(
                    *child_right_min == parent_key,
                    "Right child min == parent key"
                );
                let Node::Branch(parent) = root else {
                    unreachable!()
                };
                parent.keys.push(parent_key);
                parent.children.push(Arc::new(child_left));
                parent.children.push(Arc::new(child_right));
                None
            }
        }
    }

    /// Remove an entry, returning the existing value if present
    ///
    /// We follow the advice of the ["Deletion without rebalancing in multiway search trees"][1]
    /// paper, which suggests "rebalancing on deletion not only is unnecessary but may be harmful."
    ///
    /// [1]: https://doi.org/10.1145/2540068
    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        Arc::make_mut(&mut self.root).remove(key)
    }

    /// Get a shared reference to the value associated with the given key
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.get(key)
    }

    /// Get an exclusive reference to the value associated with the given key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        Arc::make_mut(&mut self.root).get_mut(key)
    }

    /// Check whether the key is present in the map
    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        self.root.contains_key(key)
    }

    /// Check whether the map has any entries
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    fn assert_invariants(&self)
    where
        K: Ord,
    {
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
const H_MAX: usize = 38;
const _: () = assert!(M >= 4, "`H_MAX` assumes branching factor >= 4");

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
        let mut branches = ArrayVec::new();
        loop {
            match node {
                Node::Branch(branch) => {
                    let index = branch.search(key);
                    branches.push(index);
                    node = &branch.children[index];
                    continue;
                }
                Node::Leaf(leaf) => {
                    break NodeSearchResult {
                        branches,
                        leaf: leaf.search(key),
                    };
                }
            }
        }
    }

    fn insert(&mut self, key: K, value: V) -> NodeInsertResult<K, V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        match self {
            Node::Branch(branch) => branch.insert(key, value),
            Node::Leaf(leaf) => leaf.insert(key, value),
        }
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

    fn keys(&self) -> &[K] {
        match self {
            Node::Branch(branch) => branch.keys.as_slice(),
            Node::Leaf(leaf) => leaf.keys.as_slice(),
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

    fn assert_invariants(&self, depth: u8)
    where
        K: Ord,
    {
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
    branches: ArrayVec<usize, H_MAX>,
    leaf: LeafSearchResult,
}

#[cfg_attr(not(test), expect(clippy::large_enum_variant))]
enum NodeInsertResult<K, V> {
    Replaced(V),
    Inserted,
    Split(K, Node<K, V>),
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

    fn insert(&mut self, key: K, value: V) -> NodeInsertResult<K, V>
    where
        K: Clone + Ord,
        V: Clone,
    {
        let index = self.search(&key);
        let child = Arc::make_mut(&mut self.children[index]);
        match child.insert(key, value) {
            NodeInsertResult::Split(key, child) => {
                todo!()
            }
            no_split => no_split,
        }
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

    fn assert_invariants(&self, depth: u8)
    where
        K: Ord,
    {
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
        // TODO: Assert ordering is correct (e.g. keys to the left are less than). Could pass child
        // bounds and have it check itself.
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
            Err(index) => LeafSearchResult::Missing(index),
        }
    }

    fn insert(&mut self, key: K, value: V) -> NodeInsertResult<K, V>
    where
        K: Clone + Ord,
    {
        match self.search(&key) {
            LeafSearchResult::Found(index) => {
                let previous_value = mem::replace(&mut self.values[index], value);
                NodeInsertResult::Replaced(previous_value)
            }
            LeafSearchResult::Missing(index) => {
                if !self.is_full() {
                    self.keys.insert(index, key);
                    self.values.insert(index, value);
                    return NodeInsertResult::Inserted;
                }
                let self_len = (M + 1).div_ceil(2);
                let sibling_len = (M + 1) / 2;
                let mut sibling = Self {
                    keys: self.keys.drain(self_len..).collect(),
                    values: self.values.drain(self_len..).collect(),
                };
                assert!(matches!(
                    sibling.insert(key, value),
                    NodeInsertResult::Inserted
                ));
                let parent_key = sibling.keys[0].clone();
                assert_eq!(self.keys.len(), self_len);
                assert_eq!(sibling.keys.len(), sibling_len);
                NodeInsertResult::Split(parent_key, Node::Leaf(sibling))
            }
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
        self.keys.len() == M
    }

    fn is_overflowing(&self) -> bool {
        self.keys.is_full()
    }

    fn assert_invariants(&self, depth: u8)
    where
        K: Ord,
    {
        assert!(self.keys.len() == self.values.len(), "All keys have values");
        assert!(self.keys.len() <= M, "Not overflowing");
        assert!(self.keys.is_sorted(), "Keys are sorted");
    }
}

impl<K, V> Default for NodeLeaf<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

enum LeafSearchResult {
    Found(usize),
    Missing(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_after_split() {
        // NOTE: `div_floor` is unstable, so we use `/` which does the same thing.
        let m = 4usize;
        assert_eq!((m + 1).div_ceil(2), 3);
        assert_eq!((m + 1) / 2, 2);
        let m = 5usize;
        assert_eq!((m + 1).div_ceil(2), 3);
        assert_eq!((m + 1) / 2, 3);
    }

    #[test]
    fn leaf_no_split() {
        let mut leaf = NodeLeaf::new();

        assert!(matches!(leaf.insert(1, 11), NodeInsertResult::Inserted));
        assert!(matches!(leaf.insert(2, 22), NodeInsertResult::Inserted));
        assert!(matches!(leaf.insert(3, 33), NodeInsertResult::Inserted));
        assert!(matches!(leaf.insert(4, 44), NodeInsertResult::Inserted));

        assert_eq!(leaf.get(&1), Some(&11));
        *leaf.get_mut(&2).unwrap() = 29;
        assert_eq!(leaf.get(&2), Some(&29));
        assert!(leaf.contains_key(&3));

        assert_eq!(leaf.remove(&3), Some(33));
        assert!(!leaf.contains_key(&3));
        assert_eq!(leaf.get(&3), None);
    }

    #[test]
    fn leaf_split() {
        let mut leaf1 = NodeLeaf::new();

        assert!(matches!(leaf1.insert(1, 11), NodeInsertResult::Inserted));
        assert!(matches!(leaf1.insert(2, 22), NodeInsertResult::Inserted));
        assert!(matches!(leaf1.insert(3, 33), NodeInsertResult::Inserted));
        assert!(matches!(leaf1.insert(4, 44), NodeInsertResult::Inserted));
        let NodeInsertResult::Split(parent_key, Node::Leaf(leaf2)) = leaf1.insert(5, 55) else {
            panic!();
        };

        assert_eq!(leaf1.keys.as_slice(), &[1, 2, 3]);
        assert_eq!(leaf1.values.as_slice(), &[11, 22, 33]);

        assert_eq!(parent_key, 4);

        assert_eq!(leaf2.keys.as_slice(), &[4, 5]);
        assert_eq!(leaf2.values.as_slice(), &[44, 55]);
    }

    #[test]
    fn tree_leaf_split() {
        let mut tree = BTreeMap::new();
        assert!(tree.insert(1, 11).is_none());
        assert!(tree.insert(2, 22).is_none());
        assert!(tree.insert(3, 33).is_none());
        assert!(tree.insert(4, 44).is_none());
        assert!(tree.root.is_leaf());
        assert!(tree.insert(5, 55).is_none());
        assert!(tree.root.is_branch());
    }
}
