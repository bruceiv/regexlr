// RegexLR Parser Generator
// Copyright (C) 2025  Aaron Moss
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! An append-only pool of a given type, with a typed index

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// An indexed append-only collection.
pub struct Pool<Element> {
    items: Vec<Element>,
}

impl<Element> Pool<Element> {
    /// Set up a new pool
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a new element to the pool.
    /// Return the index of the added production.
    pub fn insert(&mut self, e: Element) -> Ind<Element> {
        self.items.push(e);
        Ind::<Element>::of(self.items.len() - 1)
    }
}

impl<Element> Index<Ind<Element>> for Pool<Element> {
    type Output = Element;

    fn index(&self, i: Ind<Element>) -> &Self::Output {
        &self.items[i.to_usize()]
    }
}

impl<Element> IndexMut<Ind<Element>> for Pool<Element> {
    fn index_mut(&mut self, i: Ind<Element>) -> &mut Self::Output {
        &mut self.items[i.to_usize()]
    }
}

impl<Element: fmt::Debug> fmt::Debug for Pool<Element> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.items.iter().enumerate())
            .finish()
    }
}

/// An index into a [Pool<Element>]
pub struct Ind<Element>(u32, PhantomData<*const Element>);

impl<Element> Ind<Element> {
    /// construct from index
    fn of(i: usize) -> Self {
        Ind(i as u32, PhantomData)
    }

    /// export to index
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl<Element> Clone for Ind<Element> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<Element> Copy for Ind<Element> {}

impl<Element> PartialEq for Ind<Element> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Element> Eq for Ind<Element> {}

impl<Element> Hash for Ind<Element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<Element> fmt::Debug for Ind<Element> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A deduplicated append-only collection.
/// Uses a hashtable to ensure uniqueness of pool contents.
pub struct UniquePool<Element> {
    /// element pool; owning storage for contained elements
    items: Vec<Element>,
    /// index into element pool of hash-table entry
    index: UniquePoolIndex,
}

/// Maximum load factor for [UniquePool]
const MAX_LOAD: f32 = 0.9;

impl<Element: Eq + Hash> UniquePool<Element> {
    /// Creates a new pool.
    /// Pool is initialized empty
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index: UniquePoolIndex::new(),
        }
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of elements in the pool
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Expand current pool capacity if necessary
    fn expand_capacity_if_needed(&mut self) {
        // return early if no need to expand
        let old_cap = self.index.capacity();
        if (self.len() + 1) <= ((old_cap as f32 * MAX_LOAD) as usize) {
            return;
        }

        // set up new index and re-hash
        let new_cap = if old_cap == 0 { 8 } else { old_cap << 1 };
        let mut new_index = UniquePoolIndex::with_capacity(new_cap);
        for entry in &self.index {
            new_index.insert(entry.hash, entry.ind);
        }
        self.index = new_index;
    }

    /// Inserts a new element.
    /// Returns the new index, or the previous index if already present.
    pub fn insert(&mut self, key: Element) -> Ind<Element> {
        let h = hash(&key);

        // already present, return
        if let Some(ind) = self.find_with_hash(&key, h) {
            return ind;
        }

        // needs to be inserted
        self.expand_capacity_if_needed();
        let ind = Ind::<Element>::of(self.items.len());
        self.items.push(key);
        self.index.insert(h, ind.0);
        ind
    }

    /// Finds an already-hashed element
    fn find_with_hash(&self, key: &Element, h: u64) -> Option<Ind<Element>> {
        // break early on empty
        if self.is_empty() {
            return None;
        }

        // starting index for search
        let (mut i, _) = self.index.split_hash(h);
        // probe count
        let mut probe = 1;

        // search for hash matches
        while let Some(j) = self.index.find(h, i, &mut probe) {
            // found if keys equal
            if key == self.key_at_hash_entry(j) {
                return Some(Ind::<Element>::of(j));
            }
            // keep searching if not
            i = j;
        }

        // nothing found
        None
    }

    /// Gets a reference to the element key at hash entry `i`
    fn key_at_hash_entry(&self, i: usize) -> &Element {
        &self.items[self.index.indices[i] as usize]
    }
}

impl<Element: fmt::Debug> fmt::Debug for UniquePool<Element> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.items.iter().enumerate())
            .finish()
    }
}

impl<Element> Index<Ind<Element>> for UniquePool<Element> {
    type Output = Element;

    fn index(&self, i: Ind<Element>) -> &Self::Output {
        &self.items[i.to_usize()]
    }
}

/// Utility method to get the hash of a given key
fn hash<Key: Hash>(key: &Key) -> u64 {
    let mut state = DefaultHasher::new();
    key.hash(&mut state);
    state.finish()
}

/// The hash-index for a [UniquePool].
/// All the internal vectors are the same size and elements at the same index in each correspond.
struct UniquePoolIndex {
    /// index into element vector of [UniquePool]
    indices: Vec<u32>,
    /// hash of element in [UniquePool]
    hashes: Vec<u64>,
    /// probe count of entry; 0 for not filled
    probes: Vec<u8>,
    /// maximum probe count in index
    max_probe: u8,
}

