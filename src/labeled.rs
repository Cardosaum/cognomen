//! `#[derive(Labeled)]` implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Result, Token};

#[derive(Clone, Copy)]
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
                "unknown labeled case style; expected snake_case|kebab_case|kebab-case|camelCase|PascalCase|SCREAMING_SNAKE_CASE|lower|upper",
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

/// `#[labeled(snake_case)]` or `#[labeled(case = snake_case)]`
struct LabeledAttr {
    style: CaseStyle,
}

impl Parse for LabeledAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            let key: Ident = input.parse()?;
            if key != "case" {
                return Err(syn::Error::new(key.span(), "expected `case = <style>`"));
            }
            input.parse::<Token![=]>()?;
            let style_id: Ident = input.parse()?;
            return Ok(Self {
                style: CaseStyle::parse_ident(&style_id)?,
            });
        }
        // Style may be multi-token: kebab-case uses Ident - Ident
        let first: Ident = input.parse()?;
        if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            let second: Ident = input.parse()?;
            let joined = format!("{first}-{second}");
            let style = CaseStyle::from_str_style(&joined).ok_or_else(|| {
                syn::Error::new(first.span(), format!("unknown labeled style `{joined}`"))
            })?;
            return Ok(Self { style });
        }
        Ok(Self {
            style: CaseStyle::parse_ident(&first)?,
        })
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

    let mut style = None;
    for attr in &input.attrs {
        if attr.path().is_ident("labeled") {
            let parsed: LabeledAttr = attr.parse_args()?;
            if style.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate #[labeled(...)] attribute",
                ));
            }
            style = Some(parsed.style);
        }
    }
    let style = style.ok_or_else(|| {
        syn::Error::new(
            name.span(),
            "missing #[labeled(<case>)] container attribute (e.g. #[labeled(snake_case)])",
        )
    })?;

    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            name.span(),
            "Labeled can only be derived for enums",
        ));
    };

    let mut arms = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Labeled only supports unit variants (no fields)",
            ));
        }
        let vname = &variant.ident;
        let label = style.convert(&vname.to_string());
        arms.push(quote! {
            Self::#vname => #label
        });
    }

    if arms.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            "Labeled enum must have at least one variant",
        ));
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Stable label for this variant under the derive case style.
            #[inline]
            #[must_use]
            pub const fn label(self) -> &'static str {
                match self {
                    #(#arms,)*
                }
            }

            /// Alias of [`Self::label`] (ergonomic for config / logs).
            #[inline]
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                self.label()
            }
        }
    })
}
