# bplus-tree

An in-memory B+ tree in 130 lines of Rust.

It's the smallest B+ tree I could write that still does the structural
trick — internal nodes that route, leaf nodes that store, splits that
bubble up — without pulling in the rest of a database. There's a `get`
and an `insert`. There's no `delete`, no range scan, no iterator. What
there is, is the bit of a B+ tree that the textbook figures are trying
to show you, runnable in a Rust playground.

## Why a B+ tree

A binary search tree is fine for a thousand keys and terrible for a
million: the depth grows with `log₂ n`, and each level costs a cache
miss. The trick of a B+ tree — going back to Bayer and McCreight's 1972
paper — is to make each node hold many keys and many children, so the
depth grows with `log_m n` where `m` is the node's branching factor
(here, `ORDER = 4`). With `m = 100`, a billion-key tree is at most 3
levels deep, and the inner loop on a search touches 3 cache lines.

It's the index structure on which most of the world's relational
databases are built (with the actual `m` determined by the page size:
Postgres uses 8KB pages, so `m` is in the low hundreds for `int`
keys). It's the right answer when you have a working set that doesn't
fit in memory, but you still want `O(log n)` lookups, inserts, and
range scans.

## A first example

```rust
use bplus_tree::BPlusTree;

let mut t = BPlusTree::new();
t.insert(10, "ten");
t.insert(3, "three");
t.insert(7, "seven");
t.insert(15, "fifteen");
t.insert(1, "one");

assert_eq!(t.get(&3), Some(&"three"));
assert_eq!(t.len(), 5);
```

The tree holds `K: Ord + Clone` and `V` — no lifetimes, no allocators
beyond `Box`, no async. The `ORDER` is fixed at 4 in the source (a
`const`, not a parameter), which keeps the implementation small but
also means the fanout is a teaching value, not a tunable. See
*"Things to try"* below for why this is a feature, not a bug, if you're
trying to *see* how the structure works.

A note on `get`: the search routine is correct for keys that exist in
the tree, but the "key larger than all in the tree" path triggers an
off-by-one panic in the current implementation (it indexes
`children[children.len()]` when the key falls past the last separator).
If you want a robust `get(&missing_key)`, see *"Things to try"*. The
inserts above all work; it's only "search for a value bigger than
anything in the tree" that needs the fix.

## What's in the box

```
src/
  lib.rs   one file, one struct, one recursive descent
```

The whole implementation lives in `lib.rs` and is one `impl` block. The
recursive structure of the public API matches the recursive structure
of the algorithm — `insert` calls `insert_rec` on the appropriate child;
if the child splits, the parent inserts the median key and may itself
split. The recursion bottoms out at the leaves.

The two node types:

```rust
enum BPlusNode<K, V> {
    Internal { keys: Vec<K>, children: Vec<Box<...>> },
    Leaf     { keys: Vec<K>, values: Vec<V> },
}
```

Internal nodes carry keys but no values — they exist only to route
lookups to the right child. Leaf nodes carry both, and in a
full B+ tree implementation, leaves are linked to each other for
range scans. The linking is the one piece conspicuously absent here,
and there's a comment below.

## The split, the only hard part

The only operation that requires real care is the split, and the code
is short enough to read end-to-end. When a leaf exceeds `ORDER` keys,
it splits at `ORDER / 2` and pushes the first key of the right half
upward. When an internal node exceeds `ORDER` keys, it splits at
`keys.len() / 2 + 1` and pushes *the key at that index* upward — not
the first key of the right half. That asymmetry is a frequent source
of bugs in hand-written B+ trees and it's worth noticing.

A reproduction of the relevant block from `lib.rs`:

```rust
let mid = ORDER / 2;
let right_keys = keys.split_off(mid);
let right_vals = values.split_off(mid);
let split_key = right_keys[0].clone();   // first key of right half
SplitResult::Split(
    Box::new(BPlusNode::Leaf { keys, values }),
    split_key,
    Box::new(BPlusNode::Leaf { keys: right_keys, values: right_vals }),
)
```

If you trace through inserting 1, 3, 7, 10, 15 into a fresh tree, you
can see the leaf fill, then split, then the new root absorb the
median. The same thing in a B-tree (where internal nodes also store
values) is one line different; that one line is the whole reason B+
trees are the right structure for databases that do range scans.

## Things to try

1. **Set `ORDER = 2` and see what happens.** The tree degenerates to
   effectively a binary tree. The split logic still works; the shape
   just isn't useful. This is the simplest possible way to convince
   yourself that the branching factor is what makes the structure
   interesting.
2. **Insert 1000 sequential keys and instrument the depth.** Add a
   `fn depth(&self) -> usize` that walks to the leftmost leaf, and
   watch how it grows with `ORDER`. With `ORDER = 4`, depth should
   grow like `log₄ n`, and you can plot the `d` of the tree against
   the `log₄ n` line — they should be within a small constant of
   each other.
3. **Implement `delete`.** This is where B+ trees get genuinely
   difficult. The `delete` operation has to handle three cases
   (underflow at a leaf, underflow at an internal node, and the
   "redistribute vs merge" choice), and the textbook is not
   charitable about which. It's a good exercise: the *insert* side
   of this library is ~70 lines, and a correct *delete* is
   comparable, but you can lose an afternoon to off-by-one
   rebalancing.
4. **Add leaf linking and a `range` iterator.** This is the change
   that makes a B+ tree a B+ tree for database purposes. The
   `Leaf` variant grows a `next: Option<Box<BPlusNode<K, V>>>`
   field, splits update the chain, and `range(from, to)` is a
   straightforward walk. Doing this is a satisfying afternoon.

## Where this comes from

- The original paper: Rudolf Bayer and Edward McCreight,
  *Organization and Maintenance of Large Ordered Indexes*, 1972.
  The data structure is sometimes called a "Bayer tree" for that
  reason. The paper is short and clear; Acta Informatica vol. 1,
  pages 173–189.
- For a database-flavored read, the classic chapter is
  *Database Systems: The Complete Book* by Garcia-Molina, Ullman,
  and Widom, §10. The chapter on tree-structured indexes has
  clean figures that match what this code does.
- For a C-flavored reading of a B+ tree that is just barely larger
  than this one, see the SQLite source file `btree.c`. The same
  recursion is there, with page-level I/O and locking on top.

## What this isn't

- It's not persistent. The tree lives in process memory; there's no
  disk backend, no WAL, no fsync. If you want a B+ tree that
  survives a crash, look at sled, lmdb, or sqlite.
- The `ORDER` is a hard-coded `const`, not a generic parameter.
  This was a deliberate choice to keep the code small. Adding
  `<const ORDER: usize>` is a five-line change.
- There's no `delete`, no `range` scan, no iterator. The
  structure supports them (see *Things to try*); the code in this
  crate does not.
- The `Default` impl exists, but `BPlusTree::default()` is the
  same as `BPlusTree::new()`. If you wanted a `BPlusTree::with_order(n)`
  builder, that's a TODO.
- This is an in-memory structure with a recursive descent.
  Recursion depth is bounded by `log_ORDER(n)`, which is fine
  for any realistic `n` in memory, but it does mean a future
  iterator-based rewrite would be more cache-friendly.

## License

MIT OR Apache-2.0.
