//! This crate provides procedural macrs to generate functions that return static slices
//! of struct field names or enum variant names.
//!
//! # Examples
//! ```rust
//! use fievar::Fields;
//!
//! #[derive(Fields)]
//! struct File {
//!     id: String,
//!     name: String,
//!     mime_type: String,
//! }
//!
//! assert_eq!(&["id", "name", "mime_type"], File::fields());
//! ```
//!
//! You can also rename fields.
//! ```rust
//! use fievar::Fields;
//!
//! #[derive(Fields)]
//! struct File {
//!     id: String,
//!     name: String,
//!     #[fievar(name = "mimeType")]
//!     mime_type: String,
//! }
//!
//! assert_eq!(&["id", "name", "mimeType"], File::fields());
//! ```
//!
//! # Transformations
//! ## Syntax
//! *Expression:*  
//! &ensp;&ensp; [[*T*][`|`*Sep*]]
//!
//! *Sep:* Word separator.  
//! &ensp;&ensp; Can be any text.
//!
//! *T:*
//! Determines how to transform field/variant.  
//! &ensp;&ensp; [[*TrCase*][` `*NumAlign*]]
//!
//! *NumAlign:* Controls alignment of numerals.  
//! &ensp;&ensp; `1__` | `__1` | `_1_`  
//!
//! *TrCase:* Controls the case of letters.  
//! &ensp;&ensp; [*TrWord*[` `*TrWord*[` `*TrWord*]]]  
//!
//! *TrWord:* Controls the case of individual words.  
//! &ensp;&ensp; [*TrChar*[*TrChar*[*TrChar*]]]
//!
//! *TrChar:* Controls the case of individual characters in words.  
//! &ensp;&ensp; `c` | `C`  
//!
//! *TrCase* consists of upto three *TrWord*s separated by a space. If there is
//! only one *TrWord* then it is used to transform all words in field/variant. If
//! there are two *TrWord*s then the first *TrWord* is applied to the first word of
//! the field/variant and the second *TrWord* is applied to the rest of the words.
//! If there are three *TrWord*s then the first and last *TrWord*s are applied to
//! the first and last words of the field/variant and the second *TrWord* is
//! applied to the rest of the words. *TrChar*s work similarly on characters in a
//! word.
//!
//! ## Examples
//! ```rust
//! use fievar::Variants;
//!
//! #[derive(Variants)]
//! enum E {
//!     #[fievar(transform = "c")] // lowercase
//!     AVeryLong0Variant,
//!
//!     #[fievar(transform = "C")] // uppercase
//!     AVeryLong1Variant,
//!
//!     #[fievar(transform = "1__|_")] // align numeral left
//!     AVeryLong2Variant,
//!
//!     #[fievar(transform = "__1|_")] // align numeral right
//!     AVeryLong3Variant,
//!
//!     #[fievar(transform = "_1_|_")] // align numeral middle
//!     AVeryLong4Variant,
//!
//!     #[fievar(transform = "c Cc")] // camelCase
//!     AVeryLong5Variant,
//!
//!     #[fievar(transform = "c|_")] // snake_case
//!     AVeryLong6Variant,
//!
//!     #[fievar(transform = "CcC cCc CcC _1_|*-*")] // something different
//!     LastVeryLong7Variant,
//! }
//!
//! let v = &[
//!     "averylong0variant",
//!     "AVERYLONG1VARIANT",
//!     "A_Very_Long2_Variant",
//!     "A_Very_Long_3Variant",
//!     "A_Very_Long_4_Variant",
//!     "aVeryLong5Variant",
//!     "a_very_long6_variant",
//!     "LasT*-*vERy*-*lONg*-*7*-*VarianT"
//! ];
//! assert_eq!(v, E::variants());
//! ```

mod types;
mod utils;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed,
    Ident, Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta,
};
use types::{ExprParser, FievarParser, Tr, TrChars};

const FIEVAR: &str = "fievar";

macro_rules! gen_impl {
    ($strenm:expr, $fievars:expr, $fn_name:ident) => {{
        let fievars = $fievars;
        let strenm = $strenm;

        quote! (
            impl #strenm {
                pub fn $fn_name() -> &'static [&'static str] {
                    &[#(#fievars),*]
                }
            }
        )
    }};
}

/// Implements a `fields` method on structs that return an arry slice of struct field names.
#[proc_macro_derive(Fields, attributes(fievar))]
#[proc_macro_error]
pub fn fields(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(item as DeriveInput);

    let na = get_field_attr_pairs(data)
        .into_iter()
        .map(to_name)
        .collect::<Vec<_>>();

    gen_impl!(ident, na, fields).into()
}

/// Implements a `variants` method on enums that return an arry slice of enum variant names.
#[proc_macro_derive(Variants, attributes(fievar))]
#[proc_macro_error]
pub fn variants(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(item as DeriveInput);

    let na = get_variant_attr_pairs(data)
        .into_iter()
        .map(to_name)
        .collect::<Vec<_>>();

    gen_impl!(ident, na, variants).into()
}

fn to_name((ident, attr): (Ident, Option<Attribute>)) -> String {
    match attr {
        None => ident.to_string(),
        Some(a) => tr(ident.to_string(), a),
    }
}

