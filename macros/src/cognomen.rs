//! `#[derive(Cognomen)]` implementation.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parenthesized, Attribute, Data, DeriveInput, Fields, Ident, Result, Token};

const STYLE_HELP: &str = "snake_case|snake|kebab_case|kebab-case|kebab|camelCase|camel|PascalCase|pascal|SCREAMING_SNAKE_CASE|screaming|lower|upper|title|title_case";

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Lower,
    Upper,
    Title,
}

impl CaseStyle {
    fn from_str_style(s: &str) -> Option<Self> {
        Some(match s {
            "snake_case" | "snake" => Self::Snake,
            "kebab_case" | "kebab-case" | "kebab" => Self::Kebab,
            "camelCase" | "camel_case" | "camel" => Self::Camel,
            "PascalCase" | "pascal_case" | "pascal" => Self::Pascal,
            "SCREAMING_SNAKE_CASE" | "screaming_snake_case" | "screaming" => Self::ScreamingSnake,
            "lower" | "lowercase" => Self::Lower,
            "upper" | "uppercase" => Self::Upper,
            "title" | "title_case" | "TitleCase" => Self::Title,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Snake => "snake_case",
            Self::Kebab => "kebab-case",
            Self::Camel => "camelCase",
            Self::Pascal => "PascalCase",
            Self::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Title => "title",
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Self::Snake => "snake",
            Self::Kebab => "kebab",
            Self::Camel => "camel",
            Self::Pascal => "pascal",
            Self::ScreamingSnake => "screaming_snake",
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Title => "title",
        }
    }

    fn convert(self, pascal_ident: &str) -> String {
        let words = split_pascal_words(pascal_ident);
        let mut out = String::new();
        for (i, w) in words.iter().enumerate() {
            if i > 0 {
                match self {
                    Self::Snake | Self::ScreamingSnake => out.push('_'),
                    Self::Kebab => out.push('-'),
                    Self::Title => out.push(' '),
                    Self::Camel | Self::Pascal | Self::Lower | Self::Upper => {}
                }
            }
            match self {
                Self::Snake | Self::Kebab | Self::Lower => out.push_str(&w.to_ascii_lowercase()),
                Self::ScreamingSnake | Self::Upper => out.push_str(&w.to_ascii_uppercase()),
                Self::Camel if i == 0 => out.push_str(&w.to_ascii_lowercase()),
                Self::Camel | Self::Pascal | Self::Title => out.push_str(&capitalize(w)),
            }
        }
        out
    }
}

/// `#[cognomen(snake_case)]` or `#[cognomen(snake_case, kebab-case, prefix = "...")]`.
///
/// First case listed is the default returned by `label()`.
/// Extra methods: any `name = "..."` (or `name()`) besides reserved keys.
struct CognomenAttr {
    styles: Vec<CaseStyle>,
    prefix: String,
    crate_path: syn::Path,
    extras: Vec<(String, ExtraDecl)>,
}

struct ExtraDecl {
    /// `Some` when the enum sets `name = "..."` / `name()`; `None` falls back
    /// to each variant's default label (`as_str` / `label`).
    default: Option<String>,
    span: Span,
}

fn parse_case_style(input: ParseStream<'_>) -> Result<(CaseStyle, Span)> {
    // A style may be multi-token: kebab-case is `Ident - Ident`.
    let first: Ident = input.parse()?;
    let span = first.span();
    if input.peek(Token![-]) {
        input.parse::<Token![-]>()?;
        let second: Ident = input.parse()?;
        let joined = format!("{first}-{second}");
        let style = CaseStyle::from_str_style(&joined)
            .ok_or_else(|| syn::Error::new(span, format!("unknown cognomen style `{joined}`")))?;
        return Ok((style, span));
    }
    let s = first.to_string();
    let style = CaseStyle::from_str_style(&s).ok_or_else(|| {
        syn::Error::new(
            span,
            format!("unknown cognomen case style; expected {STYLE_HELP}"),
        )
    })?;
    Ok((style, span))
}

fn parse_eq_litstr(input: ParseStream<'_>) -> Result<(Ident, syn::LitStr)> {
    let key: Ident = input.parse()?;
    input.parse::<Token![=]>()?;
    Ok((key, input.parse()?))
}

fn parse_key_litstr(input: ParseStream<'_>) -> Result<(Ident, syn::LitStr)> {
    let key: Ident = input.parse()?;
    if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        return Ok((key, input.parse()?));
    }
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        let value: syn::LitStr = content.parse()?;
        if !content.is_empty() {
            return Err(content.error("expected a single string literal"));
        }
        return Ok((key, value));
    }
    Err(syn::Error::new(
        key.span(),
        "expected `key = \"...\"` or `key(\"...\")`",
    ))
}

