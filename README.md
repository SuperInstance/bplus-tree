# B+ Tree

**B+ Tree** is a Rust library implementing the in-memory B+ tree data structure with variable-order internal and leaf nodes, providing O(log_b n) search and insertion with linked-leaf sequential access — the standard index structure used in database engines.

## Why It Matters

B+ trees are the backbone of virtually every database index: PostgreSQL, MySQL/InnoDB, SQLite, and Oracle all use B+ tree variants for their primary indexes. The key advantage over binary search trees is the high branching factor: with order b = 100, a B+ tree storing 1 billion keys has height only 4–5, meaning any lookup requires just 4–5 page reads. This is critical for disk-based storage where each page read is expensive. B+ trees also support efficient range queries via the linked leaf list, and their insertion behavior (split propagation) keeps the tree balanced without requiring rotations. This implementation provides the in-memory variant, suitable for use as a sorted dictionary in applications requiring ordered iteration and guaranteed logarithmic performance.

## How It Works

**Structure:** A B+ tree of order b stores keys in leaf nodes and routing keys in internal nodes:

- **Leaf nodes:** Contain sorted key-value pairs. Up to b−1 entries. Linked in a list left-to-right for range scans.
- **Internal nodes:** Contain routing keys and child pointers. Up to b−1 keys, b children.

**Search:** Traverse from root to leaf:
```
search(key):
  node = root
  while node is internal:
    i = first position where key < node.keys[i]
    node = node.children[i]
  return leaf[key]  // binary search within leaf
```
Complexity: O(log_b n) — with b = 4 (this impl) and n = 1M, depth ≈ 10.

**Insertion with splitting:**
```
insert(key, value):
  1. Find target leaf
  2. Insert key-value in sorted position
  3. If leaf has ≥ b entries:
     split at midpoint → (left, mid_key, right)
     propagate mid_key to parent
  4. If parent overflows: split recursively
  5. If root splits: create new root
```

Each split is O(b) for copying keys. Splits propagate up at most O(log_b n) levels. Total insert: O(b × log_b n).

**Comparison with alternatives:**

| Structure | Search | Insert | Range Query | Space |
|-----------|--------|--------|-------------|-------|
| B+ Tree (b=100) | O(log₁₀₀ n) | O(log₁₀₀ n) | O(k) via leaf list | ~n |
| AVL Tree | O(log₂ n) | O(log₂ n) | O(k + log n) | ~2n |
| Hash Table | O(1) avg | O(1) avg | O(n) | ~2n |
| Sorted Array | O(log n) | O(n) | O(k) | n |

B+ trees win on range queries (linked leaves), memory locality (cache-friendly node sizes), and disk alignment (node = page size).

## Quick Start

```rust
fn main() {
    let mut tree = BPlusTree::new();
    tree.insert(5, "five");
    tree.insert(3, "three");
    tree.insert(7, "seven");
    tree.insert(1, "one");
    tree.insert(9, "nine");

    assert_eq!(tree.get(&3), Some(&"three"));
    assert_eq!(tree.get(&9), Some(&"nine"));
    assert_eq!(tree.get(&4), None);
    println!("Tree size: {}", tree.len());
}
```

## API

| Method | Signature | Complexity |
|--------|-----------|------------|
| `BPlusTree::new` | `() → Self` | O(1) |
| `insert` | `(K, V) → ()` | O(b log_b n) |
| `get` | `(&K) → Option<&V>` | O(log_b n) |
| `len` | `() → usize` | O(1) |
| `is_empty` | `() → bool` | O(1) |

## Architecture Notes

The B+ Tree provides the **sorted index layer** for the SuperInstance fleet's observation database. Conservation-law observations (γ + η = C) are timestamped and stored in B+ trees, enabling efficient time-range queries: "show all avoidance-ratio measurements between time T₁ and T₂." The O(log_b n) search ensures that even with millions of observations, lookups complete in microseconds.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Bayer, R. & McCreight, E. (1972). "Organization and Maintenance of Large Ordered Indices." *Acta Informatica*, 1(3), 173–189.
2. Comer, D. (1979). "The Ubiquitous B-Tree." *ACM Computing Surveys*, 11(2), 121–137.
3. Graefe, G. (2010). "Modern B-Tree Techniques." *Foundations and Trends in Databases*, 3(4), 203–402.

## License

MIT
