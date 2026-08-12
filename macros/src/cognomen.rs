//! `#[derive(Cognomen)]` implementation.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Result, Token};

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
        match s {
            "snake_case" | "snake" => Some(Self::Snake),
            "kebab_case" | "kebab-case" | "kebab" => Some(Self::Kebab),
            "camelCase" | "camel_case" | "camel" => Some(Self::Camel),
            "PascalCase" | "pascal_case" | "pascal" => Some(Self::Pascal),
            "SCREAMING_SNAKE_CASE" | "screaming_snake_case" | "screaming" => {
                Some(Self::ScreamingSnake)
            }
            "lower" | "lowercase" => Some(Self::Lower),
            "upper" | "uppercase" => Some(Self::Upper),
            "title" | "title_case" | "TitleCase" => Some(Self::Title),
            _ => None,
        }
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

    fn method_name(self, prefix: &str) -> String {
        format!("{prefix}_{}", self.suffix())
    }

    fn convert(self, pascal_ident: &str) -> String {
        let words = split_pascal_words(pascal_ident);
        match self {
            Self::Snake | Self::Kebab => {
                let sep = if matches!(self, Self::Snake) {
                    '_'
                } else {
                    '-'
                };
                join_lower(&words, sep)
            }
            Self::ScreamingSnake => join_upper(&words, '_'),
            Self::Camel => {
                let mut out = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i == 0 {
                        out.push_str(&w.to_ascii_lowercase());
                    } else {
                        out.push_str(&capitalize(w));
                    }
                }
                out
            }
            Self::Pascal => words.iter().map(|w| capitalize(w)).collect(),
            Self::Lower => words.iter().map(|w| w.to_ascii_lowercase()).collect(),
            Self::Upper => words.iter().map(|w| w.to_ascii_uppercase()).collect(),
            Self::Title => {
                let mut out = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(&capitalize(w));
                }
                out
            }
        }
    }
}

/// `#[cognomen(snake_case)]` or `#[cognomen(snake_case, kebab-case, prefix = "...")]`.
///
/// First case listed is the default returned by `label()`.
struct CognomenAttr {
    styles: Vec<CaseStyle>,
    default: CaseStyle,
    prefix: String,
    crate_path: syn::Path,
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
        Ok((style, span))
    } else {
        let s = first.to_string();
        let style = CaseStyle::from_str_style(&s).ok_or_else(|| {
            syn::Error::new(
                span,
                format!("unknown cognomen case style; expected {STYLE_HELP}"),
            )
        })?;
        Ok((style, span))
    }
}

impl Parse for CognomenAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut styles = Vec::new();
        let mut prefix = String::from("label");
        let mut seen_prefix = false;
        let mut crate_path = None;
        let mut seen_crate = false;

        while !input.is_empty() {
            if input.peek(Token![crate]) && input.peek2(Token![=]) {
                let crate_tok: Token![crate] = input.parse()?;
                input.parse::<Token![=]>()?;
                if seen_crate {
                    return Err(syn::Error::new(crate_tok.span, "duplicate cognomen crate"));
                }
                seen_crate = true;
                crate_path = Some(input.parse()?);
            } else if input.peek(syn::Ident) && input.peek2(Token![=]) {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: syn::LitStr = input.parse()?;
                if key == "prefix" {
                    if seen_prefix {
                        return Err(syn::Error::new(key.span(), "duplicate cognomen prefix"));
                    }
                    seen_prefix = true;
                    prefix = value.value();
                    if !is_ascii_ident(&prefix) {
                        return Err(syn::Error::new(
                            value.span(),
                            "prefix must be a non-empty ASCII identifier (e.g. prefix = \"label\")",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown cognomen key `{key}`"),
                    ));
                }
            } else {
                let (style, span) = parse_case_style(input)?;
                if styles.contains(&style) {
                    return Err(syn::Error::new(span, "duplicate cognomen case style"));
                }
                styles.push(style);
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if styles.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "missing cognomen case style (e.g. #[cognomen(snake_case)])",
            ));
        }

        let default = styles[0];
        Ok(Self {
            styles,
            default,
            prefix,
            crate_path: crate_path.unwrap_or_else(|| syn::parse_quote!(::cognomen)),
        })
    }
}

struct VariantAttr {
    rename: Option<syn::LitStr>,
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut rename = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: syn::LitStr = input.parse()?;
            if key == "rename" {
                if rename.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate cognomen rename"));
                }
                if value.value().is_empty() {
                    return Err(syn::Error::new(
                        value.span(),
                        "cognomen rename must not be empty",
                    ));
                }
                rename = Some(value);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    format!("unknown cognomen variant key `{key}`"),
                ));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { rename })
    }
}

struct Variant<'a> {
    ident: &'a Ident,
    rename: Option<String>,
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

