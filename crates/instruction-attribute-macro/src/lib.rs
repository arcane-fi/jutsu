// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, LitStr, Pat, PatIdent, PatType, ReturnType, Type,
};

#[proc_macro_attribute]
pub fn instruction(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mode = match parse_mode(attr) {
        Ok(m) => m,
        Err(ts) => return ts,
    };

    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let ix_name = to_pascal_case(fn_name);
    let struct_name = with_instruction_ident(ix_name.clone());
    let struct_ident = format_ident!("{struct_name}");

    let expanded = match mode {
        IxMode::Pod => expand_pod(&input_fn, &ix_name, &struct_ident),
        IxMode::EnumTail => expand_enum_tail(&input_fn, &ix_name, &struct_ident),
    };

    expanded.into()
}

#[derive(Debug, Clone, Copy)]
enum IxMode {
    Pod,
    EnumTail,
}

fn parse_mode(attr: TokenStream) -> Result<IxMode, TokenStream> {
    if attr.is_empty() {
        return Ok(IxMode::Pod);
    }

    let ident = match syn::parse::<Ident>(attr) {
        Ok(id) => id,
        Err(e) => return Err(e.to_compile_error().into()),
    };

    match ident.to_string().as_str() {
        "pod" => Ok(IxMode::Pod),
        "enum_tail" => Ok(IxMode::EnumTail),
        other => {
            let err = syn::Error::new_spanned(
                ident,
                format!("unknown instruction mode `{}` (expected `pod` or `enum_tail`)", other),
            );
            Err(err.to_compile_error().into())
        }
    }
}

fn expand_pod(input_fn: &ItemFn, ix_name: &str, struct_ident: &Ident) -> proc_macro2::TokenStream {
    let mut fields = Vec::new();

    for (i, arg) in input_fn.sig.inputs.iter().enumerate() {
        if i == 0 {
            // Expect ctx: Context<...>, skip the first arg
            continue;
        }

        let (ident, ty) = match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident = match pat.as_ref() {
                    Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                    _ => {
                        return syn::Error::new_spanned(
                            pat,
                            "expected a simple identifier pattern like `some_data: u64`",
                        )
                        .to_compile_error();
                    }
                };
                (ident, ty.as_ref().clone())
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(arg, "methods are not supported")
                    .to_compile_error();
            }
        };

        // Reject references in instruction data for POD
        if contains_reference(&ty) {
            return syn::Error::new_spanned(
                ty,
                "instruction arguments must be owned types (no references) to derive Pod/Zeroable safely",
            )
            .to_compile_error();
        }

        fields.push((ident, ty));
    }

    if !returns_result_unit(&input_fn.sig.output) {
        return syn::Error::new_spanned(
            &input_fn.sig.output,
            "expected return type `Result<()>` or equivalent",
        )
        .to_compile_error();
    }

    let field_defs = fields.iter().map(|(ident, ty)| quote! { pub #ident: #ty, });

    let mut instrumented_fn = input_fn.clone();
    let msg = LitStr::new(&format!("Instruction: {}", ix_name), Span::call_site());
    instrumented_fn
        .block
        .stmts
        .insert(0, syn::parse_quote! { log!(#msg); });

    quote! {
        #instrumented_fn

        #[derive(Pod, Zeroable, Discriminator, Clone, Copy)]
        #[repr(C)]
        pub struct #struct_ident {
            #(#field_defs)*
        }
    }
}

fn expand_enum_tail(
    input_fn: &ItemFn,
    ix_name: &str,
    struct_ident: &Ident,
) -> proc_macro2::TokenStream {
    // we expect exactly: ctx, enum, tail
    if input_fn.sig.inputs.len() != 3 {
        return syn::Error::new_spanned(
            &input_fn.sig.inputs,
            "#[instruction(enum_tail)] expects `fn(ctx, enum: T, tail: &[u8])`",
        )
        .to_compile_error();
    }

    // arg[0] is ctx, skip it
    let enum_arg = &input_fn.sig.inputs[1];
    let tail_arg = &input_fn.sig.inputs[2];

    let (enum_ident, enum_ty) = match enum_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let ident = match pat.as_ref() {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => {
                    return syn::Error::new_spanned(
                        pat,
                        "expected simple identifier pattern for enum argument",
                    )
                    .to_compile_error();
                }
            };
            (ident, ty.as_ref().clone())
        }
        _ => {
            return syn::Error::new_spanned(enum_arg, "unexpected argument pattern")
                .to_compile_error();
        }
    };

    let (tail_ident, tail_ty) = match tail_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let ident = match pat.as_ref() {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => {
                    return syn::Error::new_spanned(
                        pat,
                        "expected simple identifier pattern for tail argument",
                    )
                    .to_compile_error();
                }
            };
            (ident, ty.as_ref().clone())
        }
        _ => {
            return syn::Error::new_spanned(tail_arg, "unexpected argument pattern")
                .to_compile_error();
        }
    };

    // Ensure tail is &[u8]
    if !is_ref_slice_u8(&tail_ty) {
        return syn::Error::new_spanned(
            &tail_ty,
            "tail must be of type `&[u8]` for #[instruction(enum_tail)]",
        )
        .to_compile_error();
    }

    if !returns_result_unit(&input_fn.sig.output) {
        return syn::Error::new_spanned(
            &input_fn.sig.output,
            "expected return type `Result<()>` (or equivalent)",
        )
        .to_compile_error();
    }

    let mut instrumented_fn = input_fn.clone();
    let msg = LitStr::new(&format!("Instruction: {}", ix_name), Span::call_site());
    instrumented_fn
        .block
        .stmts
        .insert(0, syn::parse_quote! { log!(#msg); });

    quote! {
        #instrumented_fn

        #[derive(Discriminator)]
        pub struct #struct_ident<'a> {
            pub #enum_ident: #enum_ty,
            pub #tail_ident: &'a [u8],
        }

        impl<'a> DecodeIx<'a> for #struct_ident<'a> {
            type Target = Self;

            fn decode(bytes: &'a [u8]) -> Result<Self::Target> {
                let (first, tail) = bytes
                    .split_first()
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let #enum_ident = <#enum_ty as core::convert::TryFrom<u8>>::try_from(*first)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self {
                    #enum_ident,
                    #tail_ident: tail,
                })
            }
        }
    }
}

fn is_ref_slice_u8(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => match r.elem.as_ref() {
            Type::Slice(s) => match s.elem.as_ref() {
                Type::Path(p) if p.path.is_ident("u8") => true,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

fn to_pascal_case(ident: &Ident) -> String {
    let s = ident.to_string();
    let parts = s.split('_').filter(|p| !p.is_empty());
    let mut out = String::new();
    for p in parts {
        let mut chars = p.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars.flat_map(|c| c.to_lowercase()));
        }
    }

    out
}

fn with_instruction_ident(mut str: String) -> String {
    if !str.ends_with("Instruction") {
        str.push_str("Instruction");
    }

    str
}

fn returns_result_unit(ret: &ReturnType) -> bool {
    match ret {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => {
            let s = quote!(#ty).to_string().replace(' ', "");
            s == "Result<()>" || s.ends_with("::Result<()>")
        }
    }
}

fn contains_reference(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Array(a) => contains_reference(&a.elem),
        Type::Group(g) => contains_reference(&g.elem),
        Type::Paren(p) => contains_reference(&p.elem),
        Type::Path(_) => false, // could recurse generics, but keep simple
        _ => false,
    }
}
