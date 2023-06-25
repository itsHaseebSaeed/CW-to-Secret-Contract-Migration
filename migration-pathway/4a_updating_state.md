# Updating State.rs

A handful of standard dependencies need to be substituted with the alternatives provided by `secret_toolkit`. The following instructions will guide you on making these replacements within your `state.rs` file.

## 1. Swap `cw_storage_plus` Dependencies with `secret-toolkit`

```Rust
use cw_storage_plus::{Item, Map};
```

Replace with:

```Rust
use secret_toolkit::storage::{Item,Keymap};
```

The secret_toolkit is Secret Network's alternative to cw_storage_plus. While they both handle storage packages, they have different names and offer different levels of flexibility.

## 2. Modify the STATE Item Key

The Item key needs to be adjusted since secret_toolkit necessitates a byte string literal instead of a traditional string literal for the key.

```Rust
pub const STATE: Item<State> = Item::new("state");
```

Replace with:

```Rust
pub const STATE: Item<State> = Item::new(b"state");
```

## 3. Edit the USER_STATE Item Key

The `Map` in `cw_storage_plus` is similar to `Keymap` in secret toolkit*. The `Map` key also needs modification, as `secret toolkit` requires a byte string literal instead of a string literal for the key.

```Rust
pub const STATE: Map<Addr, UserState> = Map::new("user_state");
```

Replace with:

```Rust
pub const USER_STATE: Keymap<Addr, UserState> = Keymap::new(b"user_state");
```

Upon making these amendments, all errors within state.rs should be successfully rectified.