fn insert_extra(extras: &mut Vec<(String, ExtraDecl)>, key: Ident, default: String) -> Result<()> {
    let name = key.to_string();
    if name == "prefix" || name == "rename" {
        return Err(syn::Error::new(
            key.span(),
            format!("`{name}` is not an extra method"),
        ));
    }
    if !is_ascii_ident(&name) {
        return Err(syn::Error::new(
            key.span(),
            "extra method name must be a non-empty ASCII identifier",
        ));
    }
    if extras.iter().any(|(n, _)| n == &name) {
        return Err(syn::Error::new(
            key.span(),
            format!("duplicate cognomen extra `{name}`"),
        ));
    }
    extras.push((
        name,
        ExtraDecl {
            default: Some(default),
            span: key.span(),
        },
    ));
    Ok(())
}

fn reserved_idents(prefix: &str, styles: &[CaseStyle]) -> Vec<String> {
    let mut out = vec![
        String::from("label"),
        String::from("as_str"),
        String::from("from_label"),
        String::from("VARIANTS"),
        String::from("LABELS"),
        String::from("eq"),
        String::from("ne"),
        String::from("fmt"),
        String::from("as_ref"),
        String::from("try_from"),
        String::from("from_str"),
        String::from("serialize"),
        String::from("deserialize"),
    ];
    for style in styles {
        out.push(format!("{}_{}", prefix, style.suffix()));
    }
    out
}

fn check_extras(prefix: &str, styles: &[CaseStyle], extras: &[(String, ExtraDecl)]) -> Result<()> {
    let reserved = reserved_idents(prefix, styles);
    for (name, decl) in extras {
        if syn::parse_str::<Ident>(name).is_err() {
            return Err(syn::Error::new(
                decl.span,
                format!("extra method `{name}` is not a valid Rust identifier"),
            ));
        }
        if reserved.iter().any(|r| r == name) {
            return Err(syn::Error::new(
                decl.span,
                format!("extra method `{name}` conflicts with a generated cognomen item"),
            ));
        }
    }
    Ok(())
}

fn set_once<T>(slot: &mut Option<T>, value: T, span: Span, msg: &str) -> Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(span, msg));
    }
    *slot = Some(value);
    Ok(())
}

