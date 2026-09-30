//! B+ Tree implementation (in-memory, variable order)
const ORDER: usize = 4;

#[derive(Debug)]
enum BPlusNode<K: Ord + Clone, V> {
    Internal {
        keys: Vec<K>,
        children: Vec<Box<BPlusNode<K, V>>>,
    },
    Leaf {
        keys: Vec<K>,
        values: Vec<V>,
    },
}

#[derive(Debug)]
pub struct BPlusTree<K: Ord + Clone, V> {
    root: Option<Box<BPlusNode<K, V>>>,
    len: usize,
}

impl<K: Ord + Clone, V> BPlusTree<K, V> {
    pub fn new() -> Self { BPlusTree { root: None, len: 0 } }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn insert(&mut self, key: K, value: V) {
        if self.root.is_none() {
            self.root = Some(Box::new(BPlusNode::Leaf {
                keys: vec![key],
                values: vec![value],
            }));
            self.len += 1;
            return;
        }
        let root = self.root.take().unwrap();
        match Self::insert_rec(root, key, value) {
            SplitResult::NoSplit(node) => { self.root = Some(node); self.len += 1; }
            SplitResult::Split(left, mid_key, right) => {
                self.root = Some(Box::new(BPlusNode::Internal {
                    keys: vec![mid_key],
                    children: vec![left, right],
                }));
                self.len += 1;
            }
        }
    }

    fn insert_rec(node: Box<BPlusNode<K, V>>, key: K, value: V) -> SplitResult<K, V> {
        match *node {
            BPlusNode::Leaf { mut keys, mut values } => {
                let pos = keys.iter().position(|k| key <= *k).unwrap_or(keys.len());
                if pos < keys.len() && keys[pos] == key {
                    values[pos] = value;
                    return SplitResult::NoSplit(Box::new(BPlusNode::Leaf { keys, values }));
                }
                keys.insert(pos, key);
                values.insert(pos, value);
                if keys.len() < ORDER {
                    SplitResult::NoSplit(Box::new(BPlusNode::Leaf { keys, values }))
                } else {
                    let mid = ORDER / 2;
                    let right_keys = keys.split_off(mid);
                    let right_vals = values.split_off(mid);
                    let split_key = right_keys[0].clone();
                    SplitResult::Split(
                        Box::new(BPlusNode::Leaf { keys, values }),
                        split_key,
                        Box::new(BPlusNode::Leaf { keys: right_keys, values: right_vals }),
                    )
                }
            }
            BPlusNode::Internal { mut keys, mut children } => {
                let pos = keys.iter().position(|k| key < *k).unwrap_or(children.len());
                let child = children.remove(pos);
                match Self::insert_rec(child, key, value) {
                    SplitResult::NoSplit(new_child) => {
                        children.insert(pos, new_child);
                        SplitResult::NoSplit(Box::new(BPlusNode::Internal { keys, children }))
                    }
                    SplitResult::Split(left_child, mid_key, right_child) => {
                        keys.insert(pos, mid_key);
                        children.insert(pos, left_child);
                        children.insert(pos + 1, right_child);
                        if keys.len() < ORDER {
                            SplitResult::NoSplit(Box::new(BPlusNode::Internal { keys, children }))
                        } else {
                            let mid = keys.len() / 2;
                            let right_keys = keys.split_off(mid + 1);
                            let right_children = children.split_off(mid + 1);
                            let up_key = keys.pop().unwrap();
                            SplitResult::Split(
                                Box::new(BPlusNode::Internal { keys, children }),
                                up_key,
                                Box::new(BPlusNode::Internal { keys: right_keys, children: right_children }),
                            )
                        }
                    }
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        Self::search(self.root.as_deref(), key)
    }

    fn search<'a>(node: Option<&'a BPlusNode<K, V>>, key: &K) -> Option<&'a V> {
        let n = node?;
        match n {
            BPlusNode::Leaf { keys, values } => {
                keys.iter().position(|k| k == key).map(|i| &values[i])
            }
            BPlusNode::Internal { keys, children } => {
                let pos = keys.iter().position(|k| key < k).unwrap_or(children.len());
                Self::search(Some(children[pos].as_ref()), key)
            }
        }
    }
}

enum SplitResult<K: Ord + Clone, V> {
    NoSplit(Box<BPlusNode<K, V>>),
    Split(Box<BPlusNode<K, V>>, K, Box<BPlusNode<K, V>>),
}

impl<K: Ord + Clone, V> Default for BPlusTree<K, V> {
    fn default() -> Self { Self::new() }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
