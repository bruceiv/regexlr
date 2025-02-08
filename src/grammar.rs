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

//! A RegexLR grammar.
//! Includes supporting data structures for expressions, productions, and alternations.

pub(crate) mod examples;

#[cfg(test)]
mod tests;

use std::fmt;
use std::{collections::BTreeMap, ops::Index};

use bit_set::BitSet;

use crate::pool::{Ind, Pool, UniquePool};

/// A choice of RegexLR expressions
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Alternation {
    /// Productions in alternation
    prods: BitSet,
}

impl Alternation {
    /// Create a new, empty alternation
    fn new() -> Self {
        Self {
            prods: BitSet::new(),
        }
    }

    /// Create a single-production alternation
    fn of(i: Ind<Production>) -> Self {
        let mut alt = Self::new();
        alt.insert(i);
        alt
    }

    /// Add an alternate to the alternation
    fn insert(&mut self, i: Ind<Production>) {
        self.prods.insert(i.to_usize());
    }

    /// Copy an alternation into this one
    fn union_with(&mut self, that: &Alternation) {
        self.prods.union_with(&that.prods);
    }

    /// Get the production indices
    pub fn production_inds(&self) -> &BitSet {
        &self.prods
    }
}

impl fmt::Debug for Alternation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.prods)
    }
}

/// A (possibly empty) slot for an alternation
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Slot<E>(Option<E>);

impl<E> Slot<E> {
    /// a new, empty alternation slot
    fn empty() -> Self {
        Self(None)
    }

    /// an alternation slot containing a value
    fn of(item: E) -> Self {
        Self(Some(item))
    }
}

impl<E: fmt::Debug> fmt::Debug for Slot<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alt) = &self.0 {
            write!(f, "{:?}", alt)
        } else {
            write!(f, "<unset>")
        }
    }
}

/// A sequence of RegexLR expressions
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Production {
    /// expressions in production
    atoms: Vec<Atom>,
}

impl Production {
    /// Create a new, empty production
    fn new() -> Self {
        Self { atoms: Vec::new() }
    }

    /// Create a new production for a single atom
    fn of(atom: Atom) -> Self {
        Self { atoms: vec![atom] }
    }

    /// Create a new production from a list of atoms
    fn of_all(atoms: Vec<Atom>) -> Self {
        Self { atoms }
    }

    /// Copy this production with an added atom
    fn with(&self, atom: Atom) -> Self {
        let mut atoms = self.atoms.clone();
        atoms.push(atom);
        Self { atoms }
    }

    /// Add an atom to this production
    fn push(&mut self, atom: Atom) {
        self.atoms.push(atom);
    }

    /// Copy a production to this production
    fn append(&mut self, that: &Production) {
        self.atoms.append(&mut that.atoms.clone());
    }

    /// Is this production empty?
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }

    /// Borrowing iterator
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<'p> IntoIterator for &'p Production {
    type Item = &'p Atom;

    type IntoIter = <&'p Vec<Atom> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.atoms.iter()
    }
}

impl fmt::Debug for Production {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.atoms)
    }
}

/// A single RegexLR expression
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Atom {
    /// A literal string
    Literal(Ind<String>),
    /// A capture group
    Capture(Ind<Slot<Alternation>>),
    /// End-of-input
    End,
    /// Failure
    Fail,
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Literal(i) => write!(f, "Literal({:?})", i),
            Atom::Capture(i) => write!(f, "Capture({:?})", i),
            Atom::End => write!(f, "End"),
            Atom::Fail => write!(f, "Fail"),
        }
    }
}

/// An arbitrary RegexLR expression
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Expr {
    /// Choice between expressions
    Choice(Ind<Slot<Alternation>>),
    /// Sequence of expressions
    Sequence(Ind<Production>),
    /// Literal string
    Literal(Ind<String>),
    /// Always matches
    Empty,
    /// Never matches
    Fail,
}

/// A RegexLR grammar
#[derive(Debug)]
pub struct Grammar {
    /// nonterminal index
    /// maps to top-level alternation of nonterminal
    nonterminals: BTreeMap<String, Ind<Slot<Alternation>>>,
    /// alternation pool
    alternations: Pool<Slot<Alternation>>,
    /// production pool
    productions: UniquePool<Production>,
    /// start production
    start_prod: Slot<Ind<Production>>,
    /// empty production
    empty_prod: Ind<Production>,
    /// string pool
    strings: UniquePool<String>,
}

impl Grammar {
    /// Create a new, empty builder.
    /// Start rule is a reserved but un-assigned production
    pub fn new() -> Self {
        let nonterminals = BTreeMap::new();
        let alternations = Pool::new();
        let mut productions = UniquePool::new();
        let start_prod = Slot::empty();
        let empty_prod = productions.insert(Production::new());
        let strings = UniquePool::new();

        Grammar {
            nonterminals,
            alternations,
            productions,
            start_prod,
            empty_prod,
            strings,
        }
    }

    /// Get the index of the start production
    pub(crate) fn start_production(&self) -> Option<Ind<Production>> {
        // fail early if missing production
        self.start_prod.0
    }

