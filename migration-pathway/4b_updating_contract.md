# Refactoring contract.rs

You might encounter a plethora of errors popping up from your `contract.rs` file. Fear not; we will systematically dissect and rectify each of these issues.

## 1. Handling `cw2::set_contract_version`

There's no Secret Network equivalent for `cw2`, hence the associated lines need to be excised:

```rust
use cw2::set_contract_version;

//...

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
//...
set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
//...
}
```

The cw2 module is used to set the contract version, which is crucial for migrations. At present, we don't have a strategy for handling contract migrations on Secret Network compared to other networks. Read more about migration from [secret network docs](https://docs.scrt.network/secret-network-documentation/development/development-concepts/contract-migration).

## 2. Revamping the increment function

As already highlighted, the Item/Map from CosmWasm have slight variances in their helper functions compared to those of the Secret Network. Below, we provide the Secret Network rendition of the increment function.

Here's the migrated variant of the contract:

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

In this version, Map is replaced with KeyMap, which uses get and insert instead of load and save.

Now, try to modify the user_count query **YOURSELF** based on these changes.

Ok let's check the migrated solution:

```Rust
    pub fn user_count(deps: Deps, addr: Addr) -> StdResult<GetUserCountResponse> {
        let user_state = USER_STATE
            .get(deps.storage, &addr)
            .unwrap_or(UserState { count: 0 });
        Ok(GetUserCountResponse {
            count: user_state.count,
        })
    }
```

Run the test now on the `secret-counter`. The test should run seamlessly without any issues.

```Rust
cargo test
```
