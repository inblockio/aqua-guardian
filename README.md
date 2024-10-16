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

## Important Documentation

For a comprehensive understanding of the project, please refer to the following documents:

- **Architecture Specification**: This document outlines the high-level and detailed architecture of our system.  
  [Download PDF](docs/Architecture_Specification_fine.pdf)

- **Handshake and Certificate Exchange Protocol**: Details the process and protocols used for secure handshakes and certificate exchanges in our system.  
  [Download PDF](docs/Handshake_Certificate_Exchange.pdf)

These documents are essential for developers, reviewers, and anyone involved in the project to gain a deeper insight into the system's design and operation.
