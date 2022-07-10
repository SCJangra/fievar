use proc_macro_error::abort;
use syn::LitStr;

use crate::types::{
    Expr, ExprParser, FievarParser,
    NumAlign::{self, *},
    Tr::{self, *},
    TrChars::*,
};

macro_rules! next_or_return {
    ($iter:expr) => {{
        match $iter.next() {
            Some(v) => v,
            _ => return,
        }
    }};
}

macro_rules! parse_error {
    ($str:expr, $i:expr, $c:expr) => {{
        abort!($str, "invalid character '{}' at {}", $c, $i)
    }};
}

impl ExprParser {
    pub fn parse(input: LitStr) -> Expr {
        let v = input.value();

        let (expr, sep) = v
            .split_once('|')
            .map(|(e, s)| (e, s.to_string()))
            .unwrap_or((&v, "".to_string()));

        let expr = expr.chars().enumerate().collect::<Vec<_>>().into_iter();

        let mut m = Self {
            db: Left,
            exprs: vec![],
            sep,

            input,
            expr: Option::None,
            chars: expr,
            expr_count: 0,
        };
        m.run();

        Expr {
            db: m.db,
            sep: m.sep,
            trs: m.exprs,
        }
    }
    fn run(&mut self) {
        self.start();
        let e = self.expr.take();

        if let Some(e) = e {
            self.exprs.push(e);
        }
    }
    fn start(&mut self) {
        let (i, c) = next_or_return!(self.chars);
        let ec = self.expr_count;

        match c {
            'C' if ec < 3 => {
                self.expr = Some(All(Upper));
                self.all(Upper);
            }
            'c' if ec < 3 => {
                self.expr = Some(All(Lower));
                self.all(Lower);
            }
            '*' if ec < 3 => {
                self.expr = Some(All(None));
                self.all(None);
            }
            '_' => self.middle_right(),
            ' ' => self.start(),
            '1' => {
                self.db = Left;
                self.left();
            }
            _ => parse_error!(self.input, i, c),
        }
    }
    fn all(&mut self, tr: Tr) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            'C' => {
                self.expr = Some(FirstRest(tr, Upper));
                self.first_rest(tr, Upper);
            }
            'c' => {
                self.expr = Some(FirstRest(tr, Lower));
                self.first_rest(tr, Lower);
            }
            '*' => {
                self.expr = Some(FirstRest(tr, None));
                self.first_rest(tr, None);
            }
            ' ' => {
                self.exprs.push(All(tr));
                self.expr = Option::None;
                self.expr_count += 1;
                self.start();
            }
            _ => parse_error!(self.input, i, c),
        }
    }
    fn first_rest(&mut self, tr1: Tr, tr2: Tr) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            'C' => {
                self.expr = Some(FirstMiddleLast(tr1, tr2, Upper));
                self.first_middle_last(tr1, tr2, Upper);
            }
            'c' => {
                self.expr = Some(FirstMiddleLast(tr1, tr2, Lower));
                self.first_middle_last(tr1, tr2, Lower);
            }
            '*' => {
                self.expr = Some(FirstMiddleLast(tr1, tr2, None));
                self.first_middle_last(tr1, tr2, None);
            }
            ' ' => {
                self.exprs.push(FirstRest(tr1, tr2));
                self.expr = Option::None;
                self.expr_count += 1;
                self.start();
            }
            _ => parse_error!(self.input, i, c),
        }
    }
    fn first_middle_last(&mut self, tr1: Tr, tr2: Tr, tr3: Tr) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            ' ' => {
                self.exprs.push(FirstMiddleLast(tr1, tr2, tr3));
                self.expr = Option::None;
                self.expr_count += 1;
                self.start();
            }
            _ => parse_error!(self.input, i, c),
        }
    }
    fn left(&mut self) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            '_' => self.left(),
            ' ' => self.end(),
            _ => parse_error!(self.input, i, c),
        }
    }
    fn middle(&mut self) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            '_' => self.middle(),
            ' ' => self.end(),
            _ => parse_error!(self.input, i, c),
        }
    }
    fn right(&mut self) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            '_' => {
                self.db = Middle;
                self.middle();
            }
            ' ' => self.end(),
            _ => parse_error!(self.input, i, c),
        }
    }
    fn middle_right(&mut self) {
        let (i, c) = next_or_return!(self.chars);

        match c {
            '_' => self.middle_right(),
            '1' => {
                self.db = Right;
                self.right();
            }
            _ => parse_error!(self.input, i, c),
        }
    }
    fn end(&mut self) {}
}

