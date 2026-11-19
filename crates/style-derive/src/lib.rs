//! `#[derive(Style)]` for `xgpui-luma-style` (issue #282 spike).
//!
//! Struct attributes: `#[style(path = "checkbox", snap)]`.
//! - `path`: registers the style for tools; omit it for parts.
//! - `snap`: snap resolved `Px` fields to device pixels.
//!
//! Field attributes: `#[style(range = 0.0..=64.0, label = "Size", snap = false)]`.
//! - `range`: editor range for numeric fields.
//! - `snap`: override the struct's `snap` for this field.
//! - `label`: editor label; defaults to the field name in title case.
//!
//! Field types decide how a field is handled:
//! - `V<T>`: a value;
//! - `States<V<T>>`: a per-state value;
//! - `Computed<T>`: derived in Rust by a method of the same name,
//!   `fn name(&self, cx: &ResolveCx, at: StateSet) -> <T as Leaf>::Resolved`;
//!   read-only in editors and not settable by path;
//! - any other type: a nested part that must also derive `Style`.

use darling::ast::Data;
use darling::util::Ignored;
use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(FromDeriveInput)]
#[darling(attributes(style), supports(struct_named), forward_attrs(doc))]
struct StyleInput {
    ident: syn::Ident,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
    data: Data<Ignored, StyleField>,
    #[darling(default)]
    path: Option<String>,
    #[darling(default)]
    snap: bool,
}

#[derive(FromField)]
#[darling(attributes(style), forward_attrs(doc))]
struct StyleField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    range: Option<syn::Expr>,
    #[darling(default)]
    label: Option<String>,
    #[darling(default)]
    snap: Option<bool>,
}

enum Shape<'a> {
    /// `V<T>`: the inner `T`.
    Value(&'a syn::Type),
    /// `States<V<T>>`: the inner `T`.
    Stateful(&'a syn::Type),
    /// `Computed<T>`: the inner `T`.
    Computed(&'a syn::Type),
    /// Any other type: a nested style.
    Part,
}

/// The single generic argument of a path type whose last segment is `name`.
fn generic_arg<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    let syn::Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) if args.args.len() == 1 => Some(inner),
        _ => None,
    }
}

fn shape(ty: &syn::Type) -> darling::Result<Shape<'_>> {
    if let Some(inner) = generic_arg(ty, "V") {
        return Ok(Shape::Value(inner));
    }
    if let Some(inner) = generic_arg(ty, "Computed") {
        return Ok(Shape::Computed(inner));
    }
    if let Some(value) = generic_arg(ty, "States") {
        return generic_arg(value, "V")
            .map(Shape::Stateful)
            .ok_or_else(|| darling::Error::custom("`States` fields must hold `V<T>` values").with_span(ty));
    }
    Ok(Shape::Part)
}

