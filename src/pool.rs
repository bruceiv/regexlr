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
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// An indexed append-only collection of `E`
pub struct Pool<E> {
    items: Vec<E>
}

impl<E> Pool<E> {
    /// Set up a new pool
    pub fn new() -> Self { Self{ items: Vec::new() } }

    /// Add a new element to the pool.
    /// Return the index of the added production.
    pub fn insert(&mut self, e: E) -> Ind<E> {
        self.items.push(e);
        Ind::of(self.items.len() - 1)
    }
}

impl<E> Index<Ind<E>> for Pool<E> {
    type Output = E;

    fn index(&self, i: Ind<E>) -> &Self::Output { &self.items[i.to_usize()] }
}

impl<E> IndexMut<Ind<E>> for Pool<E> {
    fn index_mut(&mut self, i: Ind<E>) -> &mut Self::Output { &mut self.items[i.to_usize()] }
}

impl<E: fmt::Debug> fmt::Debug for Pool<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.items.iter().enumerate())
            .finish()
    }
}

/// An index into a [Pool<E>]
pub struct Ind<E>(u32, PhantomData<*const E>);

impl<E> Ind<E> {
    /// construct from index
    fn of(i: usize) -> Self { Ind(i as u32, PhantomData) }

    /// export to index
    pub fn to_usize(&self) -> usize { self.0 as usize }
}

impl<E> Clone for Ind<E> {
    fn clone(&self) -> Self { Self(self.0, PhantomData) }
}

impl<E> Copy for Ind<E> {}

impl<E> PartialEq for Ind<E> {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<E> Eq for Ind<E> {}

impl<E> fmt::Debug for Ind<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
