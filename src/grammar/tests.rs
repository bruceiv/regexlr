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

//! Tests for regexlr grammars

use super::examples;
use crate::matcher::matches;

#[test]
fn test_foobarbaz() {
    let g = examples::foobarbaz();
    assert!(matches(&g, "foobar").is_ok());
    assert!(matches(&g, "baz").is_ok());
    assert!(matches(&g, "foo").is_err());
    assert!(matches(&g, "bar").is_err());
    assert!(matches(&g, "").is_err());
}

#[test]
fn test_parens() {
    let g = examples::parens();
    assert!(matches(&g, "()").is_ok());
    assert!(matches(&g, "()()").is_ok());
    assert!(matches(&g, "(()())()").is_ok());
    assert!(matches(&g, "").is_ok());
    assert!(matches(&g, ")").is_err());
    assert!(matches(&g, "(()").is_err());
    assert!(matches(&g, "())").is_err());
}

#[test]
fn test_left_a_star() {
    let g = examples::left_a_star();
    assert!(matches(&g, "a").is_ok());
    assert!(matches(&g, "").is_ok());
    assert!(matches(&g, "aaa").is_ok());
    assert!(matches(&g, "b").is_err());
    assert!(matches(&g, "aab").is_err());
}