fn docs(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(pair) if pair.path.is_ident("doc") => match &pair.value {
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(text), .. }) => Some(text.value().trim().to_owned()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(name: &str) -> String {
    name.split('_')
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(index, word)| {
            if index > 0 {
                return word.to_owned();
            }
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn range(expr: &syn::Expr) -> darling::Result<TokenStream> {
    match expr {
        syn::Expr::Range(syn::ExprRange {
            start: Some(start),
            end: Some(end),
            limits: syn::RangeLimits::Closed(_),
            ..
        }) => Ok(quote!(::core::option::Option::Some(((#start) as f32, (#end) as f32)))),
        other => Err(darling::Error::custom("expected a closed range such as `0.0..=64.0`").with_span(other)),
    }
}

#[proc_macro_derive(Style, attributes(style))]
pub fn derive_style(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    StyleInput::from_derive_input(&input)
        .and_then(expand)
        .unwrap_or_else(darling::Error::write_errors)
        .into()
}

fn expand(input: StyleInput) -> darling::Result<TokenStream> {
    let ls = quote!(::xgpui_luma_style);
    let StyleInput { ident, vis, attrs, data, path, snap } = input;
    let resolved = format_ident!("Resolved{ident}");
    let fields = data.take_struct().expect("darling checks struct_named").fields;

    let mut errors = darling::Error::accumulator();
    let mut resolved_fields = Vec::new();
    let mut resolve_body = Vec::new();
    let mut scope_arms = Vec::new();
    let mut descriptors = Vec::new();
    let mut handles = Vec::new();
    let mut lerp_body = Vec::new();
    let mut computed_markers = Vec::new();
    let mut get_arms = Vec::new();
    let mut set_arms = Vec::new();
    let mut describe_arms = Vec::new();
    let mut states_used = Vec::new();

    for field in &fields {
        let name = field.ident.as_ref().expect("named field");
        let field_vis = &field.vis;
        let key = name.to_string();
        let doc = docs(&field.attrs);
        let label = field.label.clone().unwrap_or_else(|| title_case(&key));
        let ty = &field.ty;
        let Some(shape) = errors.handle(shape(ty)) else {
            continue;
        };
        let range = match &field.range {
            Some(expr) => match errors.handle(range(expr)) {
                Some(range) => range,
                None => continue,
            },
            None => quote!(::core::option::Option::None),
        };
        lerp_body.push(quote!(#name: #ls::Lerp::lerp(&a.#name, &b.#name, t)));

        match shape {
            Shape::Computed(inner) => {
                resolved_fields.push(quote!(#[doc = #doc] #field_vis #name: <#inner as #ls::Leaf>::Resolved));
                resolve_body.push(quote!(#name: Self::#name(self, cx, at)));
                computed_markers.push(quote!(let _ = &self.#name;));
                describe_arms.push(quote!((#key, ::core::option::Option::None) => {
                    ::core::option::Option::Some(::std::string::String::from("computed"))
                }));
                descriptors.push(quote!(#ls::FieldDesc {
                    name: #key,
                    label: #label,
                    doc: #doc,
                    content: #ls::FieldContent::Value {
                        kind: <#inner as #ls::Leaf>::KIND,
                        range: #range,
                        stateful: false,
                        computed: true,
                    },
                }));
            }
            Shape::Value(inner) | Shape::Stateful(inner) => {
                let stateful = matches!(shape, Shape::Stateful(_));
                let value = if stateful {
                    quote!(self.#name.select(at))
                } else {
                    quote!(self.#name)
                };
                let snap = field.snap.unwrap_or(snap);
                if stateful {
                    states_used.push(quote!(
                        for selector in self.#name.selectors() {
                            used = used.union(selector);
                        }
                    ));
                }
                let handle = format_ident!("{}", key.to_uppercase());
                resolved_fields.push(quote!(#[doc = #doc] #field_vis #name: <#inner as #ls::Leaf>::Resolved));
                handles.push(quote!(
                    #[doc = concat!("Reference to `", #key, "` for expressions and state rules.")]
                    pub const #handle: #ls::Field<#inner> = #ls::Field::new(#key);
                ));
                {
                    resolve_body.push(quote!(
                        #name: #ls::Leaf::finish(#value.eval(cx, self, at, 0), cx, #snap)
                    ));
                    scope_arms.push(quote!(
                        #key => ::core::option::Option::Some(#ls::Leaf::to_dyn(
                            &#ls::Leaf::snapped(#value.eval(cx, self, at, depth), cx, #snap),
                        ))
                    ));
                    get_arms.push(quote!((#key, ::core::option::Option::None) => {
                        #ls::FieldScope::field(self, #key, cx, at, 0)
                    }));
                    let assign = if stateful {
                        quote!(self.#name.set(at, #ls::V::Lit(literal)))
                    } else {
                        quote!(self.#name = #ls::V::Lit(literal))
                    };
                    set_arms.push(quote!((#key, ::core::option::Option::None) => {
                        match <#inner as #ls::Leaf>::from_dyn(&value) {
                            ::core::option::Option::Some(literal) => {
                                #assign;
                                true
                            }
                            ::core::option::Option::None => false,
                        }
                    }));
                    describe_arms.push(quote!((#key, ::core::option::Option::None) => {
                        ::core::option::Option::Some(#value.describe())
                    }));
                }
                descriptors.push(quote!(#ls::FieldDesc {
                    name: #key,
                    label: #label,
                    doc: #doc,
                    content: #ls::FieldContent::Value {
                        kind: <#inner as #ls::Leaf>::KIND,
                        range: #range,
                        stateful: #stateful,
                        computed: false,
                    },
                }));
            }
            Shape::Part => {
                if field.range.is_some() {
                    errors.push(darling::Error::custom("parts take no `range`").with_span(ty));
                    continue;
                }
                resolved_fields.push(quote!(#[doc = #doc] #field_vis #name: <#ty as #ls::Style>::Resolved));
                resolve_body.push(quote!(#name: #ls::Style::resolve(&self.#name, cx, at)));
                states_used.push(quote!(used = used.union(#ls::Style::states_used(&self.#name));));
                get_arms.push(quote!((#key, ::core::option::Option::Some(rest)) => {
                    #ls::Style::get(&self.#name, rest, cx, at)
                }));
                set_arms.push(quote!((#key, ::core::option::Option::Some(rest)) => {
                    #ls::Style::set(&mut self.#name, rest, at, value)
                }));
                describe_arms.push(quote!((#key, ::core::option::Option::Some(rest)) => {
                    #ls::Style::describe(&self.#name, rest, at)
                }));
                descriptors.push(quote!(#ls::FieldDesc {
                    name: #key,
                    label: #label,
                    doc: #doc,
                    content: #ls::FieldContent::Part(<#ty as #ls::Style>::SHAPE),
                }));
            }
        }
    }
    errors.finish()?;

    let style_doc = docs(&attrs);
    let name = path.clone().unwrap_or_default();
    let register = path.map(|_| quote!(#ls::inventory::submit! { #ls::Registered(<#ident as #ls::Style>::SHAPE) }));
    let resolved_doc = format!("Resolved [`{ident}`]: plain values for templates.");

    Ok(quote! {
        #[doc = #resolved_doc]
        #[derive(Clone, Debug, PartialEq)]
        #vis struct #resolved {
            #(#resolved_fields,)*
        }

        impl #ident {
            #(#handles)*
        }

        impl #ls::FieldScope for #ident {
            #[allow(unused_variables)]
            fn field(
                &self,
                name: &str,
                cx: &#ls::ResolveCx<'_>,
                at: #ls::StateSet,
                depth: u8,
            ) -> ::core::option::Option<#ls::Dyn> {
                match name {
                    #(#scope_arms,)*
                    _ => ::core::option::Option::None,
                }
            }
        }

        impl #ls::Style for #ident {
            type Resolved = #resolved;
            const SHAPE: &'static #ls::StyleShape = &#ls::StyleShape {
                name: #name,
                type_name: stringify!(#ident),
                doc: #style_doc,
                fields: &[#(#descriptors,)*],
            };

            fn resolve(&self, cx: &#ls::ResolveCx<'_>, at: #ls::StateSet) -> #resolved {
                // `Computed` fields hold no data; touching them keeps dead-code lints quiet.
                #(#computed_markers)*
                #resolved { #(#resolve_body,)* }
            }

            #[allow(unused_variables)]
            fn get(
                &self,
                path: &str,
                cx: &#ls::ResolveCx<'_>,
                at: #ls::StateSet,
            ) -> ::core::option::Option<#ls::Dyn> {
                match #ls::split_path(path) {
                    #(#get_arms)*
                    _ => ::core::option::Option::None,
                }
            }

            #[allow(unused_variables)]
            fn set(&mut self, path: &str, at: #ls::StateSet, value: #ls::Dyn) -> bool {
                match #ls::split_path(path) {
                    #(#set_arms)*
                    _ => false,
                }
            }

            #[allow(unused_variables)]
            fn describe(&self, path: &str, at: #ls::StateSet) -> ::core::option::Option<::std::string::String> {
                match #ls::split_path(path) {
                    #(#describe_arms)*
                    _ => ::core::option::Option::None,
                }
            }

            #[allow(unused_mut)]
            fn states_used(&self) -> #ls::StateSet {
                let mut used = #ls::StateSet::NONE;
                #(#states_used)*
                used
            }
        }

        impl #ls::Lerp for #resolved {
            fn lerp(a: &Self, b: &Self, t: f32) -> Self {
                Self { #(#lerp_body,)* }
            }
        }

        #register
    })
}
