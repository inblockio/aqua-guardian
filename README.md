# aqua-guardian

## Building/Running

```sh
cargo build --bin guardian
```

### Dependencies

- [Rust (nightly)](https://rustup.rs/)

## Structure

`Cargo.toml` - workspace specification
`guardian` - final executable
`.env` - configure ip address, wallet addresss, infura API-Key etc.
(+add crates using `cargo new --lib <crate_name>`)