fn eat_comma(input: ParseStream<'_>) -> Result<()> {
    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

impl Parse for CognomenAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut styles = Vec::new();
        let mut prefix = None;
        let mut crate_path = None;
        let mut extras = Vec::new();

        while !input.is_empty() {
            if input.peek(Token![crate]) && input.peek2(Token![=]) {
                let crate_tok: Token![crate] = input.parse()?;
                input.parse::<Token![=]>()?;
                set_once(
                    &mut crate_path,
                    input.parse()?,
                    crate_tok.span,
                    "duplicate cognomen crate",
                )?;
            } else if input.peek(syn::Ident) && input.peek2(syn::token::Paren) {
                let key: Ident = input.parse()?;
                let content;
                parenthesized!(content in input);
                let default = if content.is_empty() {
                    String::new()
                } else {
                    let value: syn::LitStr = content.parse()?;
                    if !content.is_empty() {
                        return Err(content.error("expected a single string literal"));
                    }
                    value.value()
                };
                insert_extra(&mut extras, key, default)?;
            } else if input.peek(syn::Ident) && input.peek2(Token![=]) {
                let (key, value) = parse_eq_litstr(input)?;
                if key == "prefix" {
                    let text = value.value();
                    set_once(
                        &mut prefix,
                        text.clone(),
                        key.span(),
                        "duplicate cognomen prefix",
                    )?;
                    if !is_ascii_ident(&text) {
                        return Err(syn::Error::new(
                            value.span(),
                            "prefix must be a non-empty ASCII identifier (e.g. prefix = \"label\")",
                        ));
                    }
                } else if key == "rename" {
                    return Err(syn::Error::new(
                        key.span(),
                        "rename is only valid on a variant (e.g. #[cognomen(rename = \"io_error\")])",
                    ));
                } else {
                    insert_extra(&mut extras, key, value.value())?;
                }
            } else {
                let (style, span) = parse_case_style(input)?;
                if styles.contains(&style) {
                    return Err(syn::Error::new(span, "duplicate cognomen case style"));
                }
                styles.push(style);
            }
            eat_comma(input)?;
        }

        if styles.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "missing cognomen case style (e.g. #[cognomen(snake_case)])",
            ));
        }

        Ok(Self {
            styles,
            prefix: prefix.unwrap_or_else(|| String::from("label")),
            crate_path: crate_path.unwrap_or_else(|| syn::parse_quote!(::cognomen)),
            extras,
        })
    }
}

struct VariantAttr {
    rename: Option<syn::LitStr>,
    extras: Vec<(String, syn::LitStr)>,
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut rename = None;
        let mut extras = Vec::new();
        while !input.is_empty() {
            let (key, value) = parse_key_litstr(input)?;
            if key == "rename" {
                if value.value().is_empty() {
                    return Err(syn::Error::new(
                        value.span(),
                        "cognomen rename must not be empty",
                    ));
                }
                set_once(&mut rename, value, key.span(), "duplicate cognomen rename")?;
            } else {
                let name = key.to_string();
                if extras.iter().any(|(n, _)| n == &name) {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("duplicate extra `{name}` on variant"),
                    ));
                }
                extras.push((name, value));
            }
            eat_comma(input)?;
        }
        Ok(Self { rename, extras })
    }
}

struct Variant<'a> {
    ident: &'a Ident,
    rename: Option<String>,
    extras: BTreeMap<String, String>,
}

impl Variant<'_> {
    fn case_label(&self, style: CaseStyle) -> String {
        style.convert(&self.ident.to_string())
    }

    fn default_label(&self, default: CaseStyle) -> String {
        self.rename
            .clone()
            .unwrap_or_else(|| self.case_label(default))
    }

    fn all_labels(&self, styles: &[CaseStyle]) -> Vec<String> {
        let mut labels: Vec<String> = styles.iter().map(|s| self.case_label(*s)).collect();
        if let Some(r) = &self.rename {
            labels.push(r.clone());
        }
        labels.sort();
        labels.dedup();
        labels
    }
}

fn split_pascal_words(s: &str) -> Vec<String> {
    // ponytail: ASCII camel split only. Digits stay glued (Utf8 -> utf8, IPv4 -> i_pv4).
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut prev: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_uppercase() && !cur.is_empty() {
            let prev_lower = prev.is_some_and(|p| p.is_lowercase());
            let next_lower = chars.peek().is_some_and(|n| n.is_lowercase());
            if prev_lower || next_lower {
                words.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c);
        prev = Some(c);
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn is_ascii_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {
            chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
        }
        _ => false,
    }
}

fn capitalize(w: &str) -> String {
    let mut cs = w.chars();
    match cs.next() {
        None => String::new(),
        Some(f) => f.to_ascii_uppercase().to_string() + &cs.as_str().to_ascii_lowercase(),
    }
}

fn find_cognomen_attr<'a>(
    attrs: &'a [Attribute],
    duplicate_msg: &str,
) -> Result<Option<&'a Attribute>> {
    let mut found = None;
    for attr in attrs {
        if !attr.path().is_ident("cognomen") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new(attr.span(), duplicate_msg));
        }
        found = Some(attr);
    }
    Ok(found)
}

