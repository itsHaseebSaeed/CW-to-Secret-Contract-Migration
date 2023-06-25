# Updating `Cargo.toml`

This document outlines changes made to contract dependencies in `Cargo.toml`.

## 1. Change the name package name in Cargo.toml

```toml
name = "cw-counter"
..
```

Replace With:

```toml
name = "secret-counter"
..
```

## 2. Change the contract name in /bin/schema.rs

```Rust
use cw_counter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
```

With:

```Rust
use secret_counter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
```

## 3. Build and run tests

After these changes, build and test the project to check for any errors:

```Rust
cargo build
cargo test
```

That's it for the Cargo.toml modifications. Proceeding with these changes should get your Secret Network contract off to a solid start.
