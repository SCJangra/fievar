//! This crate provides procedural macrs to generate functions that return static slices
//! of struct field names and enum variant names.

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed,
    Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta,
};

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
///
/// # Examples
///
/// ```rust
/// use fievar::Fields;
///
/// #[derive(Fields)]
/// struct Token {
///     access_token: String,
///     refresh_token: String,
/// }
///
/// assert_eq!(&["access_token", "refresh_token"], Token::fields());
/// ```
///
/// You can also rename fields.
/// ```rust
/// use fievar::Fields;
///
/// #[derive(Fields)]
/// struct Token {
///     #[fievar(name = "accessToken")]
///     access_token: String,
///     refresh_token: String,
/// }
///
/// assert_eq!(&["accessToken", "refresh_token"], Token::fields());
/// ```
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
///
/// This uses the same syntax as [`Fields`] macro. Look there for examples.
///
/// [`Fields`]: fields
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
        Some(a) => {
            let mut name = ident.to_string();

            for nv in get_meta_nv(a) {
                name = transform(name, nv);
            }
            name
        }
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

fn get_meta_nv(attr: Attribute) -> Vec<MetaNameValue> {
    let nm = match attr.parse_meta() {
        Ok(Meta::List(MetaList { nested, .. })) => nested,
        Ok(v) => abort!(v, "expected #[{}(...)]", FIEVAR),
        Err(e) => abort!(attr, "expected #[{}(...)], {}", FIEVAR, e),
    };

    nm.into_iter()
        .map(|nm| match nm {
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
            NestedMeta::Meta(Meta::List(MetaList { path, .. })) => abort!(
                path,
                "unexpected attribute `{}`", quote!(#path);
                help = "expected `key = value` pairs"
            ),
            NestedMeta::Meta(Meta::NameValue(nv)) => nv,
        })
        .collect::<Vec<_>>()
}

fn transform(_name: String, nv: MetaNameValue) -> String {
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

    match key.as_str() {
        "name" => get_name(nv.lit),
        _ => {
            let p = nv.path;
            abort!(p, "unrecognized attribute `{}`", quote!(#p))
        }
    }
}

fn get_name(lit: Lit) -> String {
    match lit {
        Lit::Str(l) => l.value(),
        l => abort!(
            l,
            "unexpected literal `{}`", quote!(#l);
            help = "expected a string literal"
        ),
    }
}
