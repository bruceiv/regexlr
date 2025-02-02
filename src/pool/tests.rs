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

//! Tests for [pool] module

use std::fmt::Debug;

use bit_set::BitSet;

use super::*;

/// List of strings to test [Pool]
fn strings_to_ten() -> Vec<&'static str> {
    vec![
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ]
}

/// List of bitsets to test [Pool]
fn bitsets_to_twenty() -> Vec<BitSet> {
    vec![
        BitSet::new(),
        BitSet::from_bytes(&[0x1]),
        BitSet::from_bytes(&[0x2]),
        BitSet::from_bytes(&[0x3]),
        BitSet::from_bytes(&[0x4]),
        BitSet::from_bytes(&[0x5]),
        BitSet::from_bytes(&[0x6]),
        BitSet::from_bytes(&[0x7]),
        BitSet::from_bytes(&[0x8]),
        BitSet::from_bytes(&[0x9]),
        BitSet::from_bytes(&[0xA]),
        BitSet::from_bytes(&[0xB]),
        BitSet::from_bytes(&[0xC]),
        BitSet::from_bytes(&[0xD]),
        BitSet::from_bytes(&[0xE]),
        BitSet::from_bytes(&[0xF]),
        BitSet::from_bytes(&[0x10]),
        BitSet::from_bytes(&[0x11]),
        BitSet::from_bytes(&[0x12]),
        BitSet::from_bytes(&[0x13]),
    ]
}

/// Test that a pool accurately creates its items
fn test_pool<Element: Clone + Debug + Eq>(elems: Vec<Element>) {
    let mut pool = Pool::new();

    // test element insertion
    let xs = elems.clone();
    for (i, x) in xs.into_iter().enumerate() {
        assert_eq!(pool.insert(x).to_usize(), i);
    }

    // test element retrieval
    for (i, x) in pool.into_iter().enumerate() {
        assert_eq!(x, elems[i]);
    }
}

/// Test that a unique pool accurately creates its items.
/// `elems` should have at least 8 items.
fn test_unique_pool<Element: Clone + Debug + Eq + Hash>(elems: Vec<Element>) {
    let mut pool = UniquePool::new();

    // test element insertion
    let xs = elems.clone();
    for (i, x) in xs.into_iter().enumerate() {
        // check element deduplication
        if i == 2 {
            assert_eq!(pool.insert(x.clone()).to_usize(), i);
        }

        assert_eq!(pool.insert(x).to_usize(), i);
    }

    // test element deduplication after inserts
    assert_eq!(pool.insert(elems[7].clone()).to_usize(), 7);

    for (i, x) in pool.into_iter().enumerate() {
        assert_eq!(x, elems[i]);
    }
}

#[test]
fn string_pool() {
    test_pool(strings_to_ten());
}

#[test]
fn bitset_pool() {
    test_pool(bitsets_to_twenty());
}

#[test]
fn string_unique_pool() {
    test_unique_pool(strings_to_ten());
}

#[test]
fn bitset_unique_pool() {
    test_unique_pool(bitsets_to_twenty());
}