fn ensure_extra(extras: &mut Vec<(String, ExtraDecl)>, name: String, span: Span) {
    if extras.iter().any(|(n, _)| n == &name) {
        return;
    }
    extras.push((
        name,
        ExtraDecl {
            default: None,
            span,
        },
    ));
}

fn parse_variant_meta(
    variant: &syn::Variant,
    declared: &mut Vec<(String, ExtraDecl)>,
) -> Result<(Option<String>, BTreeMap<String, String>)> {
    let mut extras = BTreeMap::new();
    let mut rename = None;

    if let Some(attr) = find_cognomen_attr(&variant.attrs, "duplicate #[cognomen(...)] on variant")?
    {
        let parsed: VariantAttr = attr.parse_args()?;
        if parsed.rename.is_none() && parsed.extras.is_empty() {
            return Err(syn::Error::new(
                attr.span(),
                "variant #[cognomen(...)] requires rename = \"...\" or an extra method",
            ));
        }
        rename = parsed.rename.map(|l| l.value());
        for (name, lit) in parsed.extras {
            if extras.contains_key(&name) {
                return Err(syn::Error::new(
                    lit.span(),
                    format!("duplicate extra `{name}` on variant"),
                ));
            }
            ensure_extra(declared, name.clone(), lit.span());
            extras.insert(name, lit.value());
        }
    }

    Ok((rename, extras))
}

fn unit_variants<'a>(
    input: &'a DeriveInput,
    declared: &mut Vec<(String, ExtraDecl)>,
) -> Result<Vec<Variant<'a>>> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.ident.span(),
            "Cognomen can only be derived for enums",
        ));
    };

    let mut variants = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Cognomen only supports unit variants (no fields)",
            ));
        }
        let (rename, extras) = parse_variant_meta(variant, declared)?;
        variants.push(Variant {
            ident: &variant.ident,
            rename,
            extras,
        });
    }
    if variants.is_empty() {
        return Err(syn::Error::new(
            input.ident.span(),
            "Cognomen enum must have at least one variant",
        ));
    }
    Ok(variants)
}

