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

//! A matcher for RegexLR expressions.
//! 
//! This uses a Packrat algorithm, modified to keep multiple possible prefix 
//! matches for RegexLR's unordered-choice semantics. It uses the memoization 
//! approach of Warth, Douglass & Millstein to support left-recursion.

use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::{hash_map, HashMap};
use std::iter::zip;

use bit_set::BitSet;

use crate::grammar::{Alternation, Atom, Grammar, Production, Slot};
use crate::pool::Ind;
use crate::Error::{InputDoesNotMatch, MissingRule, MissingStart};

/// A success result for matching
#[derive(Clone, Debug)]
enum Match {
    /// Short-circuit on successful match of entire input
    Done,
    /// Match a prefix of the entire input, ending at the contained indices.
    /// Indices will be non-empty.
    Prefix(BitSet),
    /// `Prefix`, but with incomplete information due to unresolved left-recursion.
    /// Indices may be empty.
    Incomplete(BitSet),
}

impl Match {
    /// Creates a partial match at the given index
    fn at(i: usize) -> Self {
        let mut set = BitSet::new();
        set.insert(i);
        Self::Prefix(set)
    }

    /// Creates a new incomplete result
    fn incomplete() -> Self {
        Self::Incomplete(BitSet::new())
    }
}

/// Failure result.
/// Couples index of farthest match with vector of failures
#[derive(Clone, Debug)]
pub struct ParseFailure(usize, Vec<crate::Error>);

impl ParseFailure {
    /// Create a new empty failure list at the given index
    fn new(start: usize) -> Self {
        Self(start, Vec::new())
    }

    /// Create a new single failure at the given index
    fn at(start: usize, err: crate::Error) -> Self {
        Self(start, vec![err])
    }

    /// Create a new [InputDoesNotMatch] failure at the given index
    fn no_match_at(start: usize) -> Self {
        Self::at(start, InputDoesNotMatch { location: start })
    }

    /// Update the failure list with a new set of failures
    /// Does nothing if new set is earlier, replaces if later, appends if equal position
    fn append(&mut self, mut other: Self) {
        match other.0.cmp(&self.0) {
            Less => {
                // do nothing
            }
            Greater => {
                // new latest errors
                self.0 = other.0;
                self.1 = other.1;
            }
            Equal => {
                // add to existing set of latest errors
                self.1.append(&mut other.1);
            }
        }
    }
}

/// Intermediate parse result.
/// A non-empty set of next indices on success, a non-empty list of errors from the
/// furthest match postion on failure.
type ParseResult = Result<Match, ParseFailure>;

/// State of the parser
struct ParseState {
    /// Memoization table for alternation results;
    /// key is index into input and alternation table, value is cached result
    alternation_results: HashMap<(usize, Ind<Slot<Alternation>>), ParseResult>,
    /// Memoization table for production results;
    /// key is index into input and production table, value is cached result
    production_results: HashMap<(usize, Ind<Production>), ParseResult>,
}

impl ParseState {
    /// Creates a new empty state
    fn new() -> Self {
        Self {
            alternation_results: HashMap::new(),
            production_results: HashMap::new(),
        }
    }

    /// Looks up an alternation, either returning `Some` of a previous result or inserting a new incomplete result
    fn lookup_alternation(
        &mut self,
        i: usize,
        a_ind: Ind<Slot<Alternation>>,
    ) -> Option<ParseResult> {
        use hash_map::Entry::{Occupied, Vacant};
        // look up previously-computed result in memo-table
        match self.alternation_results.entry((i, a_ind)) {
            Occupied(entry) => {
                // Clone previous result
                Some(entry.get().clone())
            }
            Vacant(entry) => {
                // start new empty in-progress state
                entry.insert(Ok(Match::incomplete()));
                None
            }
        }
    }

    /// Updates an alternation state with a new incomplete match
    fn update_alternation(
        &mut self,
        i: usize,
        a_ind: Ind<Slot<Alternation>>,
        partial: &BitSet,
    ) {
        self.alternation_results
            .insert((i, a_ind), Ok(Match::Incomplete(partial.clone())));
    }

    /// Finalizes an alternation state, returning the state
    fn finish_alternation(
        &mut self,
        i: usize,
        a_ind: Ind<Slot<Alternation>>,
        res: ParseResult,
    ) -> ParseResult {
        self.alternation_results.insert((i, a_ind), res.clone());
        res
    }

    /// Looks up a production, returning `Some` of a state if present
    fn lookup_production(&self, i: usize, p_ind: Ind<Production>) -> Option<ParseResult> {
        self.production_results
            .get(&(i, p_ind))
            .map(|res| res.clone())
    }

    /// Inserts a production state, returning the state
    fn insert_production(
        &mut self,
        i: usize,
        p_ind: Ind<Production>,
        res: ParseResult,
    ) -> ParseResult {
        self.production_results.insert((i, p_ind), res.clone());
        res
    }
}

/// Is `s` matched by `g`?
/// Returns furthest error on failure.
pub fn matches(g: &Grammar, s: &str) -> Result<(), ParseFailure> {
    // start production of grammar
    let Some(start_prod) = g.start_production() else {
        return Err(ParseFailure::at(0, MissingStart));
    };

    // set up memo-table and start matching
    let mut state = ParseState::new();
    matches_prod(g, start_prod, s, &mut state, 0).and(Ok(()))
}

