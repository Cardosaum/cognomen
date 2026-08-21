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

    #[cfg_attr(not(test), allow(dead_code))]
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
    /// Accepted for compatibility; case accessors live on `Label`.
    #[allow(dead_code)]
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

fn insert_extra(
    extras: &mut Vec<(String, ExtraDecl)>,
    key: Ident,
    default: String,
    value_span: Span,
) -> Result<()> {
    let name = key.to_string();
    if name == "prefix" || name == "rename" || name == "alias" || name == "unknown" {
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
    if template_has_fields(&default, value_span)? {
        return Err(syn::Error::new(
            value_span,
            "placeholders are only valid on a variant extra",
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

const ALL_STYLES: [CaseStyle; 8] = [
    CaseStyle::Snake,
    CaseStyle::Kebab,
    CaseStyle::Camel,
    CaseStyle::Pascal,
    CaseStyle::ScreamingSnake,
    CaseStyle::Lower,
    CaseStyle::Upper,
    CaseStyle::Title,
];

impl CaseStyle {
    fn case_path(self, crate_path: &syn::Path) -> TokenStream {
        match self {
            Self::Snake => quote! { #crate_path::Case::Snake },
            Self::Kebab => quote! { #crate_path::Case::Kebab },
            Self::Camel => quote! { #crate_path::Case::Camel },
            Self::Pascal => quote! { #crate_path::Case::Pascal },
            Self::ScreamingSnake => quote! { #crate_path::Case::ScreamingSnake },
            Self::Lower => quote! { #crate_path::Case::Lower },
            Self::Upper => quote! { #crate_path::Case::Upper },
            Self::Title => quote! { #crate_path::Case::Title },
        }
    }
}

fn reserved_idents() -> [&'static str; 15] {
    [
        "label",
        "as_str",
        "in_case",
        "from_label",
        "extra",
        "VARIANTS",
        "LABELS",
        "eq",
        "ne",
        "fmt",
        "as_ref",
        "try_from",
        "from_str",
        "serialize",
        "deserialize",
    ]
}

fn check_extras(extras: &[(String, ExtraDecl)]) -> Result<()> {
    let reserved = reserved_idents();
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
                let (default, value_span) = if content.is_empty() {
                    (String::new(), key.span())
                } else {
                    let value: syn::LitStr = content.parse()?;
                    if !content.is_empty() {
                        return Err(content.error("expected a single string literal"));
                    }
                    (value.value(), value.span())
                };
                insert_extra(&mut extras, key, default, value_span)?;
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
                } else if key == "alias" {
                    return Err(syn::Error::new(
                        key.span(),
                        "alias is only valid on a variant (e.g. #[cognomen(alias = \"main\")])",
                    ));
                } else if key == "unknown" {
                    return Err(syn::Error::new(
                        key.span(),
                        "unknown is only valid on a variant (e.g. #[cognomen(unknown)])",
                    ));
                } else {
                    insert_extra(&mut extras, key, value.value(), value.span())?;
                }
            } else if input.peek(Ident) {
                let ahead = input.fork();
                let ident: Ident = ahead.parse()?;
                if ident == "unknown" && !ahead.peek(Token![-]) {
                    let key: Ident = input.parse()?;
                    return Err(syn::Error::new(
                        key.span(),
                        "unknown is only valid on a variant (e.g. #[cognomen(unknown)])",
                    ));
                }
                let (style, span) = parse_case_style(input)?;
                if styles.contains(&style) {
                    return Err(syn::Error::new(span, "duplicate cognomen case style"));
                }
                styles.push(style);
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
    aliases: Vec<syn::LitStr>,
    unknown: Option<Span>,
    extras: Vec<(String, syn::LitStr)>,
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut rename = None;
        let mut aliases = Vec::new();
        let mut unknown = None;
        let mut extras = Vec::new();
        while !input.is_empty() {
            if input.peek(Ident) && !input.peek2(Token![=]) && !input.peek2(syn::token::Paren) {
                let key: Ident = input.parse()?;
                if key == "unknown" {
                    set_once(
                        &mut unknown,
                        key.span(),
                        key.span(),
                        "duplicate cognomen unknown",
                    )?;
                } else {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected `key = \"...\"` or `key(\"...\")`",
                    ));
                }
            } else {
                let (key, value) = parse_key_litstr(input)?;
                if key == "rename" {
                    if value.value().is_empty() {
                        return Err(syn::Error::new(
                            value.span(),
                            "cognomen rename must not be empty",
                        ));
                    }
                    set_once(&mut rename, value, key.span(), "duplicate cognomen rename")?;
                } else if key == "alias" {
                    if value.value().is_empty() {
                        return Err(syn::Error::new(
                            value.span(),
                            "cognomen alias must not be empty",
                        ));
                    }
                    aliases.push(value);
                } else if key == "unknown" {
                    return Err(syn::Error::new(
                        key.span(),
                        "unknown does not take a value (use #[cognomen(unknown)])",
                    ));
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
            }
            eat_comma(input)?;
        }
        Ok(Self {
            rename,
            aliases,
            unknown,
            extras,
        })
    }
}

struct Variant<'a> {
    ident: &'a Ident,
    rename: Option<String>,
    aliases: Vec<String>,
    unknown: Option<Span>,
    extras: BTreeMap<String, syn::LitStr>,
    fields: FieldsKind,
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

    /// Declared labels plus parse-only aliases. Used by `FromStr` / serde-in.
    fn parse_labels(&self, styles: &[CaseStyle]) -> Vec<String> {
        let mut labels = self.all_labels(styles);
        labels.extend(self.aliases.iter().cloned());
        labels.sort();
        labels.dedup();
        labels
    }

    fn extra_text(&self, name: &str, decl: &ExtraDecl, default: CaseStyle) -> String {
        self.extras
            .get(name)
            .map(syn::LitStr::value)
            .or_else(|| decl.default.clone())
            .unwrap_or_else(|| self.default_label(default))
    }

    fn ignore_pat(&self) -> TokenStream {
        let ident = self.ident;
        match &self.fields {
            FieldsKind::Unit => quote! { Self::#ident },
            FieldsKind::Named(_) => quote! { Self::#ident { .. } },
            FieldsKind::Unnamed(_) => quote! { Self::#ident(..) },
        }
    }

    fn bind_pat(&self, needed: &[String]) -> TokenStream {
        let ident = self.ident;
        match &self.fields {
            FieldsKind::Unit => quote! { Self::#ident },
            FieldsKind::Named(_) => {
                let binds = needed.iter().map(|n| format_ident!("{n}"));
                quote! { Self::#ident { #(#binds,)* .. } }
            }
            FieldsKind::Unnamed(n) => {
                let max_needed = needed
                    .iter()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .max()
                    .map(|m| m + 1)
                    .unwrap_or(0);
                let mut elems = Vec::new();
                for i in 0..*n {
                    let name = i.to_string();
                    if needed.iter().any(|f| f == &name) {
                        let id = format_ident!("__f{i}");
                        elems.push(quote! { #id });
                    } else if i < max_needed {
                        elems.push(quote! { _ });
                    } else {
                        break;
                    }
                }
                if elems.len() < *n {
                    quote! { Self::#ident(#(#elems,)* ..) }
                } else {
                    quote! { Self::#ident(#(#elems),*) }
                }
            }
        }
    }

    fn formatted_expr(
        &self,
        text: &str,
        span: Span,
        crate_path: &syn::Path,
    ) -> Result<TokenStream> {
        let segments = parse_template(text, span)?;
        if segments.len() > 16 {
            return Err(syn::Error::new(
                span,
                "cognomen extra has too many `{field}` / literal fragments (max 16)",
            ));
        }
        let mut chain = quote! { #crate_path::Formatted::empty() };
        for seg in &segments {
            match seg {
                Segment::Lit(t) if t.is_empty() => {}
                Segment::Lit(t) => {
                    chain = quote! { #chain.lit(#t) };
                }
                Segment::Field(name) => {
                    let ident = self.fields.bind_ident(name);
                    chain = quote! { #chain.arg(#ident) };
                }
            }
        }
        Ok(chain)
    }

    fn validate_extra(&self, name: &str, lit: &syn::LitStr) -> Result<()> {
        let segments = parse_template(&lit.value(), lit.span())?;
        let known = self.fields.names();
        for seg in &segments {
            let Segment::Field(field) = seg else {
                continue;
            };
            if !known.iter().any(|n| n == field) {
                let msg = if known.is_empty() {
                    format!("placeholder `{{{field}}}` is invalid on a unit variant")
                } else {
                    format!("unknown field `{{{field}}}` in cognomen extra `{name}`")
                };
                return Err(syn::Error::new(lit.span(), msg));
            }
        }
        Ok(())
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    Lit(String),
    Field(String),
}

impl Segment {
    fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }
}

fn is_field_ref(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    is_ascii_ident(s)
}

fn parse_template(s: &str, span: Span) -> Result<Vec<Segment>> {
    let mut out = Vec::new();
    let mut lit = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                lit.push('{');
                continue;
            }
            if !lit.is_empty() {
                out.push(Segment::Lit(std::mem::take(&mut lit)));
            }
            let mut name = String::new();
            let mut closed = false;
            for c in chars.by_ref() {
                if c == '}' {
                    closed = true;
                    break;
                }
                if c == '{' {
                    return Err(syn::Error::new(
                        span,
                        "nested `{` in cognomen extra; escape a literal brace as `{{`",
                    ));
                }
                name.push(c);
            }
            if !closed {
                return Err(syn::Error::new(span, "unclosed `{` in cognomen extra"));
            }
            if name.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "empty `{}` placeholder; use a field name (e.g. `{cause}`)",
                ));
            }
            if !is_field_ref(&name) {
                return Err(syn::Error::new(
                    span,
                    format!("placeholder `{{{name}}}` is not a field name"),
                ));
            }
            out.push(Segment::Field(name));
        } else if c == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                lit.push('}');
            } else {
                return Err(syn::Error::new(
                    span,
                    "unmatched `}` in cognomen extra; escape as `}}`",
                ));
            }
        } else {
            lit.push(c);
        }
    }
    if !lit.is_empty() {
        out.push(Segment::Lit(lit));
    }
    Ok(out)
}

fn template_has_fields(s: &str, span: Span) -> Result<bool> {
    Ok(parse_template(s, span)?.iter().any(Segment::is_field))
}

enum FieldsKind {
    Unit,
    Named(Vec<Ident>),
    Unnamed(usize),
}

impl FieldsKind {
    fn from_fields(fields: &Fields) -> Self {
        match fields {
            Fields::Unit => Self::Unit,
            Fields::Named(named) => {
                Self::Named(named.named.iter().filter_map(|f| f.ident.clone()).collect())
            }
            Fields::Unnamed(unnamed) => Self::Unnamed(unnamed.unnamed.len()),
        }
    }

    fn has_payload(&self) -> bool {
        match self {
            Self::Unit => false,
            Self::Named(n) => !n.is_empty(),
            Self::Unnamed(n) => *n > 0,
        }
    }

    fn names(&self) -> Vec<String> {
        match self {
            Self::Unit => Vec::new(),
            Self::Named(n) => n.iter().map(ToString::to_string).collect(),
            Self::Unnamed(n) => (0..*n).map(|i| i.to_string()).collect(),
        }
    }

    fn bind_ident(&self, field: &str) -> Ident {
        match self {
            Self::Unnamed(_) => format_ident!("__f{field}"),
            _ => format_ident!("{field}"),
        }
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
) -> Result<(
    Option<String>,
    Vec<String>,
    Option<Span>,
    BTreeMap<String, syn::LitStr>,
)> {
    let mut extras = BTreeMap::new();
    let mut rename = None;
    let mut aliases = Vec::new();
    let mut unknown = None;

    if let Some(attr) = find_cognomen_attr(&variant.attrs, "duplicate #[cognomen(...)] on variant")?
    {
        let parsed: VariantAttr = attr.parse_args()?;
        if parsed.rename.is_none()
            && parsed.aliases.is_empty()
            && parsed.unknown.is_none()
            && parsed.extras.is_empty()
        {
            return Err(syn::Error::new(
                attr.span(),
                "variant #[cognomen(...)] requires rename = \"...\", alias = \"...\", unknown, or an extra method",
            ));
        }
        rename = parsed.rename.map(|l| l.value());
        aliases = parsed.aliases.iter().map(syn::LitStr::value).collect();
        unknown = parsed.unknown;
        for (name, lit) in parsed.extras {
            if extras.contains_key(&name) {
                return Err(syn::Error::new(
                    lit.span(),
                    format!("duplicate extra `{name}` on variant"),
                ));
            }
            ensure_extra(declared, name.clone(), lit.span());
            extras.insert(name, lit);
        }
    }

    Ok((rename, aliases, unknown, extras))
}

fn enum_variants<'a>(
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
        let (rename, aliases, unknown, extras) = parse_variant_meta(variant, declared)?;
        let parsed = Variant {
            ident: &variant.ident,
            rename,
            aliases,
            unknown,
            extras,
            fields: FieldsKind::from_fields(&variant.fields),
        };
        for (name, lit) in &parsed.extras {
            parsed.validate_extra(name, lit)?;
        }
        variants.push(parsed);
    }
    if variants.is_empty() {
        return Err(syn::Error::new(
            input.ident.span(),
            "Cognomen enum must have at least one variant",
        ));
    }
    let marked_unknown: Vec<&Variant<'_>> =
        variants.iter().filter(|v| v.unknown.is_some()).collect();
    if marked_unknown.len() > 1 {
        return Err(syn::Error::new(
            marked_unknown[1].unknown.unwrap(),
            "only one variant may be marked #[cognomen(unknown)]",
        ));
    }
    if let Some(v) = marked_unknown.first() {
        if v.fields.has_payload() {
            return Err(syn::Error::new(
                v.unknown.unwrap(),
                "#[cognomen(unknown)] requires a unit variant",
            ));
        }
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
        let eq_labels = v.all_labels(styles);
        let parse_labels = v.parse_labels(styles);
        let parse_lits: Vec<syn::LitStr> = parse_labels
            .iter()
            .map(|label| syn::LitStr::new(label, v.ident.span()))
            .collect();
        let eq_lits: Vec<syn::LitStr> = eq_labels
            .iter()
            .map(|label| syn::LitStr::new(label, v.ident.span()))
            .collect();
        for label in &parse_labels {
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
        let pat = v.ignore_pat();
        reverse_arms.push(quote! { #(#parse_lits)|* => ::core::result::Result::Ok(#name::#ident) });
        eq_arms.push(quote! { #pat => matches!(other, #(#eq_lits)|*) });
    }

    Ok((reverse_arms, eq_arms, owner.into_keys().collect()))
}

fn extra_key_ty(name: &str, crate_path: &syn::Path) -> Option<TokenStream> {
    let key = match name {
        "reason" => quote! { ReasonKey },
        "blurb" => quote! { BlurbKey },
        "hint" => quote! { HintKey },
        "help" => quote! { HelpKey },
        _ => return None,
    };
    Some(quote! { #crate_path::#key })
}

fn extra_arms(
    extra_name: &str,
    decl: &ExtraDecl,
    variants: &[Variant<'_>],
    default: CaseStyle,
    crate_path: &syn::Path,
) -> Result<Vec<TokenStream>> {
    variants
        .iter()
        .map(|v| {
            let text = v.extra_text(extra_name, decl, default);
            let span = v
                .extras
                .get(extra_name)
                .map(syn::LitStr::span)
                .unwrap_or_else(|| v.ident.span());
            let segments = parse_template(&text, span)?;
            let needed: Vec<String> = segments
                .iter()
                .filter_map(|seg| match seg {
                    Segment::Field(f) => Some(f.clone()),
                    Segment::Lit(_) => None,
                })
                .collect();
            let pat = v.bind_pat(&needed);
            let args = v.formatted_expr(&text, span, crate_path)?;
            Ok(quote! { #pat => #args })
        })
        .collect()
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
    let variants = enum_variants(&input, &mut extras)?;
    check_extras(&extras)?;
    let crate_path = &attr.crate_path;
    let default = attr.styles[0];
    let fieldless = variants.iter().all(|v| !v.fields.has_payload());
    if let Some(v) = variants.iter().find(|v| v.unknown.is_some()) {
        if !fieldless {
            return Err(syn::Error::new(
                v.unknown.unwrap(),
                "#[cognomen(unknown)] is only valid on a fieldless enum",
            ));
        }
    }
    let unknown_ident = variants.iter().find_map(|v| v.unknown.map(|_| v.ident));
    let default_labels: Vec<String> = variants.iter().map(|v| v.default_label(default)).collect();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let extra_impls = extras
        .iter()
        .map(|(extra_name, decl)| {
            let arms = extra_arms(extra_name, decl, &variants, default, crate_path)?;
            let body = quote! {
                #[inline]
                fn extra(&self) -> #crate_path::Formatted<'_> {
                    match self { #(#arms,)* }
                }
            };
            Ok(if let Some(key) = extra_key_ty(extra_name, crate_path) {
                quote! {
                    impl #impl_generics #crate_path::Extra<#key> for #name #ty_generics #where_clause {
                        #body
                    }
                }
            } else {
                quote! {
                    const _: () = {
                        enum __CognomenExtraKey {}
                        impl #impl_generics #crate_path::Extra<__CognomenExtraKey> for #name #ty_generics #where_clause {
                            #body
                        }
                    };
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let label_arms = variants
        .iter()
        .zip(default_labels.iter())
        .map(|(v, label)| {
            let pat = v.ignore_pat();
            quote! { #pat => #label }
        });

    let in_case_arms = variants.iter().map(|v| {
        let pat = v.ignore_pat();
        let case_arms = ALL_STYLES.iter().map(|style| {
            let case = style.case_path(crate_path);
            let label = v.case_label(*style);
            quote! { #case => #label }
        });
        quote! {
            #pat => match case {
                #(#case_arms,)*
            }
        }
    });

    let (reverse_arms, eq_arms, all_labels) = reverse_eq_arms(name, &variants, &attr.styles)?;

    let variants_impl = (fieldless && input.generics.params.is_empty()).then(|| {
        let idents: Vec<&Ident> = variants.iter().map(|v| v.ident).collect();
        quote! {
            impl #crate_path::Variants for #name {
                const VARIANTS: &'static [Self] = &[#(Self::#idents,)*];
                const LABELS: &'static [&'static str] = &[#(#default_labels,)*];
            }
        }
    });

    let parse_catchall = match unknown_ident {
        Some(ident) => quote! { _ => ::core::result::Result::Ok(#name::#ident) },
        None => {
            quote! { _ => ::core::result::Result::Err(#crate_path::FromLabelError::new(s)) }
        }
    };

    let parse_impls = fieldless.then(|| quote! {
        impl #impl_generics #crate_path::FromLabel for #name #ty_generics #where_clause {
            #[inline]
            fn from_label(s: &str) -> ::core::result::Result<Self, #crate_path::FromLabelError> {
                ::core::convert::TryFrom::try_from(s)
            }
        }

        impl #impl_generics ::core::convert::TryFrom<&str> for #name #ty_generics #where_clause {
            type Error = #crate_path::FromLabelError;
            #[inline]
            fn try_from(s: &str) -> ::core::result::Result<Self, #crate_path::FromLabelError> {
                match s {
                    #(#reverse_arms,)*
                    #parse_catchall
                }
            }
        }

        impl #impl_generics ::core::str::FromStr for #name #ty_generics #where_clause {
            type Err = #crate_path::FromLabelError;
            #[inline]
            fn from_str(s: &str) -> ::core::result::Result<Self, #crate_path::FromLabelError> {
                ::core::convert::TryFrom::try_from(s)
            }
        }
    });

    let de_impl_generics = {
        let params = &input.generics.params;
        if params.is_empty() {
            quote! { <'de> }
        } else {
            quote! { <'de, #params> }
        }
    };

    let serde_catchall = match unknown_ident {
        Some(ident) => quote! { _ => ::core::result::Result::Ok(#name::#ident) },
        None => quote! {
            _ => ::core::result::Result::Err(
                E::unknown_variant(v, &[#(#all_labels,)*]),
            )
        },
    };

    let serde_impls = (fieldless && cfg!(feature = "serde")).then(|| {
        quote! {
            impl #impl_generics #crate_path::__serde::Serialize for #name #ty_generics #where_clause {
                fn serialize<S: #crate_path::__serde::Serializer>(
                    &self,
                    serializer: S,
                ) -> ::core::result::Result<S::Ok, S::Error> {
                    #crate_path::__serde::Serialize::serialize(
                        #crate_path::Label::label(self),
                        serializer,
                    )
                }
            }

            impl #de_impl_generics #crate_path::__serde::Deserialize<'de> for #name #ty_generics #where_clause {
                fn deserialize<D: #crate_path::__serde::Deserializer<'de>>(
                    deserializer: D,
                ) -> ::core::result::Result<Self, D::Error> {
                    struct __CognomenVisitor #impl_generics (
                        ::core::marker::PhantomData<#name #ty_generics>
                    ) #where_clause;
                    impl #de_impl_generics #crate_path::__serde::de::Visitor<'de> for __CognomenVisitor #ty_generics #where_clause {
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
                                #serde_catchall
                            }
                        }
                        fn visit_borrowed_str<E: #crate_path::__serde::de::Error>(
                            self,
                            v: &'de str,
                        ) -> ::core::result::Result<Self::Value, E> {
                            self.visit_str(v)
                        }
                    }
                    deserializer.deserialize_str(__CognomenVisitor(::core::marker::PhantomData))
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics #crate_path::Label for #name #ty_generics #where_clause {
            #[inline]
            fn label(&self) -> &'static str {
                match self { #(#label_arms,)* }
            }

            #[inline]
            fn in_case(&self, case: #crate_path::Case) -> &'static str {
                match self { #(#in_case_arms,)* }
            }
        }

        #(#extra_impls)*

        #variants_impl

        impl #impl_generics ::core::convert::AsRef<str> for #name #ty_generics #where_clause {
            #[inline]
            fn as_ref(&self) -> &str {
                #crate_path::Label::label(self)
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
    use quote::{format_ident, quote};

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
    fn parse_template_parts() {
        let span = Span::call_site();
        assert_eq!(
            parse_template("host open failed {cause}", span).unwrap(),
            [
                Segment::Lit(String::from("host open failed ")),
                Segment::Field(String::from("cause")),
            ]
        );
        assert_eq!(
            parse_template("use {{braces}} {name}", span).unwrap(),
            [
                Segment::Lit(String::from("use {braces} ")),
                Segment::Field(String::from("name")),
            ]
        );
        assert_eq!(
            parse_template("open failed {0}", span).unwrap(),
            [
                Segment::Lit(String::from("open failed ")),
                Segment::Field(String::from("0")),
            ]
        );
        assert!(parse_template("x {}", span).is_err());
        assert!(parse_template("x {", span).is_err());
        assert!(parse_template("x }", span).is_err());
        assert_eq!(
            parse_template("a {b} c", span).unwrap(),
            [
                Segment::Lit(String::from("a ")),
                Segment::Field(String::from("b")),
                Segment::Lit(String::from(" c")),
            ]
        );
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
        assert!(!is_ascii_ident("caf\u{e9}"));
    }

    #[test]
    fn style_aliases() {
        let cases = [
            ("snake_case", "snake_case"),
            ("snake", "snake_case"),
            ("kebab_case", "kebab-case"),
            ("kebab-case", "kebab-case"),
            ("kebab", "kebab-case"),
            ("camelCase", "camelCase"),
            ("camel_case", "camelCase"),
            ("camel", "camelCase"),
            ("PascalCase", "PascalCase"),
            ("pascal_case", "PascalCase"),
            ("pascal", "PascalCase"),
            ("SCREAMING_SNAKE_CASE", "SCREAMING_SNAKE_CASE"),
            ("screaming_snake_case", "SCREAMING_SNAKE_CASE"),
            ("screaming", "SCREAMING_SNAKE_CASE"),
            ("lower", "lower"),
            ("lowercase", "lower"),
            ("upper", "upper"),
            ("uppercase", "upper"),
            ("title", "title"),
            ("title_case", "title"),
            ("TitleCase", "title"),
        ];
        for (alias, name) in cases {
            assert_eq!(
                CaseStyle::from_str_style(alias).unwrap().name(),
                name,
                "{alias}"
            );
        }
        assert!(CaseStyle::from_str_style("not_a_case").is_none());
        assert!(CaseStyle::from_str_style("kebab-foo").is_none());
    }

    #[test]
    fn split_more_idents() {
        assert_eq!(
            split_pascal_words("XMLHttpRequest"),
            ["XML", "Http", "Request"]
        );
        assert_eq!(split_pascal_words("A1B2"), ["A1B2"]);
        assert!(split_pascal_words("").is_empty());
    }

    fn err_msg(input: TokenStream) -> String {
        derive(input)
            .expect_err("expected derive to fail")
            .to_string()
    }

    fn assert_err(input: TokenStream, needle: &str) {
        let msg = err_msg(input);
        assert!(
            msg.contains(needle),
            "expected {needle:?} in error, got {msg:?}"
        );
    }

    fn ok(input: TokenStream) -> String {
        derive(input)
            .expect("expected derive to succeed")
            .to_string()
    }

    #[test]
    fn rejects_every_container_error() {
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                struct NotAnEnum { x: u8 }
            },
            "Cognomen can only be derived for enums",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                union NotAnEnum { x: u8 }
            },
            "Cognomen can only be derived for enums",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {}
            },
            "Cognomen enum must have at least one variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, reason = "host open failed {cause}")]
                enum Mode { A }
            },
            "placeholders are only valid on a variant extra",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(reason = "host open failed {cause}")]
                    A,
                }
            },
            "placeholder `{cause}` is invalid on a unit variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(reason = "host open failed {nope}")]
                    OpenFailed { cause: String },
                }
            },
            "unknown field `{nope}` in cognomen extra `reason`",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(reason = "host open failed {}")]
                    OpenFailed { cause: String },
                }
            },
            "empty `{}` placeholder",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(reason = "host open failed {cause")]
                    OpenFailed { cause: String },
                }
            },
            "unclosed `{` in cognomen extra",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(reason = "host open failed cause}")]
                    OpenFailed { cause: String },
                }
            },
            "unmatched `}` in cognomen extra",
        );
        assert_err(
            quote! { enum Mode { A } },
            "missing #[cognomen(<case>)] container attribute",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                #[cognomen(lower)]
                enum Mode { A }
            },
            "duplicate #[cognomen(...)] attribute",
        );
        assert_err(
            quote! {
                #[cognomen()]
                enum Mode { A }
            },
            "missing cognomen case style",
        );
        assert_err(
            quote! {
                #[cognomen(not_a_case)]
                enum Mode { A }
            },
            "unknown cognomen case style",
        );
        assert_err(
            quote! {
                #[cognomen(kebab-foo)]
                enum Mode { A }
            },
            "unknown cognomen style `kebab-foo`",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, rename = "x")]
                enum Mode { A }
            },
            "rename is only valid on a variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, alias = "main")]
                enum Mode { A }
            },
            "alias is only valid on a variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, unknown = "x")]
                enum Mode { A }
            },
            "unknown is only valid on a variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, unknown)]
                enum Mode { A }
            },
            "unknown is only valid on a variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, snake)]
                enum Mode { A }
            },
            "duplicate cognomen case style",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, prefix = "a", prefix = "b")]
                enum Mode { A }
            },
            "duplicate cognomen prefix",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, crate = ::a, crate = ::b)]
                enum Mode { A }
            },
            "duplicate cognomen crate",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, prefix = "my-label")]
                enum Mode { A }
            },
            "prefix must be a non-empty ASCII identifier",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, prefix = "")]
                enum Mode { A }
            },
            "prefix must be a non-empty ASCII identifier",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, prefix = "1abc")]
                enum Mode { A }
            },
            "prefix must be a non-empty ASCII identifier",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, prefix())]
                enum Mode { A }
            },
            "`prefix` is not an extra method",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, rename())]
                enum Mode { A }
            },
            "`rename` is not an extra method",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, alias())]
                enum Mode { A }
            },
            "`alias` is not an extra method",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, unknown())]
                enum Mode { A }
            },
            "`unknown` is not an extra method",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, blurb = "", blurb = "")]
                enum Mode { A }
            },
            "duplicate cognomen extra `blurb`",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, blurb("a", "b"))]
                enum Mode { A }
            },
            "expected a single string literal",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case, blurb(1))]
                enum Mode { A }
            },
            "expected string literal",
        );
        let extra = Ident::new("caf\u{e9}", Span::call_site());
        assert_err(
            quote! {
                #[cognomen(snake_case, #extra = "x")]
                enum Mode { A }
            },
            "extra method name must be a non-empty ASCII identifier",
        );
    }

    #[test]
    fn rejects_reserved_extras() {
        for name in [
            "label",
            "as_str",
            "from_label",
            "in_case",
            "extra",
            "VARIANTS",
            "LABELS",
            "eq",
            "ne",
            "fmt",
            "as_ref",
            "try_from",
            "from_str",
            "serialize",
            "deserialize",
        ] {
            let ident = format_ident!("{name}");
            assert_err(
                quote! {
                    #[cognomen(snake_case, #ident = "")]
                    enum Mode { A }
                },
                "conflicts with a generated cognomen item",
            );
        }
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(label = "x")]
                    A,
                }
            },
            "conflicts with a generated cognomen item",
        );
    }

    #[test]
    fn rejects_every_variant_error() {
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(rename = "")]
                    A,
                }
            },
            "cognomen rename must not be empty",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(alias = "")]
                    A,
                }
            },
            "cognomen alias must not be empty",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(rename = "a", rename = "b")]
                    A,
                }
            },
            "duplicate cognomen rename",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(blurb = "a", blurb = "b")]
                    A,
                }
            },
            "duplicate extra `blurb` on variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(blurb = "a")]
                    #[cognomen(blurb = "a")]
                    A,
                }
            },
            "duplicate #[cognomen(...)] on variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen()]
                    A,
                }
            },
            "variant #[cognomen(...)] requires rename = \"...\", alias = \"...\", unknown, or an extra method",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(foo)]
                    A,
                }
            },
            "expected `key = \"...\"` or `key(\"...\")`",
        );
        assert_err(
            quote! {
                #[cognomen(lower)]
                enum Collide { Zero, zero }
            },
            "generated label `zero` is shared by multiple variants",
        );
        assert_err(
            quote! {
                #[cognomen(lower)]
                enum Collide {
                    #[cognomen(rename = "zero")]
                    Other,
                    Zero,
                }
            },
            "generated label `zero` is shared by multiple variants",
        );
        assert_err(
            quote! {
                #[cognomen(lower)]
                enum Collide {
                    #[cognomen(alias = "zero")]
                    Other,
                    Zero,
                }
            },
            "generated label `zero` is shared by multiple variants",
        );
        assert_err(
            quote! {
                #[cognomen(lower)]
                enum Collide {
                    #[cognomen(alias = "x")]
                    A,
                    #[cognomen(alias = "x")]
                    B,
                }
            },
            "generated label `x` is shared by multiple variants",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(unknown)]
                    A,
                    #[cognomen(unknown)]
                    B,
                }
            },
            "only one variant may be marked #[cognomen(unknown)]",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(unknown, unknown)]
                    A,
                }
            },
            "duplicate cognomen unknown",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(unknown = "x")]
                    A,
                }
            },
            "unknown does not take a value",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    #[cognomen(unknown)]
                    Other { tag: &'static str },
                }
            },
            "#[cognomen(unknown)] requires a unit variant",
        );
        assert_err(
            quote! {
                #[cognomen(snake_case)]
                enum Mode {
                    Named { x: u8 },
                    #[cognomen(unknown)]
                    Other,
                }
            },
            "#[cognomen(unknown)] is only valid on a fieldless enum",
        );
    }

    #[test]
    fn accepts_forms() {
        let basic = ok(quote! {
            #[cognomen(snake_case, kebab-case)]
            enum Mode { SingleProcess, MultiProcess }
        });
        assert!(basic.contains("FromLabelError"));
        assert!(!basic.contains("Self :: Error"));
        assert!(!basic.contains("Self :: Err"));
        assert!(basic.contains("Variants"));
        assert!(basic.contains("Label"));
        assert!(basic.contains("in_case"));
        assert!(basic.contains("FromLabel"));
        assert!(!basic.contains("pub const fn label"));
        assert!(!basic.contains("label_snake"));

        ok(quote! {
            #[cognomen(snake, kebab, camel, pascal, screaming, lowercase, uppercase, title_case)]
            enum Mode { SingleProcess }
        });
        ok(quote! {
            #[cognomen(snake_case, kebab-case, prefix = "cfg")]
            enum Mode { A }
        });
        ok(quote! {
            #[cognomen(snake_case)]
            enum Mode {
                #[cognomen(rename = "io_error")]
                IoFailed,
                OpenFailed,
            }
        });
        let aliased = ok(quote! {
            #[cognomen(snake_case)]
            enum Role {
                #[cognomen(alias = "main")]
                Supervisor,
                Worker,
            }
        });
        assert!(aliased.contains("main"));
        assert!(aliased.contains("FromLabel"));
        ok(quote! {
            #[cognomen(snake_case)]
            enum Role {
                #[cognomen(alias = "main", alias = "lead")]
                Supervisor,
                Worker,
            }
        });
        let unknown = ok(quote! {
            #[cognomen(snake_case)]
            enum Kind {
                Trades,
                #[cognomen(unknown)]
                Other,
            }
        });
        assert!(unknown.contains("FromLabel"));
        ok(quote! {
            #[cognomen(snake_case)]
            enum Kind {
                #[cognomen(rename = "io_error", alias = "io")]
                IoFailed,
                #[cognomen(unknown, blurb = "catch-all")]
                Other,
            }
        });
        ok(quote! {
            #[cognomen(lower, blurb = "", hint = "n/a")]
            enum Mode {
                #[cognomen(blurb = "mic", hint = "in")]
                Mic,
                App,
            }
        });
        ok(quote! {
            #[cognomen(lower, blurb())]
            enum Mode {
                #[cognomen(blurb = "mic")]
                Mic,
                App,
            }
        });
        ok(quote! {
            #[cognomen(snake_case,)]
            enum Mode {
                #[cognomen(rename = "x",)]
                A,
            }
        });
        let fielded = ok(quote! {
            #[cognomen(snake_case)]
            enum HostError {
                #[cognomen(reason = "host open failed {cause}")]
                OpenFailed { cause: String },
                #[cognomen(reason = "host refused request {why}")]
                BadRequest { why: &'static str },
                Unit,
            }
        });
        assert!(fielded.contains("Formatted"));
        assert!(fielded.contains("empty"));
        assert!(!fielded.contains("from_label"));
        assert!(!fielded.contains("Variants"));
        ok(quote! {
            #[cognomen(snake_case)]
            enum Mode {
                #[cognomen(reason = "open failed {0}")]
                OpenFailed(String),
            }
        });
        let via_crate = ok(quote! {
            #[cognomen(lower, crate = ::other::cognomen)]
            enum Mode { A }
        });
        assert!(via_crate.contains("other"));

        let generic = ok(quote! {
            #[cognomen(snake_case)]
            enum Flag<const N: usize> { LeftHand, RightHand }
        });
        assert!(!generic.contains("Variants"));
        #[cfg(feature = "serde")]
        {
            assert!(generic.contains("const N"));
            assert!(generic.contains("Serialize"));
            assert!(generic.contains("Deserialize"));
        }

        let status = ok(quote! {
            #[cognomen(lower)]
            enum Status { Error, Err, Ok }
        });
        assert!(status.contains("FromLabelError"));
        assert!(!status.contains("Result < Self , Self :: Error >"));
        assert!(!status.contains("type Err = Self :: Err"));
    }
}