impl UniquePoolIndex {
    /// Creates a new, empty index
    fn new() -> Self {
        Self {
            indices: Vec::new(),
            hashes: Vec::new(),
            probes: Vec::new(),
            max_probe: 0,
        }
    }

    /// Creates a new index with the given capacity.
    /// Capacity must be a power of two
    fn with_capacity(cap: usize) -> Self {
        assert!(cap.is_power_of_two());
        let mut ret = Self {
            indices: Vec::with_capacity(cap),
            hashes: Vec::with_capacity(cap),
            probes: Vec::with_capacity(cap),
            max_probe: 0,
        };
        ret.indices.resize(cap, 0);
        ret.hashes.resize(cap, 0);
        ret.probes.resize(cap, 0);
        ret
    }

    /// Checks if this index is empty
    fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Gets the capacity of this index
    fn capacity(&self) -> usize {
        self.indices.len()
    }

    /// Looks up a hash in the index.
    /// Starts at the given index `i` and probe count `probe`; returns `Some(ind)` if hash is found
    /// or `None` if hash is not present
    fn find(&self, h: u64, mut i: usize, probe: &mut u8) -> Option<usize> {
        // Not present if empty
        if self.is_empty() {
            return None;
        }

        // search through, not exceeding maximum probe count
        let (_, stride) = self.split_hash(h);
        while *probe <= self.max_probe && self.probes[i] > 0 {
            if self.hashes[i] == h {
                return Some(i);
            }

            self.next_index(&mut i, stride, probe);
        }

        // not found
        None
    }

    /// Insert a new element.
    /// Must have already determined element not present and sufficient capacity available.
    fn insert(&mut self, mut h: u64, mut ind: u32) {
        // current hash-entry index and stride
        let (mut i, mut stride) = self.split_hash(h);
        // count of probes
        let mut probe = 1;

        // search for location where insert can end
        loop {
            // search for "wealthy" tenant to evict
            while self.probes[i] > probe {
                self.next_index(&mut i, stride, &mut probe);
            }

            // found empty location
            if self.probes[i] == 0 {
                break;
            }

            // otherwise evict "wealthy" tenant and continue loop
            let (h2, ind2, probe2) = (self.hashes[i], self.indices[i], self.probes[i]);
            let (_, stride2) = self.split_hash(h2);

            self.set_hash_entry(i, ind, h, probe);

            h = h2;
            ind = ind2;
            probe = probe2;
            stride = stride2;
            self.next_index(&mut i, stride, &mut probe);
        }

        // insert new element in empty slot
        self.set_hash_entry(i, ind, h, probe);
    }

    /// Split a hash into an index modulo the current size of the pool and a stride.
    /// The stride will always be relatively prime to the size of the pool.
    /// Safe to index by, as long as the pool is non-empty and a power-of-two capacity.
    /// Must not be called on empty pool.
    fn split_hash(&self, h: u64) -> (usize, usize) {
        let cap = self.capacity();
        debug_assert!(cap != 0 && cap.is_power_of_two());
        // all zeros followed by all ones for power-of-two size
        let low_mask = cap - 1;
        // same, reversed
        let high_mask = !low_mask;

        let hu = h as usize;
        // valid index into the hashes/probes vector
        let index = hu & low_mask;
        // stride into the hashes/probes vector (odd number)
        let stride = ((hu & high_mask) >> (cap.ilog2() - 1)) | 0x1;
        (index, stride)
    }

    /// Updates the probe index and count.
    /// Guaranteed to be valid index into on non-empty pool.
    fn next_index(&self, i: &mut usize, stride: usize, probe: &mut u8) {
        // add the stride, mask off bits above power-of-two size
        *i = i.wrapping_add(stride) & self.capacity().wrapping_sub(1);
        *probe = probe.saturating_add(1);
    }

    /// Sets the hash-entry at a given index
    fn set_hash_entry(&mut self, i: usize, ind: u32, h: u64, probe: u8) {
        self.indices[i] = ind;
        self.hashes[i] = h;
        self.probes[i] = probe;
        if self.max_probe < probe {
            self.max_probe = probe;
        }
    }
}

/// Iterator entry for [UniquePoolIndex]
struct UniquePoolIndexEntry {
    /// element key
    ind: u32,
    /// element hash
    hash: u64,
}

/// Iterator for [UniquePoolIndex]
struct UniquePoolIndexIter<'pool> {
    /// Pool being indexed
    index: &'pool UniquePoolIndex,
    /// current iteration index
    i: usize,
}

impl<'pool> Iterator for UniquePoolIndexIter<'pool> {
    /// lookup index, hash, probe
    type Item = UniquePoolIndexEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // loop through table
        while self.i < self.index.indices.len() {
            // return first non-empty probe
            if self.index.probes[self.i] > 0 {
                let ret = UniquePoolIndexEntry {
                    ind: self.index.indices[self.i],
                    hash: self.index.hashes[self.i],
                };
                self.i += 1;
                return Some(ret);
            }
            self.i += 1;
        }
        // not found
        None
    }
}

impl<'pool> IntoIterator for &'pool UniquePoolIndex {
    type Item = UniquePoolIndexEntry;

    type IntoIter = UniquePoolIndexIter<'pool>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { index: self, i: 0 }
    }
}
