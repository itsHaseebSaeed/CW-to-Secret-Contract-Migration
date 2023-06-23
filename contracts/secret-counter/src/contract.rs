#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use secret_toolkit::crypto::sha_256;

use crate::error::ContractError;
use crate::msg::{ExecuteAnswer, ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, UserState, STATE, USER_STATE};

use secret_toolkit::permit::Permit;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

const PREFIX_REVOKED_PERMITS: &str = "prefix_revoked_permits";

// version info for migration info

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    STATE.save(deps.storage, &state)?;

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);
    ViewingKey::set_seed(deps.storage, &prng_seed_hashed);

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps, info),
        ExecuteMsg::SetViewingKey { key } => execute::try_set_key(deps, info, key),
        ExecuteMsg::CreateViewingKey { entropy, .. } => {
            execute::try_create_key(deps, env, info, entropy)
        }
    }
}

pub mod execute {

    use super::*;

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

    pub fn try_set_key(
        deps: DepsMut,
        info: MessageInfo,
        key: String,
    ) -> Result<Response, ContractError> {
        ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
        Ok(Response::new())
    }

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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
        QueryMsg::WithPermit { permit, query } => {
            query::permit_queries(
                deps, // env is sometimes not needed
                env, permit, query,
            )
        }
        _ => query::viewing_keys_queries(deps, msg),
    }
}

pub mod query {

    use cosmwasm_std::{Addr, StdError};

    use crate::{
        error,
        msg::{CounterContractPermissions, GetUserCountResponse, QueryWithPermit},
        state::USER_STATE,
    };

    use super::*;

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

    pub fn permit_queries(
        deps: Deps,
        env: Env,
        permit: Permit<CounterContractPermissions>,
        query: QueryWithPermit,
    ) -> Result<Binary, StdError> {
        // Validate permit content
        let contract_address = env.contract.address;

        let account = secret_toolkit::permit::validate(
            deps,
            PREFIX_REVOKED_PERMITS,
            &permit,
            contract_address.into_string(),
            None,
        )?;

        // Permit validated! We can now execute the query.
        match query {
            QueryWithPermit::GetUserCount {} => {
                if !permit.check_permission(&CounterContractPermissions::UserCount) {
                    return Err(StdError::generic_err(format!(
                        "No permission to query balance, got permissions {:?}",
                        permit.params.permissions
                    )));
                }
                let address = Addr::unchecked(account);

                to_binary(&user_count(deps, address)?)
            }
        }
    }

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }

    pub fn user_count(deps: Deps, addr: Addr) -> StdResult<GetUserCountResponse> {
        let user_state = USER_STATE
            .get(deps.storage, &addr)
            .unwrap_or(UserState { count: 0 });
        Ok(GetUserCountResponse {
            count: user_state.count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{CounterContractPermissions, GetUserCountResponse};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, StdError};
    use secret_toolkit::permit::{validate, PermitParams, PermitSignature, PubKey};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            prng_seed: Binary::default(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            prng_seed: Binary::default(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("secret_user", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn query_with_vk() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            prng_seed: Binary::default(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("secret_user", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);

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

    #[test]

    fn query_with_permit() {
        const USER: &str = "secret12eqwlqwcu2cundyg8ytnuhjnr29fh3zgt68602";
        const CONTRACT_ADDRESS: &str = "cosmos2contract";
        const PERMIT_NAME: &str = "CounterPermit";
        const CHAIN_ID: &str = "secret-4";

        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            prng_seed: Binary::default(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // beneficiary can release it
        let info = mock_info(USER, &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);

        //Using Permit to check the counter.

        //1) Checking signature validity
        let permit: Permit<CounterContractPermissions> = Permit{
                    params: PermitParams {
                        allowed_tokens: vec![CONTRACT_ADDRESS.to_string()],
                        permit_name: PERMIT_NAME.to_string(),
                        chain_id: CHAIN_ID.to_string(),
                        permissions: vec![CounterContractPermissions::UserCount]
                    },
                    signature: PermitSignature {
                        pub_key: PubKey {
                            r#type: "tendermint/PubKeySecp256k1".to_string(),
                            value: Binary::from_base64("Ar1oSw8miosG4fJqucfb8c+HpHfr5dSvyGC5kQG9hIUy").unwrap(),
                        },
                        signature: Binary::from_base64("2pZXHXJZJw8s5q3ju0z9d5Njh77GtgEgLzdXcNHVg91H93MpuTHWPy99MQCUaw5O2dRE44G2GWQDqCGTwlO47w==").unwrap()
                    }
                };

        let address = validate::<CounterContractPermissions>(
            deps.as_ref(),
            PREFIX_REVOKED_PERMITS,
            &permit,
            CONTRACT_ADDRESS.to_string(),
            None,
        )
        .unwrap();

        assert_eq!(
            address,
            "secret12eqwlqwcu2cundyg8ytnuhjnr29fh3zgt68602".to_string()
        );

        //Querying with correct address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::WithPermit {
                permit: permit,
                query: crate::msg::QueryWithPermit::GetUserCount {},
            },
        )
        .unwrap();
        let value: GetUserCountResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.count);
    }
}
