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

//! Compilation and matching errors for RegexLR parsers.

use crate::grammar::{Alternation, Slot};
use crate::pool::Ind;

/// Errors that a RegexLR grammar may encounter
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Input does not match grammar
    InputDoesNotMatch {
        /// location of furthest partial match
        location: usize,
    },
    /// Grammar has no start rule
    MissingStart,
    /// Grammar missing called non-terminal
    MissingRule { ind: Ind<Slot<Alternation>> },
}