/// Does alternation `a` of `g` match a prefix of `s` starting at `start`?
/// Returns matching indices on success, furthest error on failure.
fn matches_alt(
    g: &Grammar,
    a_ind: Ind<Slot<Alternation>>,
    s: &str,
    state: &mut ParseState,
    start: usize,
) -> ParseResult {
    // short-circuit from memo-table
    if let Some(result) = state.lookup_alternation(start, a_ind) {
        return result;
    }

    // look up alternation in grammar
    let Some(alt) = &g[a_ind] else {
        // fail with a grammar error if alternation does not exist
        let no_such_rule = Err(ParseFailure::at(start, MissingRule { ind: a_ind }));
        return state.finish_alternation(start, a_ind, no_such_rule);
    };

    // matches for this alternation at this start position
    let mut all_matches = BitSet::new();
    // matches found in the current loop iteration
    let mut new_matches = BitSet::new();
    // errors at furthest position
    let mut latest_errs = ParseFailure::new(start);
    // productions with incomplete results
    let mut incomplete_prods = alt.production_inds().clone();

    // while we're still finding new matches
    loop {
        // productions completed this iteration
        let mut completed_prods = BitSet::new();

        // loop through alternatives
        for p in &incomplete_prods {
            let p_ind = Ind::of(p);
            match matches_prod(g, p_ind, s, state, start) {
                Ok(Match::Done) => {
                    // short-circuit on completed match
                    return Ok(Match::Done);
                }
                Ok(Match::Prefix(inds)) => {
                    // add success results to output set, mark production complete
                    new_matches.union_with(&inds);
                    completed_prods.insert(p);
                }
                Ok(Match::Incomplete(inds)) => {
                    // add success results to output set
                    new_matches.union_with(&inds);
                }
                Err(errs) => {
                    // track latest errors, mark production complete
                    latest_errs.append(errs);
                    completed_prods.insert(p);
                }
            }
        }
        // stop re-trying completed productions and previous matches
        new_matches.difference_with(&all_matches);
        incomplete_prods.difference_with(&completed_prods);
        
        if new_matches.is_empty() || incomplete_prods.is_empty() {
            // found all (possibly recursive) matches
            all_matches.union_with(&new_matches);
            let res = if all_matches.is_empty() {
                Err(latest_errs)
            } else {
                Ok(Match::Prefix(all_matches))
            };
            return state.finish_alternation(start, a_ind, res);
        } else {
            // update current set of matches and continue
            all_matches.union_with(&new_matches);
            state.update_alternation(start, a_ind, &all_matches);
        }
    }
}

/// Does production `p` of `g` match a prefix of `s` starting at `start`?
/// Returns matching indices on success, furthest error on failure.
fn matches_prod(
    g: &Grammar,
    p_ind: Ind<Production>,
    s: &str,
    state: &mut ParseState,
    start: usize,
) -> ParseResult {
    // short-circuit on production already seen and completed
    // NOTE: production only inserted if complete
    if let Some(result) = state.lookup_production(start, p_ind) {
        return result;
    }

    // production under investigation
    let prod = &g[p_ind];

    // next valid indices to match
    let mut start_inds = BitSet::new();
    start_inds.insert(start);

    // quick exit on empty production
    if prod.is_empty() {
        return state.insert_production(start, p_ind, Ok(Match::Prefix(start_inds)));
    }

    // errors at furthest position
    let mut latest_errs = ParseFailure::new(start);
    // is the match complete?
    let mut is_complete = true;

    // loop through productions
    for atom in prod {
        let mut next_inds = BitSet::new();

        // loop through start indices
        for i in &start_inds {
            match matches_atom(g, atom, s, state, i) {
                Ok(Match::Done) => {
                    // short-circuit on completed match
                    return Ok(Match::Done);
                }
                Ok(Match::Prefix(inds)) => {
                    // add success results to output set
                    next_inds.union_with(&inds);
                }
                Ok(Match::Incomplete(inds)) => {
                    // add success results to output set, but mark incomplete
                    is_complete = false;
                    next_inds.union_with(&inds);
                }
                Err(errs) => {
                    // track latest errors
                    latest_errs.append(errs);
                }
            }
        }

        if next_inds.is_empty() {
            if is_complete {
                // fail if no remaining match indices
                return state.insert_production(start, p_ind, Err(latest_errs));
            } else {
                // return incomplete failure
                return Ok(Match::Incomplete(next_inds));
            }
        } else {
            // update for start of next production otherwise
            start_inds = next_inds;
        }
    }

    // got to the end, everything matches, only cache state if complete
    if is_complete {
        state.insert_production(start, p_ind, Ok(Match::Prefix(start_inds)))
    } else {
        Ok(Match::Incomplete(start_inds))
    }
}

/// Does atom `a` of `g` match of prefix of `s` starting at `start`?
/// Returns matching indices on success, furthest error on failure.
fn matches_atom(
    g: &Grammar,
    atom: &Atom,
    s: &str,
    state: &mut ParseState,
    start: usize,
) -> ParseResult {
    use Atom::*;
    match atom {
        Literal(ind) => {
            let lit = &g[*ind];
            let (_, s_post) = s.split_at(start);
            if s_post.starts_with(lit) {
                // match after the literal
                Ok(Match::at(start + lit.len()))
            } else {
                // find index of first non-match in strings
                let last_match = zip(s_post.bytes(), lit.bytes())
                    .enumerate()
                    .find(|(_, (b1, b2))| b1 != b2)
                    .map(|(i, _)| start + i)
                    .unwrap_or(start + lit.len()); // shouldn't happen
                Err(ParseFailure::no_match_at(last_match))
            }
        }
        Capture(ind) => matches_alt(g, *ind, s, state, start),
        End => {
            if start == s.len() {
                // found total match, insert short-circuit
                Ok(Match::Done)
            } else {
                // not end-of-input
                Err(ParseFailure::no_match_at(start))
            }
        }
        // always fails
        Fail => Err(ParseFailure::no_match_at(start)),
    }
}
