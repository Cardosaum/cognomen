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
    fn parse_ident(id: &Ident) -> Result<Self> {
        // Accept both snake_case spellings and punctuated forms via string.
        let s = id.to_string();
        Self::from_str_style(&s).ok_or_else(|| {
            syn::Error::new(
                id.span(),
                "unknown cognomen case style; expected snake_case|kebab_case|kebab-case|camelCase|PascalCase|SCREAMING_SNAKE_CASE|lower|upper",
            )
        })
    }

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

    /// Canonical attribute keyword for this case style (used in docs).
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

    /// Suffix for the per-case `label_<case>` accessor.
    fn method_name(self) -> &'static str {
        match self {
            Self::Snake => "label_snake",
            Self::Kebab => "label_kebab",
            Self::Camel => "label_camel",
            Self::Pascal => "label_pascal",
            Self::ScreamingSnake => "label_screaming_snake",
            Self::Lower => "label_lower",
            Self::Upper => "label_upper",
        }
    }

    fn convert(self, pascal_ident: &str) -> String {
        let words = split_pascal_words(pascal_ident);
        match self {
            Self::Snake => words
                .iter()
                .map(|w| w.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .join("_"),
            Self::Kebab => words
                .iter()
                .map(|w| w.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .join("-"),
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
            Self::ScreamingSnake => words
                .iter()
                .map(|w| w.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
            Self::Lower => words
                .iter()
                .map(|w| w.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .concat(),
            Self::Upper => words
                .iter()
                .map(|w| w.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .concat(),
        }
    }
}

/// `#[cognomen(snake_case)]` or `#[cognomen(snake_case, kebab-case)]`.
///
/// The first case listed is the default returned by `label()`/`as_str()`.
struct CognomenAttr {
    styles: Vec<CaseStyle>,
    default: CaseStyle,
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
        CaseStyle::parse_ident(&first)
    }
}

impl Parse for CognomenAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let first = parse_case_style(input)?;
        let mut styles = vec![first];
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break; // trailing comma
            }
            let style = parse_case_style(input)?;
            if styles.contains(&style) {
                return Err(syn::Error::new(
                    input.span(),
                    "duplicate cognomen case style",
                ));
            }
            styles.push(style);
        }
        let default = styles[0];
        Ok(Self { styles, default })
    }
}

fn split_pascal_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() && !cur.is_empty() {
            let prev_lower = chars
                .get(i.wrapping_sub(1))
                .is_some_and(|p| p.is_lowercase());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if prev_lower || next_lower {
                words.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c);
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

    // A `label_<case>` accessor for every declared case.
    let case_methods = attr
        .styles
        .iter()
        .map(|style| {
            let method = format_ident!("{}", style.method_name());
            let doc = format!(
                "Stable label for this variant in the `{}` case.",
                style.name()
            );
            let arms = variants
                .iter()
                .map(|v| {
                    let label = style.convert(&v.to_string());
                    quote! { Self::#v => #label }
                })
                .collect::<Vec<_>>();
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
        })
        .collect::<Vec<_>>();

    // `label()` / `as_str()` are aliases for the default (first) case.
    let default_method = format_ident!("{}", attr.default.method_name());

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Stable label for this variant in the default (first) case.
            #[inline]
            #[must_use]
            pub const fn label(self) -> &'static str {
                self.#default_method()
            }

            /// Alias of [`Self::label`] (ergonomic for config / logs).
            #[inline]
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                self.label()
            }

            #(#case_methods)*
        }
    })
}
