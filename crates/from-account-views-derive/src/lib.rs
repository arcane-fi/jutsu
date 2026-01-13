// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Type,
};

#[proc_macro_derive(FromAccountViews, attributes(meta))]
pub fn derive_from_account_views(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    // ---- extract exactly one lifetime ('ix)
    let info_lt = match input.generics.lifetimes().collect::<Vec<_>>().as_slice()
    {
        [lt] => &lt.lifetime,
        [] => {
            return syn::Error::new(
                input.span(),
                "FromAccountViews requires exactly one lifetime parameter",
            )
            .to_compile_error()
            .into();
        }
        _ => {
            return syn::Error::new(
                input.span(),
                "FromAccountViews supports exactly one lifetime parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => {
                return syn::Error::new(
                    s.fields.span(),
                    "FromAccountViews supports named fields only",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(
                input.span(),
                "FromAccountViews can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut bindings = Vec::new();
    let mut field_idents = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        field_idents.push(ident);

        let meta_expr = match parse_meta(&field.attrs, ty, info_lt) {
            Ok(m) => m,
            Err(e) => return e.to_compile_error().into(),
        };

        bindings.push(quote! {
            let #ident =
                <#ty as FromAccountView<#info_lt>>::try_from_account_view(
                    account_views.next()?,
                    #meta_expr,
                )?;
        });
    }

    let expanded = quote! {
        impl #impl_generics FromAccountViews<#info_lt>
            for #struct_name #ty_generics #where_clause
        {
            #[inline(always)]
            fn try_from_account_views(
                account_views: &mut AccountIter<#info_lt>
            ) -> Result<Self> {
                #(#bindings)*

                Ok(Self {
                    #(#field_idents,)*
                })
            }
        }
    };

    expanded.into()
}

fn parse_meta(
    attrs: &[syn::Attribute],
    ty: &Type,
    info_lt: &syn::Lifetime,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("meta") {
            let args = attr.parse_args_with(
                syn::punctuated::Punctuated::<
                    syn::MetaNameValue,
                    syn::Token![,],
                >::parse_terminated,
            )?;

            // Named args only; values are passed in declaration order
            let values = args.iter().map(|kv| &kv.value);

            return Ok(quote! {
                <#ty as FromAccountView<#info_lt>>::Meta<'_>::new(
                    #(#values),*
                )
            });
        }
    }

    Ok(quote! { () })
}
