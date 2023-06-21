# Change Log

Copy the `cw-counter` folder and rename it `secret-counter`.

## Cargo.toml

This document outlines changes made to contract dependencies in `Cargo.toml`.

- Change the name package name in Cargo.toml

```toml
name = "cw-counter"
```

With:

```toml
name = "secret-counter"
```

- Change the contract name in /bin/schema.rs

```toml
use cw_counter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
```

With:

```toml
use secret_counter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
```

Run

```Rust
cargo build
cargo test
```

to check for any errors

## Multi-Tests

- Secret Network does not officially support multi-tests. Consequently, it is recommended to remove the `integration_tests.rs` file from your project.

- Remove the following lines from your `Cargo.toml` file:

```toml
[dev-dependencies]
cw-multi-test = "0.15.1"
```

- There are some other teams that have worked with multi-test. [This](https://github.com/securesecrets/secret-plus-utils) package by secure secrets can be an alternative to
  secret networks storage package
  aka secret-toolkit. But it's recommended to use secret-toolkit for advanced storage packages.

## CosmWasm Dependencies

- Replace the existing dependencies `cosmwasm-schema`, `cosmwasm-std`, and `cosmwasm-storage` with the Secret Network versions.

- Remove the following lines from your `Cargo.toml` file:

```toml
[dependencies]
cosmwasm-schema = "1.1.3"
cosmwasm-std = "1.1.3"
cosmwasm-storage = "1.1.3"
```

and replace the line with

```toml
cosmwasm-schema = { version = "1.1.8" }
cosmwasm-std = { git = "https://github.com/scrtlabs/cosmwasm/", default-features = false, tag = "v1.1.9-secret" }
cosmwasm-storage = { git = "https://github.com/scrtlabs/cosmwasm/", tag = "v1.1.9-secret" }
secret-toolkit = { git = "https://github.com/scrtlabs/secret-toolkit", features = [
  "permit",
  "viewing-key",
], rev = "9b74bdac71c2fedcc12246f18cdfdd94b8991282" }
```

- Secret Network has it's own version of cosmwasm and hence we need to change the dependencies

### TODO add more explaination to this contract

Now you'll be seeing some errors in `contract.rs`, `helpers.rs`, `integrations.rs` and `state.rs`

- 1. First it's best to remove `integration_tests.rs` because as I mentioned before secret network doesn't supports multi-tests.

## lib.rs

Remove this line from `lib.rs` as well

```Rust
pub mod integration_tests;
```

- 2. Remove `helpers.rs` since in this example `helper.rs` are only used in `integration_tests.rs`

Remove this line from `lib.rs` as well

```Rust
pub mod helpers;
```

## State.rs

- 2. Replace the

```Rust
use cw_storage_plus::{Item, Map};
```

With:

```Rust
use secret_toolkit::storage::{Item,Keymap};
```

secret_toolkit is secret networks's the alternative to cw_storage_plus. The storage packages have same concepts but have different names and flexability.

- 3 Now edit the STATE, Item key needs to be changed as well because secret toolkit required byte string literal instead of string literal for the key:

```Rust
pub const STATE: Item<State> = Item::new("state");
```

Replace with:

```Rust
pub const STATE: Item<State> = Item::new(b"state");
```

- 4 Same edit USER_STATE, Map is more or less similar to KeyMap in secret-toolkit. Map key needs to be changed as well because secret toolkit required byte string literal instead of string literal for the key

```Rust
pub const STATE: Map<Addr, UserState> = Map::new("user_state");
```

Replace with:

```Rust
pub const USER_STATE: Keymap<Addr, UserState> = Keymap::new(b"user_state");
```

Looks like all the errors in `state.rs` are resolved.

## contract.rs

You might be seeing alot of error but these errors. Let's disect each error one by one.

1. `use cw2::set_contract_version;`.

Secret Network doesn't have a cw2 alternative so we'll have to delete

```rust
use cw2::set_contract_version;
```

and

```rust
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
```

CW2 is used set the contract version, this is needed for migrations

//TODO: how to do contract migration on secret compared to other networks

### increment function()

As mentioned before Items/Maps on CW are different in terms on helper functions

Try changing the increment function yourself first.

This is the migrated version of the contract:

```Rust
    pub fn increment(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state: State| {
            state.count += 1;
            Ok(state)
        })?;

        let sender = info.sender;
        let mut user: UserState = USER_STATE
            .get(deps.storage, &sender)
            .unwrap_or(UserState { count: 0 });
        user.count += 1;
        USER_STATE.insert(deps.storage, &sender, &user)?;

        Ok(Response::new().add_attribute("action", "increment"))
    }
```

As you can see that we replaced `Map` with `KeyMap`. KeyMap uses `get` and `insert` instead of `load` and `save`.

Now using this try to change the user_count query yourself.

Migrated solution:

```Rust

    pub fn user_count(deps: Deps, user: Addr) -> StdResult<GetUserCountResponse> {
        let user_state = USER_STATE.load(deps.storage, user)?;
        Ok(GetUserCountResponse {
            count: user_state.count,
        })
    }

```