fn tr(mut name: String, attr: Attribute) -> String {
    let nms = match attr.parse_meta() {
        Ok(Meta::List(MetaList { nested, .. })) => nested,
        Ok(v) => abort!(v, "expected #[{}(...)]", FIEVAR),
        Err(e) => abort!(attr, "expected #[{}(...)], {}", FIEVAR, e),
    };

    for nm in nms.into_iter() {
        name = match nm {
            NestedMeta::Lit(l) => abort!(
                l,
                "unexpected literal `{}`", quote!(#l);
                help = "expected `key = value` pairs"
            ),
            NestedMeta::Meta(Meta::Path(p)) => abort!(
                p,
                "unexpected attribute `{}`", quote!(#p);
                help = "expected `key = value` pairs"
            ),
            NestedMeta::Meta(Meta::List(ml)) => abort!(
                ml,
                "unexpected list `{}`", quote!(#ml);
                help = "expected `key = value` pairs"
            ),
            NestedMeta::Meta(Meta::NameValue(nv)) => tr_nv(name, nv),
        };
    }

    name
}

fn tr_nv(name: String, nv: MetaNameValue) -> String {
    let key = match nv.path.get_ident() {
        Some(i) => i.to_string(),
        None => {
            let p = nv.path;
            abort!(
                p,
                "unexpected attribute `{}`", quote!(#p);
                help = "expected a valid identifier"
            )
        }
    };

    let val = match nv.lit {
        Lit::Str(v) => v,
        l => abort!(
            l,
            "unexpected literal `{}`", quote!(#l);
            help = "expected a string literal"
        ),
    };

    match key.as_str() {
        "name" => val.value(),
        "transform" => tr_expr(name, val),
        _ => {
            let p = nv.path;
            abort!(p, "unrecognized attribute `{}`", quote!(#p))
        }
    }
}

fn tr_expr(name: String, expr: LitStr) -> String {
    let expr = ExprParser::parse(expr);
    let trs_len = expr.trs.len();

    let words = FievarParser::parse(&name, expr.db);
    let word_count = words.len();
    let words = words.into_iter();

    let tr_char = |txt: &str, tr: Tr| -> String {
        match tr {
            Tr::Upper => txt.to_uppercase(),
            Tr::Lower => txt.to_lowercase(),
            Tr::None => txt.to_string(),
        }
    };

    let tr_word = |txt: &str, tr: &TrChars| -> String {
        let l = txt.len();

        match (tr, l) {
            (&TrChars::All(tr), _) => tr_char(txt, tr),
            (&TrChars::FirstRest(f, _), 1) => tr_char(txt, f),
            (&TrChars::FirstRest(f, r), _) => {
                format!("{}{}", tr_char(&txt[..1], f), tr_char(&txt[1..], r))
            }
            (&TrChars::FirstMiddleLast(f, _, _), 1) => tr_char(txt, f),
            (&TrChars::FirstMiddleLast(f, m, l), _) => {
                let second_last = txt.len() - 1;
                format!(
                    "{}{}{}",
                    tr_char(&txt[..1], f),
                    tr_char(&txt[1..second_last], m),
                    tr_char(&txt[second_last..], l),
                )
            }
        }
    };

    match trs_len {
        0 => words.collect::<Vec<_>>().join(&expr.sep),
        1 => words
            .map(|w| tr_word(w, &expr.trs[0]))
            .collect::<Vec<_>>()
            .join(&expr.sep),
        2 => words
            .enumerate()
            .map(|(i, w)| match i {
                0 => tr_word(w, &expr.trs[0]),
                _ => tr_word(w, &expr.trs[1]),
            })
            .collect::<Vec<_>>()
            .join(&expr.sep),
        3 => words
            .enumerate()
            .map(|(i, w)| {
                if i == 0 {
                    tr_word(w, &expr.trs[0])
                } else if (1..word_count - 1).contains(&i) {
                    tr_word(w, &expr.trs[1])
                } else {
                    tr_word(w, &expr.trs[2])
                }
            })
            .collect::<Vec<_>>()
            .join(&expr.sep),
        _ => unreachable!(),
    }
}

fn get_variant_attr_pairs(data: Data) -> Vec<(Ident, Option<Attribute>)> {
    let variants = match data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("this macro can only be applied to enums"),
    };

    variants
        .into_iter()
        .map(|v| (v.ident, get_attr(v.attrs)))
        .collect::<Vec<_>>()
}

fn get_field_attr_pairs(data: Data) -> Vec<(Ident, Option<Attribute>)> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => panic!("this macro can only be applied to structs with named fields"),
    };

    fields
        .into_iter()
        .map(|f| (f.ident.unwrap(), get_attr(f.attrs)))
        .collect::<Vec<_>>()
}

fn get_attr(attrs: Vec<Attribute>) -> Option<Attribute> {
    let mut attrs = attrs
        .into_iter()
        .filter(|a| a.path.is_ident(FIEVAR))
        .collect::<Vec<_>>();

    match attrs.len() {
        0 => None,
        1 => Some(attrs.swap_remove(0)),
        _ => abort!(
            quote!(#(#attrs)*),
            "this attribute cannot be applied more than once"
        ),
    }
}
