# Const Code Gen

A rust application to generate js and rust constant definitions to multiple
modules from a toml file

## Build

```sh
cargo build --release
cp target/release/constcodegen .
```

## Usage

```
Usage: constcodegen generate [OPTIONS] <input>

Arguments:
  <input>

Options:
      --language <language>  [default: js]
      --root-dir <root-dir>  [default: .]
  -h, --help                 Print help information
```

## Run

### Single Rust File (to Standard Output)

```sh
./constcodegen generate --language rust-single samples/simple.toml
```

```rust
mod foo {
    pub const DECIMAL: f64 = 1.5;
    pub const INTEGER: i64 = 42;
    pub const LOGIC: bool = true;
    pub const NS: &str = "Foo";
}

mod http {
    pub const NS: &str = "HTTP";
}
```

```sh
./constcodegen generate --language js-single samples/simple.toml
```

```js
// foo
export const FOO_DECIMAL = 1.5;
export const FOO_INTEGER = 42;
export const FOO_LOGIC = true;
export const FOO_NS = "Foo";


// http
export const HTTP_NS = "HTTP";
```

## Muliple JS Files

```sh
./constcodegen generate --language js --root-dir js-src samples/simple.toml
```

```
js-src/
├── foo.js
└── http.js
```

```js
export const DECIMAL = 1.5;
export const INTEGER = 42;
export const LOGIC = true;
export const NS = "Foo";
```

## Muliple Rust Files

```sh
./constcodegen generate --language rust --root-dir rust-src samples/simple.toml
```

```
rust-src/
├── foo.rs
└── http.rs
```

```
pub const DECIMAL: f64 = 1.5;
pub const INTEGER: i64 = 42;
pub const LOGIC: bool = true;
pub const NS: &str = "Foo";
```

## License

MIT
