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

//! Example RegexLR grammars.
//! Mostly present for testing

use crate::{Expr, Grammar};

/// Matches "foobar" or "baz"
pub fn foobarbaz() -> Grammar {
    let mut g = Grammar::new();

    let foo = g.lit(&"foo");
    let bar = g.lit(&"bar");
    let b1 = g.seq(vec![foo, bar]);

    let baz = g.lit(&"baz");

    let b0 = g.alt(vec![b1, baz]);
    g.start(b0);

    g
}

/// Matches any set of properly nested `(` and `)` parens
pub fn parens() -> Grammar {
    let mut g = Grammar::new();

    let p = g.call("P");
    let open = g.lit(&"(");
    let close = g.lit(&")");
    let nested_ps = g.seq(vec![open, p, close, p]);

    g.rule("P", vec![nested_ps, Expr::Empty]);
    g.start(p);

    g
}

/// Left-recursively matches `a*` (any string containing only `a`s)
pub fn left_a_star() -> Grammar {
    let mut g = Grammar::new();

    let a_star = g.call("a_star");
    let a = g.lit(&"a");
    let a_rec = g.seq(vec![a_star, a]);

    g.rule("a_star", vec![a_rec, Expr::Empty]);
    g.start(a_star);

    g
}