fn join_lower(words: &[String], sep: char) -> String {
    let mut out = String::new();
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            out.push(sep);
        }
        out.push_str(&w.to_ascii_lowercase());
    }
    out
}

fn join_upper(words: &[String], sep: char) -> String {
    let mut out = String::new();
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            out.push(sep);
        }
        out.push_str(&w.to_ascii_uppercase());
    }
    out
}

fn parse_variant_rename(variant: &syn::Variant) -> Result<Option<String>> {
    let mut rename = None;
    for attr in &variant.attrs {
        if !attr.path().is_ident("cognomen") {
            continue;
        }
        if rename.is_some() {
            return Err(syn::Error::new(
                attr.span(),
                "duplicate #[cognomen(...)] on variant",
            ));
        }
        let parsed: VariantAttr = attr.parse_args()?;
        rename = parsed.rename.map(|l| l.value());
        if rename.is_none() {
            return Err(syn::Error::new(
                attr.span(),
                "variant #[cognomen(...)] requires rename = \"...\"",
            ));
        }
    }
    Ok(rename)
}

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let name = &input.ident;

    let mut attr = None;
    for attr_ in &input.attrs {
        if attr_.path().is_ident("cognomen") {
            if attr.is_some() {
                return Err(syn::Error::new(
                    attr_.span(),
                    "duplicate #[cognomen(...)] attribute",
                ));
            }
            attr = Some(attr_.parse_args::<CognomenAttr>()?);
        }
    }
    let attr = attr.ok_or_else(|| {
        syn::Error::new(
            name.span(),
            "missing #[cognomen(<case>)] container attribute (e.g. #[cognomen(snake_case)])",
        )
    })?;

    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            name.span(),
            "Cognomen can only be derived for enums",
        ));
    };

    let mut variants: Vec<Variant<'_>> = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Cognomen only supports unit variants (no fields)",
            ));
        }
        variants.push(Variant {
            ident: &variant.ident,
            rename: parse_variant_rename(variant)?,
        });
    }

    if variants.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            "Cognomen enum must have at least one variant",
        ));
    }

    let crate_path = &attr.crate_path;
    let idents: Vec<&Ident> = variants.iter().map(|v| v.ident).collect();
    let default_labels: Vec<String> = variants
        .iter()
        .map(|v| v.default_label(attr.default))
        .collect();

    let case_methods = attr.styles.iter().map(|style| {
        let method = format_ident!("{}", style.method_name(&attr.prefix));
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
                match self {
                    #(#arms,)*
                }
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

    let mut reverse_arms = Vec::new();
    let mut eq_arms = Vec::new();
    let mut label_owner: std::collections::BTreeMap<String, &Ident> =
        std::collections::BTreeMap::new();
    let mut all_labels: Vec<String> = Vec::new();
    for v in &variants {
        let labels = v.all_labels(&attr.styles);
        let lits: Vec<syn::LitStr> = labels
            .iter()
            .map(|label| syn::LitStr::new(label, v.ident.span()))
            .collect();
        for lit in &lits {
            if let Some(owner) = label_owner.insert(lit.value(), v.ident) {
                if owner != v.ident {
                    return Err(syn::Error::new(
                        v.ident.span(),
                        format!(
                            "generated label `{}` is shared by multiple variants",
                            lit.value()
                        ),
                    ));
                }
            }
        }
        all_labels.extend(labels);
        let ident = v.ident;
        let pat = quote! { #(#lits)|* };
        reverse_arms.push(quote! { #pat => ::core::result::Result::Ok(#name::#ident) });
        eq_arms.push(quote! { Self::#ident => matches!(other, #(#lits)|*) });
    }
    all_labels.sort();
    all_labels.dedup();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let has_generics = !input.generics.params.is_empty();

    let variants_const = if has_generics {
        quote! {}
    } else {
        quote! {
            /// All variants in declaration order.
            pub const VARIANTS: &'static [Self] = &[#(Self::#idents,)*];

            /// Default [`Self::label`] for each variant in declaration order.
            pub const LABELS: &'static [&'static str] = &[#(#default_labels,)*];
        }
    };

    let parse_impls = if cfg!(feature = "alloc") {
        quote! {
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
        }
    } else {
        quote! {}
    };

    let serde_impls = if cfg!(feature = "serde") {
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
    } else {
        quote! {}
    };

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #variants_const

            /// Stable label for this variant in the default (first) case.
            ///
            /// Overridden by `#[cognomen(rename = "...")]` on the variant.
            #[inline]
            #[must_use]
            pub const fn label(&self) -> &'static str {
                match self {
                    #(#label_arms,)*
                }
            }

            /// Alias of [`Self::label`].
            #[inline]
            #[must_use]
            pub const fn as_str(&self) -> &'static str {
                self.label()
            }

            #(#case_methods)*
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
                match self {
                    #(#eq_arms,)*
                }
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