    /// Build the start rule from the given expression
    pub fn start(&mut self, expr: Expr) {
        let prod = match expr {
            Expr::Choice(alt) => Production::of_all(vec![Atom::Capture(alt), Atom::End]),
            Expr::Sequence(prod) => self.productions[prod].with(Atom::End),
            Expr::Literal(lit) => Production::of_all(vec![Atom::Literal(lit), Atom::End]),
            Expr::Empty => Production::of(Atom::End),
            Expr::Fail => Production::of(Atom::Fail),
        };
        let ind = self.productions.insert(prod);
        self.start_prod = Slot::of(ind);
    }

    /// Build a new non-terminal from the given expression
    pub fn rule(&mut self, name: &str, exprs: Vec<Expr>) {
        let i = self.begin_rule(name);
        self.finish_rule(i, exprs)
    }

    /// Sets up a new rule with the given name, returning its index
    fn begin_rule(&mut self, name: &str) -> Ind<Slot<Alternation>> {
        match self.nonterminals.get(name) {
            Some(i) => *i,
            None => {
                let i = self.alternations.insert(Slot::empty());
                self.nonterminals.insert(name.to_string(), i);
                i
            }
        }
    }

    /// Completes a rule with an expression
    fn finish_rule(&mut self, i: Ind<Slot<Alternation>>, exprs: Vec<Expr>) {
        let alternation = self.alternation_of(exprs);
        self.alternations[i] = Slot::of(alternation);
    }

    /// Creates a call to a nonterminal
    pub fn call(&mut self, name: &str) -> Expr {
        Expr::Choice(self.begin_rule(name))
    }

    /// Creates an alternation of expressions
    pub fn alt(&mut self, exprs: Vec<Expr>) -> Expr {
        // filter out failure values from alternation
        let exprs: Vec<_> = exprs.into_iter().filter(|x| *x != Expr::Fail).collect();

        // failure for empty alternation
        if exprs.is_empty() {
            return Expr::Fail;
        }

        // single expression for single value
        if exprs.len() == 1 {
            return exprs[0];
        }

        // transform alternation of expressions into new alternation expression
        let alternation = self.alternation_of(exprs);
        Expr::Choice(self.alternations.insert(Slot::of(alternation)))
    }

    /// Converts a list of expressions into an alternation
    fn alternation_of(&mut self, exprs: Vec<Expr>) -> Alternation {
        let mut alternation = Alternation::new();
        for expr in exprs {
            match expr {
                Expr::Choice(alt) => {
                    if let Some(inner_alt) = &self.alternations[alt].0 {
                        // union in already-defined alternations
                        alternation.union_with(&inner_alt);
                    } else {
                        // make new production for undefined alternations
                        let prod = Production::of(Atom::Capture(alt));
                        alternation.insert(self.productions.insert(prod));
                    }
                }
                Expr::Sequence(prod) => {
                    alternation.insert(prod);
                }
                Expr::Literal(lit) => {
                    let prod = Production::of(Atom::Literal(lit));
                    alternation.insert(self.productions.insert(prod));
                }
                Expr::Empty => {
                    alternation.insert(self.empty_prod);
                }
                Expr::Fail => { /* do nothing */ }
            }
        }
        alternation
    }

    /// Creates a sequence of expressions
    pub fn seq(&mut self, exprs: Vec<Expr>) -> Expr {
        // filter out empty values from sequence
        let exprs: Vec<_> = exprs.into_iter().filter(|x| *x != Expr::Empty).collect();

        // empty for empty sequence
        if exprs.is_empty() {
            return Expr::Empty;
        }

        // single expression for single value
        if exprs.len() == 1 {
            return exprs[0];
        }

        // transform sequence of expressions into new Sequence expression
        let mut production = Production::new();
        for expr in exprs {
            match expr {
                Expr::Choice(alt) => {
                    production.push(Atom::Capture(alt));
                }
                Expr::Sequence(prod) => {
                    production.append(&self.productions[prod]);
                }
                Expr::Literal(lit) => {
                    production.push(Atom::Literal(lit));
                }
                Expr::Empty => {
                    unreachable!("empty expressions filtered earlier")
                }
                Expr::Fail => {
                    // any failure in a sequence causes the whole sequence to fail
                    return Expr::Fail;
                }
            }
        }
        Expr::Sequence(self.productions.insert(production))
    }

    /// Creates a literal string.
    /// Returns [Expr::Empty] for empty string.
    pub fn lit<S: ToString>(&mut self, s: &S) -> Expr {
        let t = s.to_string();
        if t.is_empty() {
            Expr::Empty
        } else {
            Expr::Literal(self.strings.insert(t))
        }
    }
}

impl Index<Ind<Slot<Alternation>>> for Grammar {
    type Output = Option<Alternation>;

    /// get alternation from pool, `None` if not set
    fn index(&self, i: Ind<Slot<Alternation>>) -> &Self::Output {
        &self.alternations[i].0
    }
}

impl Index<Ind<Production>> for Grammar {
    type Output = Production;

    /// get string from pool
    fn index(&self, i: Ind<Production>) -> &Self::Output {
        &self.productions[i]
    }
}

impl Index<Ind<String>> for Grammar {
    type Output = String;

    /// get string from pool
    fn index(&self, i: Ind<String>) -> &Self::Output {
        &self.strings[i]
    }
}
