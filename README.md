# cognomen

[![crates.io](https://img.shields.io/crates/v/cognomen.svg)](https://crates.io/crates/cognomen)
[![docs.rs](https://docs.rs/cognomen/badge.svg)](https://docs.rs/cognomen)
[![license](https://img.shields.io/crates/l/cognomen.svg)](https://github.com/Cardosaum/cognomen)

*Cognomen* is Latin for "an extra name given to a person or thing". This crate
gives every enum variant a second, stable string **label**.

Downstream crates use it as the seam between a Rust ident and the string a
config file, log line, CLI flag, or wire protocol actually carries. Case
conversion runs at compile time and is emitted as a `&'static str`.

Full API reference: <https://docs.rs/cognomen>

```toml
[dependencies]
cognomen = "0.4"
```

```rust
use cognomen::{Cognomen, Variants};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Mode {
    SingleProcess, // "single_process" / "single-process"
    MultiProcess,  // "multi_process"  / "multi-process"
}

assert_eq!(Mode::SingleProcess.label(), "single_process");
assert_eq!(Mode::MultiProcess.as_str(), "multi_process");
assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
assert_eq!(Mode::try_from("single-process"), Ok(Mode::SingleProcess));
assert!(Mode::SingleProcess == "single-process");
assert_eq!(Mode::VARIANTS.len(), 2);
```

The first case in `#[cognomen(...)]` is the default (`label()` / `as_str()` /
serde out). Every listed case also gets `{prefix}_{case}`.

## Case styles

| `#[cognomen(...)]` | short aliases | `VariantName` becomes |
|--------------------|---------------|-----------------------|
| `snake_case` | `snake` | `variant_name` |
| `kebab-case` / `kebab_case` | `kebab` | `variant-name` |
| `camelCase` / `camel_case` | `camel` | `variantName` |
| `PascalCase` / `pascal_case` | `pascal` | `VariantName` |
| `SCREAMING_SNAKE_CASE` | `screaming` | `VARIANT_NAME` |
| `lower` / `lowercase` | | `variantname` |
| `upper` / `uppercase` | | `VARIANTNAME` |
| `title` / `title_case` | | `Variant Name` |

## Attributes

**Container** (required): `#[cognomen(<case>, ...)]`

- One or more cases, comma-separated. First is the default.
- `prefix = "cfg"`: accessors become `cfg_snake`, `cfg_kebab`, ...
  (non-empty ASCII identifier; default `label`).
- `crate = ::other::cognomen`: generated path when you re-export this crate.

**Variant** (optional): `#[cognomen(rename = "io_error")]`

Sets the default label to that exact string and accepts it when parsing.
Other case accessors still convert from the ident.

```rust
#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Wire {
    #[cognomen(rename = "io_error")]
    IoFailed,
    OpenFailed,
}

assert_eq!(Wire::IoFailed.label(), "io_error");
assert_eq!(Wire::IoFailed.label_snake(), "io_failed");
assert_eq!(Wire::from_label("io_error").unwrap(), Wire::IoFailed);
```

Any other `name = "..."` is an [extra method](#extra-methods).

Violations (non-enum, missing case, collisions, bad prefix, bad extra, unknown
`{field}` placeholder) are compile errors, pinned by trybuild tests under
`tests/ui/`.

## Extra methods

Any `name = "..."` in `#[cognomen(...)]` besides `prefix`, `crate`, and
`rename` becomes an extra method.

On a variant, that string is the variant's value. On the enum, that string
is the default for variants that omit the key. If the enum does not set a
default, omitted variants use `as_str()` / `label()` (including `rename`).
`name()` on the enum is the same as `name = ""`.

`{field}` in a **variant** extra interpolates that named (or tuple-index)
payload. The method then returns `Formatted` instead of `&'static str`.
Enum-level defaults cannot contain placeholders. Write `{` / `}` as `{{` /
`}}`.

```rust
use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower)]
enum SourceKind {
    #[cognomen(blurb = "microphone / input device")]
    Mic,
    #[cognomen(blurb = "system-wide loopback")]
    System,
    App,
}

assert_eq!(SourceKind::Mic.as_str(), "mic");
assert_eq!(SourceKind::Mic.blurb(), "microphone / input device");
assert_eq!(SourceKind::App.blurb(), "app");
```

An enum-level default overrides `as_str()` for omitted variants:

```rust
use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower, blurb = "", hint = "n/a")]
enum SourceKind {
    #[cognomen(blurb = "microphone / input device", hint = "CoreAudio input")]
    Mic,
    App,
}

assert_eq!(SourceKind::Mic.blurb(), "microphone / input device");
assert_eq!(SourceKind::App.blurb(), "");
assert_eq!(SourceKind::Mic.hint(), "CoreAudio input");
assert_eq!(SourceKind::App.hint(), "n/a");
```

Fielded variants keep `label()` / `as_str()`. Parse, `Variants`, and serde
are omitted. Interpolate extras from the fields:

```rust
use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum HostError {
    #[cognomen(reason = "host backend unsupported {capability}")]
    Unsupported { capability: &'static str },
    #[cognomen(reason = "host open failed {cause}")]
    OpenFailed { cause: &'static str },
}

let e = HostError::OpenFailed { cause: "busy" };
assert_eq!(e.as_str(), "open_failed");
assert_eq!(e.reason(), "host open failed busy");
```

Several extras can coexist. They are not accepted by `from_label` or
serde. Names that collide with generated items (`label`, `as_str`,
`{prefix}_{case}`, ...) are compile errors.

## Generated API

For `#[cognomen(snake_case, kebab-case)]` on `E`:

| item | notes |
|------|--------|
| `label()` / `as_str()` | default case, or `rename` |
| `label_snake()`, `label_kebab()`, ... | one method per declared case |
| `{name}()` | each extra; `as_str()` if omitted, unless the enum sets a default. `{field}` interpolation returns `Formatted` |
| `Variants` | `E::VARIANTS` / `E::LABELS` after `use cognomen::Variants` (non-generic, fieldless). Trait items, so they cannot clash |
| `AsRef<str>`, `PartialEq<str>` | compare against any declared label |
| `TryFrom<&str>`, `FromStr`, `from_label` | fieldless enums; uses `core` |
| `Serialize` / `Deserialize` | feature `serde`, fieldless; out is `label()`, in accepts any declared case |
| `T::value_parser()` | feature `clap`; import `cognomen::clap::ArgType` in the binary |

## Features

| feature | default | unlocks |
|---------|---------|---------|
| `std` | yes | `alloc` + `std::error::Error` for `FromLabelError` |
| `alloc` | via `std` | `FromLabelError.input` stores the unmatched string |
| `serde` | no | `Serialize` / `Deserialize` |
| `clap` | no | `T::value_parser()` for clap flags; implies `std` |

`no_std`, including embedded:

```toml
cognomen = { version = "0.4", default-features = false }
```

Labels, parse, `AsRef`, and `Variants` use only `core`. Add
`features = ["alloc"]` to keep the unmatched string on parse errors. Add
`features = ["serde"]` for wire formats. Enable `clap` in the binary that
owns the CLI, not in a `no_std` kernel:

```rust
use clap::Parser;
use cognomen::clap::ArgType;

#[derive(Parser)]
struct Cli {
    #[arg(long, value_parser = Kind::value_parser())]
    kind: Kind,
}
```

## Word splitting

Idents split on ASCII camel-case boundaries. Acronyms stay together
(`HTTPResponse` -> `http_response`). Digits stay glued (`Utf8` -> `utf8`,
`IPv4` -> `i_pv4`). Re-Pascal of an acronym title-cases the run
(`HTTPResponse` -> `HttpResponse`).

## MSRV

Rust **1.71.1** for default features, `alloc`, and `serde`. Floor is the
pinned `proc-macro2` (rustc 1.71+). Cognomen itself only needs
`Option::is_some_and` (1.70).

The `clap` feature is not part of that floor. It needs a rustc that can
compile the resolved clap crate (clap 4.6 needs 1.85). CI checks `clap` on
stable with `--all-features`.

## Publishing

`cognomen` depends on `cognomen-macros` by version. The first crates.io
upload must publish `cognomen-macros` first, then `cognomen`. New crate
names cannot use trusted publishing; that first upload is manual. Later
releases are prepared and published by release-plz on `main`.

## License

Dual-licensed under **MIT OR Apache-2.0**, see
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
Copyright (c) 2026 Matheus Cardoso.
