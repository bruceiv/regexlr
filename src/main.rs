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

#[allow(dead_code)]
mod grammar;
mod pool;

use grammar::{Expr, Grammar};

fn foobarbaz() -> Grammar {
    let mut g = Grammar::new();

    let foo = g.lit(&"foo");
    let bar = g.lit(&"bar");
    let b1 = g.seq(vec![foo, bar]);

    let baz = g.lit(&"baz");

    let b0 = g.alt(vec![b1, baz]);
    g.start(b0);

    g
}

fn parens() -> Grammar {
    let mut g = Grammar::new();

    let p = g.call("P");
    let open = g.lit(&"(");
    let close = g.lit(&")");
    let ps = g.seq(vec![open, p, close]);

    g.rule("P", vec![ps, Expr::Empty]);
    g.start(p);

    g
}

fn main() {
    println!("foobarbaz: {:#?}", foobarbaz());
    println!("parens: {:#?}", parens());
}