fn reverse_eq_arms(
    name: &Ident,
    variants: &[Variant<'_>],
    styles: &[CaseStyle],
) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<String>)> {
    let mut owner = BTreeMap::<String, &Ident>::new();
    let mut reverse_arms = Vec::new();
    let mut eq_arms = Vec::new();

    for v in variants {
        let labels = v.all_labels(styles);
        let lits: Vec<syn::LitStr> = labels
            .iter()
            .map(|label| syn::LitStr::new(label, v.ident.span()))
            .collect();
        for label in &labels {
            if let Some(prev) = owner.insert(label.clone(), v.ident) {
                if prev != v.ident {
                    return Err(syn::Error::new(
                        v.ident.span(),
                        format!("generated label `{label}` is shared by multiple variants"),
                    ));
                }
            }
        }
        let ident = v.ident;
        reverse_arms.push(quote! { #(#lits)|* => ::core::result::Result::Ok(#name::#ident) });
        eq_arms.push(quote! { Self::#ident => matches!(other, #(#lits)|*) });
    }

    Ok((reverse_arms, eq_arms, owner.into_keys().collect()))
}

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;

    let attr = match find_cognomen_attr(&input.attrs, "duplicate #[cognomen(...)] attribute")? {
        Some(a) => a.parse_args::<CognomenAttr>()?,
        None => {
            return Err(syn::Error::new(
                name.span(),
                "missing #[cognomen(<case>)] container attribute (e.g. #[cognomen(snake_case)])",
            ));
        }
    };

    let mut extras = attr.extras;
    let variants = unit_variants(&input, &mut extras)?;
    check_extras(&attr.prefix, &attr.styles, &extras)?;
    let crate_path = &attr.crate_path;
    let default = attr.styles[0];
    let idents: Vec<&Ident> = variants.iter().map(|v| v.ident).collect();
    let default_labels: Vec<String> = variants.iter().map(|v| v.default_label(default)).collect();

    let case_methods = attr.styles.iter().map(|style| {
        let method = format_ident!("{}_{}", attr.prefix, style.suffix());
        let doc = format!(
            "Stable label for this variant in the `{}` case.",
            style.name()
        );
        let arms = variants.iter().map(|v| {
            let ident = v.ident;
            let label = v.case_label(*style);
            quote! { Self::#ident => #label }
        });
        quote! {
            #[doc = #doc]
            #[inline]
            #[must_use]
            pub const fn #method(&self) -> &'static str {
                match self { #(#arms,)* }
            }
        }
    });

    let extra_methods = extras.iter().map(|(name, decl)| {
        let method = format_ident!("{name}");
        let doc = format!("Extra string for this variant (`{name}`).");
        let arms = variants.iter().map(|v| {
            let ident = v.ident;
            let text = v.extras.get(name).cloned().unwrap_or_else(|| {
                decl.default
                    .clone()
                    .unwrap_or_else(|| v.default_label(default))
            });
            quote! { Self::#ident => #text }
        });
        quote! {
            #[doc = #doc]
            #[inline]
            #[must_use]
            pub const fn #method(&self) -> &'static str {
                match self { #(#arms,)* }
            }
        }
    });

    let label_arms = variants
        .iter()
        .zip(default_labels.iter())
        .map(|(v, label)| {
            let ident = v.ident;
            quote! { Self::#ident => #label }
        });

    let (reverse_arms, eq_arms, all_labels) = reverse_eq_arms(name, &variants, &attr.styles)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let variants_const = input.generics.params.is_empty().then(|| {
        quote! {
            /// All variants in declaration order.
            pub const VARIANTS: &'static [Self] = &[#(Self::#idents,)*];

            /// Default [`Self::label`] for each variant in declaration order.
            pub const LABELS: &'static [&'static str] = &[#(#default_labels,)*];
        }
    });

    let parse_impls = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Parse any declared-case label, or a variant `rename`, into `Self`.
            ///
            /// Equivalent to [`TryFrom::try_from`](core::convert::TryFrom).
            #[inline]
            pub fn from_label(s: &str) -> ::core::result::Result<Self, #crate_path::FromLabelError> {
                ::core::convert::TryFrom::try_from(s)
            }
        }

        impl #impl_generics ::core::convert::TryFrom<&str> for #name #ty_generics #where_clause {
            type Error = #crate_path::FromLabelError;
            #[inline]
            fn try_from(s: &str) -> ::core::result::Result<Self, Self::Error> {
                match s {
                    #(#reverse_arms,)*
                    _ => ::core::result::Result::Err(#crate_path::FromLabelError::new(s)),
                }
            }
        }

        impl #impl_generics ::core::str::FromStr for #name #ty_generics #where_clause {
            type Err = #crate_path::FromLabelError;
            #[inline]
            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                ::core::convert::TryFrom::try_from(s)
            }
        }
    };

    let serde_impls = cfg!(feature = "serde").then(|| {
        quote! {
            impl #impl_generics #crate_path::__serde::Serialize for #name #ty_generics #where_clause {
                fn serialize<S: #crate_path::__serde::Serializer>(
                    &self,
                    serializer: S,
                ) -> ::core::result::Result<S::Ok, S::Error> {
                    #crate_path::__serde::Serialize::serialize(self.label(), serializer)
                }
            }

            impl<'de> #crate_path::__serde::Deserialize<'de> for #name #ty_generics #where_clause {
                fn deserialize<D: #crate_path::__serde::Deserializer<'de>>(
                    deserializer: D,
                ) -> ::core::result::Result<Self, D::Error> {
                    struct __CognomenVisitor;
                    impl<'de> #crate_path::__serde::de::Visitor<'de> for __CognomenVisitor {
                        type Value = #name #ty_generics;
                        fn expecting(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            f.write_str("a cognomen label")
                        }
                        fn visit_str<E: #crate_path::__serde::de::Error>(
                            self,
                            v: &str,
                        ) -> ::core::result::Result<Self::Value, E> {
                            match v {
                                #(#reverse_arms,)*
                                _ => ::core::result::Result::Err(
                                    E::unknown_variant(v, &[#(#all_labels,)*]),
                                ),
                            }
                        }
                        fn visit_borrowed_str<E: #crate_path::__serde::de::Error>(
                            self,
                            v: &'de str,
                        ) -> ::core::result::Result<Self::Value, E> {
                            self.visit_str(v)
                        }
                    }
                    deserializer.deserialize_str(__CognomenVisitor)
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #variants_const

            /// Stable label for this variant in the default (first) case.
            ///
            /// Overridden by `#[cognomen(rename = "...")]` on the variant.
            #[inline]
            #[must_use]
            pub const fn label(&self) -> &'static str {
                match self { #(#label_arms,)* }
            }

            /// Alias of [`Self::label`].
            #[inline]
            #[must_use]
            pub const fn as_str(&self) -> &'static str {
                self.label()
            }

            #(#case_methods)*

            #(#extra_methods)*
        }

        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.label())
            }
        }

        impl #impl_generics ::core::convert::AsRef<str> for #name #ty_generics #where_clause {
            #[inline]
            fn as_ref(&self) -> &str {
                self.label()
            }
        }

        impl #impl_generics ::core::cmp::PartialEq<str> for #name #ty_generics #where_clause {
            #[inline]
            fn eq(&self, other: &str) -> bool {
                match self { #(#eq_arms,)* }
            }
        }

        impl #impl_generics ::core::cmp::PartialEq<&str> for #name #ty_generics #where_clause {
            #[inline]
            fn eq(&self, other: &&str) -> bool {
                *self == **other
            }
        }

        impl #impl_generics ::core::cmp::PartialEq<#name #ty_generics> for str #where_clause {
            #[inline]
            fn eq(&self, other: &#name #ty_generics) -> bool {
                other == self
            }
        }

        impl #impl_generics ::core::cmp::PartialEq<#name #ty_generics> for &str #where_clause {
            #[inline]
            fn eq(&self, other: &#name #ty_generics) -> bool {
                other == *self
            }
        }

        #parse_impls
        #serde_impls
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_acronyms_and_singles() {
        assert_eq!(split_pascal_words("HTTPResponse"), ["HTTP", "Response"]);
        assert_eq!(split_pascal_words("WebSocket"), ["Web", "Socket"]);
        assert_eq!(split_pascal_words("A"), ["A"]);
        assert_eq!(split_pascal_words("ABC"), ["ABC"]);
        assert_eq!(split_pascal_words("Utf8"), ["Utf8"]);
        assert_eq!(split_pascal_words("parseHTML"), ["parse", "HTML"]);
        assert_eq!(split_pascal_words("IPv4"), ["I", "Pv4"]);
    }

    #[test]
    fn convert_cases() {
        assert_eq!(CaseStyle::Snake.convert("HTTPResponse"), "http_response");
        assert_eq!(CaseStyle::Kebab.convert("WebSocket"), "web-socket");
        assert_eq!(CaseStyle::Camel.convert("SingleProcess"), "singleProcess");
        assert_eq!(CaseStyle::Pascal.convert("SingleProcess"), "SingleProcess");
        assert_eq!(CaseStyle::Pascal.convert("HTTPResponse"), "HttpResponse");
        assert_eq!(
            CaseStyle::ScreamingSnake.convert("SingleProcess"),
            "SINGLE_PROCESS"
        );
        assert_eq!(CaseStyle::Lower.convert("Zero"), "zero");
        assert_eq!(CaseStyle::Upper.convert("Zero"), "ZERO");
        assert_eq!(CaseStyle::Snake.convert("Utf8"), "utf8");
        assert_eq!(CaseStyle::Title.convert("SingleProcess"), "Single Process");
    }

    #[test]
    fn prefix_ident() {
        assert!(is_ascii_ident("label"));
        assert!(is_ascii_ident("my_label"));
        assert!(is_ascii_ident("_x"));
        assert!(!is_ascii_ident(""));
        assert!(!is_ascii_ident("my-label"));
        assert!(!is_ascii_ident("1abc"));
        assert!(!is_ascii_ident("a.b"));
    }
}
