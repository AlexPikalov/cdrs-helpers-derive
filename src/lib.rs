//! This trait provides functionality for derivation  `IntoCDRSBytes` trait implementation
//! for underlying

extern crate cdrs;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate rand;
extern crate syn;

mod common;
mod into_cdrs_value;
mod try_from_row;
mod try_from_udt;

use proc_macro::TokenStream;
use into_cdrs_value::impl_into_cdrs_value;
use try_from_row::impl_try_from_row;
use try_from_udt::impl_try_from_udt;

#[proc_macro_derive(IntoCDRSValue)]
pub fn into_cdrs_value(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_into_cdrs_value(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

#[proc_macro_derive(TryFromRow)]
pub fn try_from_row(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_try_from_row(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

#[proc_macro_derive(TryFromUDT)]
pub fn try_from_udt(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_try_from_udt(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}
