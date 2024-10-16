# aqua-guardian

## Building/Running

1. Build the binaries

```sh
cargo build --bin guardian
```

2. Create a `.env` file based on `.env.template`
3. Run your guardian binary

```sh
cargo run --bin guardian
```

### Dependencies

- [Rust (nightly)](https://rustup.rs/)

## Structure

`Cargo.toml` - workspace specification
`guardian` - final executable
`.env` - configure ip address, wallet addresss, infura API-Key etc.
(+add crates using `cargo new --lib <crate_name>`)
