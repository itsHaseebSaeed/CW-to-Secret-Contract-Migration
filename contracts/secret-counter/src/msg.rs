use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Deps, StdResult};
use secret_toolkit::permit::Permit;

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
    pub prng_seed: Binary,
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {},
    CreateViewingKey { entropy: String },
    SetViewingKey { key: String },
}

#[cw_serde]
pub enum ExecuteAnswer {
    // Native
    CreateViewingKey { key: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
    #[returns(GetUserCountResponse)]
    GetUserCount { addr: Addr, key: String },
    #[returns(QueryWithPermit)]
    WithPermit {
        permit: Permit<CounterContractPermissions>,
        query: QueryWithPermit,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryWithPermit {
    #[returns(GetUserCountResponse)]
    GetUserCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}

#[cw_serde]
pub struct GetUserCountResponse {
    pub count: i32,
}

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

#[cw_serde]
pub enum CounterContractPermissions {
    UserCount,
    Owner,
}
