# 1. Adding Viewing keys

Viewing keys can be compared to passwords. When performing an authenticated query using viewing keys, you need to provide a public address and its associated viewing key. This process is similar to authenticating your username (public address) and password (viewing key). For more details and explaination please check [implementing Viewing Keys](https://scrt.university/pathways/33/implementing-viewing-keys-and-permits).

Viewing keys need to be created/set by sending a transaction to the contract. At the start you were told to add this dependency.

```Rust
secret-toolkit = { git = "https://github.com/scrtlabs/secret-toolkit", features = [
  "permit",
  "viewing-key",
], rev = "9b74bdac71c2fedcc12246f18cdfdd94b8991282" }
```

With `viewing-key` package the implementation process of viewing keys is streamlined.

## Defining the messages

In your message type definitions (typically in the msg.rs file), you need to make the following additions:

1. Add execute function message to create viewing keys
2. Create viewing key query interface
3. Implement get_validation_params method (recommended but not strictly necessary)

### 1. Add execute function message to create viewing keys (`msg.rs`)

```Rust
pub enum ExecuteMsg {
    ....
    CreateViewingKey { entropy: String },
    SetViewingKey { key: String },
}
```

CreateViewingKey and SetViewingKey are two types of messages. CreateViewingKey generates a random viewing key and associates it with the user's address, while SetViewingKey allows the user to set a specific key of their choice. Although SetViewingKey provides more user control, it risks weak key selection. From an end user's perspective, the two operations may appear identical.

### 2. Create viewing key query interface (`msg.rs`)

The contract initiator supplies a prng_seed for the random number generator. This seed, accessible only by the contract, is used in the creation of viewing keys.
Add `prng_seed` to the `InstantiateMsg`.

```Rust
#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
    pub prng_seed: Binary,
}
```

ExecuteAnswer enum is used to define the response type a smart contract provides upon its execution. Specifically, the CreateViewingKey variant is returned when a new viewing key is generated by the contract, encapsulating the generated key within the key field.

```Rust
#[cw_serde]
pub enum ExecuteAnswer {
    // Native
    CreateViewingKey { key: String },
}
```

Query interfaces are already in the contract but now add key field because the querier needs to provide the addr (public account address) and key (viewing key) fields. The contract verifies these inputs and responds with the private data if they are valid. Otherwise, an error occurs.

```Rust
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    ...
    #[returns(GetUserCountResponse)]
    GetUserCount { addr: Addr, key:String },
}


#[cw_serde]
pub struct GetUserCountResponse {
    pub count: i32,
}

```

### 3. Implement get_validation_params method (`msg.rs`)

We're implementing helper functions to confirm the viewing key for a provided user address.

import `Deps`, `StdResult` and `Binary` from cosmwasm_std in `msg.rs`

```Rust
use cosmwasm_std::{Addr, Binary, Deps, StdResult};
```

```Rust
impl QueryMsg {
    pub fn get_validation_params(&self, deps: Deps) -> StdResult<(Vec<Addr>, String)> {
        match self {
            Self::GetUserCount { addr, key } => {
                let address = deps.api.addr_validate(addr.as_str())?;
                Ok((vec![address], key.clone()))
            }
            _ => panic!("This query type does not require authentication"),
        }
    }
}

```

We also check the validity of the address by using the api.addr_validate method. If the address is valid, the function returns a tuple containing the validated address and the viewing key.

## Updating Contract.rs

The following steps need to be taken for the main body of the contract (typically in contract.rs):

1. import viewing key package from Secret Toolkit

2. `set_seed` in `instanstiate`

3. execute function(s) to create viewing keys

4. write code in query entry point to handle viewing key queries. It can be modularized into a separate function that handles all vk queries

5. create functions to handle the queries (as you would with any query)

### 1. import vk toolkit, sha256 and ExecuteAnswer

```Rust
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};
use secret_toolkit::crypto::sha_256;
use crate::msg::{ExecuteAnswer, ...};

```

### 2. Set Seed for `CreateViewingKeys`

```Rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
...

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);
    ViewingKey::set_seed(deps.storage, &prng_seed_hashed);

...
}
```

### 3.  Execute function(s) to create viewing keys

We're allowing Execute function to handle set and create viewing key functions:

```Rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetViewingKey { key } => execute::try_set_key(deps, info, key),
        ExecuteMsg::CreateViewingKey { entropy, .. } => {
            execute::try_create_key(deps, env, info, entropy)
        }
        // ...
    }
}
```

We'll first look into SetViewingKey, try_set_key function implements the ViewingKey::set associated function from the ViewingKeyStore trait

```Rust
pub fn try_set_key(deps: DepsMut, info: MessageInfo, key: String) -> Result<Response, ContractError> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
    Ok(Response::new())
}
```

As you might remember that we used `ViewingKey::set_seed` to set the prng_seed. This seed is used by the random number generator to create veiwing keys.

`try_create_key` function:

```Rust
    pub fn try_create_key(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        entropy: String,
    ) -> Result<Response, ContractError> {
        let key = ViewingKey::create(
            deps.storage,
            &info,
            &env,
            info.sender.as_str(),
            entropy.as_ref(),
        );

        Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CreateViewingKey { key })?))
    }
```

The ViewingKey struct provides the create associated function that handles the random number generation internally. The function saves the viewing key in the contract storage, associating it with the sender's public address.

### 4. Query entry point to handle vk queries

Create a function that only handles the viewing key queries:

```Rust

pub mod query {

  ...

      pub fn viewing_keys_queries(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        let (addresses, key) = msg.get_validation_params(deps)?;

        for addr in addresses {
            let result = ViewingKey::check(deps.storage, addr.as_str(), key.as_str());
            if result.is_ok() {
                return match msg {
                    // Base
                    QueryMsg::GetUserCount { addr, .. } => {
                        Ok(to_binary(&query::user_count(deps, addr)?)?)
                    }
                    _ => panic!("This query type does not require authentication"),
                };
            }
        }

        panic!("Wrong viewing key for this address or viewing key not set")
    }
}
```

Replace `QueryMsg::GetUserCount` in `query` function

```Rust
QueryMsg::GetUserCount { addr } => to_binary(&query::user_count(deps, addr)?),
```

With:

```Rust
_ => query::viewing_keys_queries(deps, msg),
```

It means that any Query Message that doesn't match any other message are directed to `viewing_keys_queries`

```Rust

    pub fn viewing_keys_queries(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        let (addresses, key) = msg.get_validation_params(deps)?;

        for addr in addresses {
            let result = ViewingKey::check(deps.storage, addr.as_str(), key.as_str());
            if result.is_ok() {
                return match msg {
                    // Base
                    QueryMsg::GetUserCount { addr, .. } => {
                        Ok(to_binary(&query::user_count(deps, addr)?)?)
                    }
                    _ => panic!("This query type does not require authentication"),
                };
            }
        }

      return Err(StdError::GenericErr {
            msg: "Wrong viewing key for this address or viewing key not set".to_string(),
        });
    }

```

### 5. Modifying Unit-test

Viewing key is added and now it's time to modify the unit tests to check the results. We'll change increment test. Read though the test to understand. Remember to import all the packages needed for unit-test.

```Rust
 use super::*;
 use crate::msg::GetUserCountResponse;
 use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
 use cosmwasm_std::{coins, from_binary, Addr, StdError};

    ....


 #[test]
    fn query_with_vk() {
        //..
        //Using Viewing key to check the counter.

        //Setting viewing_key first
        let info = mock_info("secret_user", &coins(2, "token"));
        let msg = ExecuteMsg::SetViewingKey {
            key: "vk_1".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        //Querying with wrong address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUserCount {
                addr: Addr::unchecked("not_secret_user"),
                key: "vk_1".to_string(),
            },
        );
        assert_eq!(
            res.unwrap_err(),
            StdError::GenericErr {
                msg: "Wrong viewing key for this address or viewing key not set".to_string(),
            }
        );

        //Querying with correct address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetUserCount {
                addr: Addr::unchecked("secret_user"),
                key: "vk_1".to_string(),
            },
        )
        .unwrap();
        let value: GetUserCountResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.count);
    }
```

Now run test to check the final results.

```Rust
cargo test
```