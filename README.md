# cognomen

*Cognomen* is Latin for "an extra name given to a person or thing". It is a
zero-cost procedural macro that gives every unit-like variant of an enum a
second name: a stable, case-configured string **label**.

Case conversion happens at compile time and is emitted as a `&'static str`.

```rust
use cognomen::Cognomen;

// The first case listed is the default; every case gets a `label_<case>`
// accessor. `label()` is the default.
#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Mode {
    SingleProcess, // "single_process" / "single-process"
    MultiProcess,  // "multi_process"  / "multi-process"
}

assert_eq!(Mode::SingleProcess.label(), "single_process");
assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
```

## Case styles

| `#[cognomen(...)]`       | short aliases                         | `VariantName` becomes |
|--------------------------|---------------------------------------|-----------------------|
| `snake_case`             | `snake`                               | `variant_name`        |
| `kebab-case` / `kebab_case` | `kebab`                            | `variant-name`        |
| `camelCase` / `camel_case`  | `camel`                            | `variantName`         |
| `PascalCase` / `pascal_case` | `pascal`                          | `VariantName`         |
| `SCREAMING_SNAKE_CASE` / `screaming_snake_case` | `screaming` | `VARIANT_NAME`        |
| `lower`                  | `lowercase`                           | `variantname`         |
| `upper`                  | `uppercase`                           | `VARIANTNAME`         |

List more than one case comma-separated; the **first** is the default returned
by `label()`, and every listed case gets its own `label_<case>` accessor.

## Requirements

- Derive on **enums only**.
- **Unit variants only** (no fields).
- At least one variant.
- A `#[cognomen(<case style>)]` container attribute with one or more
  comma-separated cases. The **first** case is the default.

Violations are compile-time errors. Failure cases are pinned by
[trybuild](https://docs.rs/trybuild) UI tests under `tests/ui/`.

## Generated API

For an enum `E` with `#[cognomen(snake_case, kebab-case)]`:

- `E::variant.label() -> &'static str` — default (first) case.
- `E::variant.label_snake() -> &'static str`
- `E::variant.label_kebab() -> &'static str`

Default prefix is `label`. Use `prefix = "..."` to change it:

```rust
#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case, prefix = "my_label")]
enum Mode { SingleProcess, MultiProcess }
assert_eq!(Mode::SingleProcess.label(), "single_process");
assert_eq!(Mode::SingleProcess.my_label_kebab(), "single-process");
```

| case                   | accessor                |
|------------------------|-------------------------|
| `snake_case`           | `label_snake`           |
| `kebab-case`           | `label_kebab`           |
| `camelCase`            | `label_camel`           |
| `PascalCase`           | `label_pascal`          |
| `SCREAMING_SNAKE_CASE` | `label_screaming_snake` |
| `lower`                | `label_lower`           |
| `upper`                | `label_upper`           |

Reverse direction — any declared case parses back to the variant:

- `TryFrom<&str> for E`
- `FromStr for E`

Both return a `FromLabelError` when nothing matches (`Display` + `Error`).

```rust
assert_eq!("single-process".parse::<Mode>(), Ok(Mode::SingleProcess));
assert_eq!(Mode::try_from("multi_process"), Ok(Mode::MultiProcess));
```

## MSRV

Rust **1.71.1**, determined with `cargo-msrv` (bisect, default deps). The floor
is set by the pinned `proc-macro2` dependency (needs rustc 1.71 or newer);
cognomen's own code only requires `Option::is_some_and` (Rust 1.70).

## License

Dual-licensed under **MIT OR Apache-2.0**, see
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
Copyright (c) 2026 Matheus Cardoso.