impl<'a> FievarParser<'a> {
    pub fn parse(s: &'a str, db: NumAlign) -> Vec<&str> {
        let last = s.len() - 1;
        let mut m = Self {
            db,
            input: s.chars().enumerate(),
            breaks: vec![],
        };

        m.start();

        let len = m.breaks.len();

        if len % 2 != 0 {
            m.breaks.push(last);
        }

        let mut r = vec![];
        for i in (0..len).step_by(2) {
            r.push(&s[m.breaks[i]..=m.breaks[i + 1]]);
        }
        r
    }
    fn start(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.under(),
            'A'..='Z' => self.push_next(&[i], Self::upper),
            'a'..='z' => self.push_next(&[i], Self::lower),
            _ => unreachable!(),
        }
    }
    fn under(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.under(),
            'A'..='Z' => self.push_next(&[i], Self::upper),
            'a'..='z' => self.push_next(&[i], Self::lower),
            '0'..='9' => self.push_next(&[i], Self::digit),
            _ => unreachable!(),
        }
    }
    fn upper(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.push_next(&[i - 1], Self::under),
            'A'..='Z' => self.upper_upper(),
            'a'..='z' => self.lower(),
            '0'..='9' => self.alphabet_digit(i),
            _ => unreachable!(),
        }
    }
    fn upper_upper(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.push_next(&[i - 1], Self::under),
            'A'..='Z' => self.upper_upper(),
            'a'..='z' => self.push_next(&[i - 2, i - 1], Self::lower),
            '0'..='9' => self.alphabet_digit(i),
            _ => unreachable!(),
        }
    }
    fn lower(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.push_next(&[i - 1], Self::under),
            'A'..='Z' => self.push_next(&[i - 1, i], Self::upper),
            'a'..='z' => self.lower(),
            '0'..='9' => self.alphabet_digit(i),
            _ => unreachable!(),
        }
    }
    fn alphabet_digit(&mut self, ds: usize) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.push_next(&[i - 1], Self::under),
            'A'..='Z' => match self.db {
                Left => self.push_next(&[i - 1, i], Self::upper),
                Middle => self.push_next(&[ds - 1, ds, i - 1, i], Self::upper),
                Right => self.push_next(&[ds - 1, ds], Self::upper),
            },
            'a'..='z' => match self.db {
                Left => self.push_next(&[i - 1, i], Self::lower),
                Middle => self.push_next(&[ds - 1, ds, i - 1, i], Self::lower),
                Right => self.push_next(&[ds - 1, ds], Self::lower),
            },
            '0'..='9' => self.alphabet_digit(ds),
            _ => unreachable!(),
        }
    }
    fn digit(&mut self) {
        let (i, c) = next_or_return!(self.input);

        match c {
            '_' => self.push_next(&[i - 1], Self::under),
            'A'..='Z' => match self.db {
                Right => self.upper(),
                _ => self.push_next(&[i - 1, i], Self::upper),
            },
            'a'..='z' => match self.db {
                Right => self.lower(),
                _ => self.push_next(&[i - 1, i], Self::lower),
            },
            '0'..='9' => self.digit(),
            _ => unreachable!(),
        }
    }
    fn push_next(&mut self, i: &[usize], f: fn(&mut FievarParser<'a>)) {
        self.breaks.extend_from_slice(i);
        f(self);
    }
}
