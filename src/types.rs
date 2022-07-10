use std::{iter::Enumerate, str::Chars, vec::IntoIter};
use syn::LitStr;

#[derive(Clone, Copy, Debug)]
pub enum NumAlign {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum Tr {
    Upper,
    Lower,
    None,
}

#[derive(Debug)]
pub enum TrChars {
    All(Tr),
    FirstRest(Tr, Tr),
    FirstMiddleLast(Tr, Tr, Tr),
}

#[derive(Debug)]
pub struct Expr {
    pub db: NumAlign,
    pub sep: String,
    pub trs: Vec<TrChars>,
}

pub struct ExprParser {
    pub db: NumAlign,
    pub exprs: Vec<TrChars>,
    pub sep: String,

    pub input: LitStr,
    pub expr: Option<TrChars>,
    pub chars: IntoIter<(usize, char)>,
    pub expr_count: usize,
}

pub struct FievarParser<'a> {
    pub db: NumAlign,
    pub breaks: Vec<usize>,
    pub input: Enumerate<Chars<'a>>,
}
