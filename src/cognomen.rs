//! `#[derive(Cognomen)]` implementation.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Result, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Lower,
    Upper,
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
            _ => None,
        }
    }

    /// Canonical name used in generated docs.
    fn name(self) -> &'static str {
        match self {
            Self::Snake => "snake_case",
            Self::Kebab => "kebab-case",
            Self::Camel => "camelCase",
            Self::Pascal => "PascalCase",
            Self::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Self::Lower => "lower",
            Self::Upper => "upper",
        }
    }

    /// Accessor suffix: `label_{suffix}` / `{prefix}_{suffix}`.
    fn suffix(self) -> &'static str {
        match self {
            Self::Snake => "snake",
            Self::Kebab => "kebab",
            Self::Camel => "camel",
            Self::Pascal => "pascal",
            Self::ScreamingSnake => "screaming_snake",
            Self::Lower => "lower",
            Self::Upper => "upper",
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
}

fn parse_case_style(input: ParseStream<'_>) -> Result<CaseStyle> {
    // A style may be multi-token: kebab-case is `Ident - Ident`.
    let first: Ident = input.parse()?;
    if input.peek(Token![-]) {
        input.parse::<Token![-]>()?;
        let second: Ident = input.parse()?;
        let joined = format!("{first}-{second}");
        CaseStyle::from_str_style(&joined).ok_or_else(|| {
            syn::Error::new(first.span(), format!("unknown cognomen style `{joined}`"))
        })
    } else {
        let s = first.to_string();
        CaseStyle::from_str_style(&s).ok_or_else(|| {
            syn::Error::new(
                first.span(),
                "unknown cognomen case style; expected snake_case|snake|kebab_case|kebab-case|kebab|camelCase|camel|PascalCase|pascal|SCREAMING_SNAKE_CASE|screaming|lower|upper",
            )
        })
    }
}

impl Parse for CognomenAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut styles = Vec::new();
        let mut prefix = String::from("label");

        while !input.is_empty() {
            if input.peek(syn::Ident) && input.peek2(Token![=]) {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: syn::LitStr = input.parse()?;
                if key == "prefix" {
                    prefix = value.value();
                } else {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown cognomen key `{key}`"),
                    ));
                }
            } else {
                let style = parse_case_style(input)?;
                if styles.contains(&style) {
                    return Err(syn::Error::new(
                        input.span(),
                        "duplicate cognomen case style",
                    ));
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
        })
    }
}

fn split_pascal_words(s: &str) -> Vec<String> {
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

    let mut variants: Vec<&Ident> = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Cognomen only supports unit variants (no fields)",
            ));
        }
        variants.push(&variant.ident);
    }

    if variants.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            "Cognomen enum must have at least one variant",
        ));
    }

    let case_methods = attr.styles.iter().map(|style| {
        let method = format_ident!("{}", style.method_name(&attr.prefix));
        let doc = format!(
            "Stable label for this variant in the `{}` case.",
            style.name()
        );
        let arms = variants.iter().map(|v| {
            let label = style.convert(&v.to_string());
            quote! { Self::#v => #label }
        });
        quote! {
            #[doc = #doc]
            #[inline]
            #[must_use]
            pub const fn #method(self) -> &'static str {
                match self {
                    #(#arms,)*
                }
            }
        }
    });

    // Reverse path: any declared-case label → variant. Dedup + collision check.
    let mut reverse_arms = Vec::new();
    let mut label_owner: std::collections::BTreeMap<String, &Ident> =
        std::collections::BTreeMap::new();
    for &v in &variants {
        let mut labels: Vec<String> = attr
            .styles
            .iter()
            .map(|style| style.convert(&v.to_string()))
            .collect();
        labels.sort();
        labels.dedup();
        let lits: Vec<syn::LitStr> = labels
            .iter()
            .map(|label| syn::LitStr::new(label, v.span()))
            .collect();
        for lit in &lits {
            if let Some(owner) = label_owner.insert(lit.value(), v) {
                if owner != v {
                    return Err(syn::Error::new(
                        v.span(),
                        format!(
                            "generated label `{}` is shared by multiple variants",
                            lit.value()
                        ),
                    ));
                }
            }
        }
        let pat = quote! { #(#lits)|* };
        reverse_arms.push(quote! { #pat => Ok(Self::#v) });
    }

    // Proc-macro crates cannot export ordinary items; hide the error type per enum.
    let helper = format_ident!("cognomen_{}", name);
    let helper_mod = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #helper {
            #[derive(Clone, PartialEq, Eq, Debug)]
            pub struct FromLabelError {
                pub input: ::std::string::String,
            }

            impl ::core::fmt::Display for FromLabelError {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "no cognomen label matches `{}`", self.input)
                }
            }

            impl ::std::error::Error for FromLabelError {}
        }
    };

    let default_method = format_ident!("{}", attr.default.method_name(&attr.prefix));
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #helper_mod

        impl #impl_generics #name #ty_generics #where_clause {
            /// Stable label for this variant in the default (first) case.
            #[inline]
            #[must_use]
            pub const fn label(self) -> &'static str {
                self.#default_method()
            }

            #(#case_methods)*
        }

        impl #impl_generics ::core::convert::TryFrom<&str> for #name #ty_generics #where_clause {
            type Error = #helper::FromLabelError;
            #[inline]
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s {
                    #(#reverse_arms,)*
                    _ => Err(#helper::FromLabelError { input: s.to_owned() }),
                }
            }
        }

        impl #impl_generics ::core::str::FromStr for #name #ty_generics #where_clause {
            type Err = #helper::FromLabelError;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                ::core::convert::TryFrom::try_from(s)
            }
        }
    })
}
