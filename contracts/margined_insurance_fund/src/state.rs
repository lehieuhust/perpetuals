use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Deps, DepsMut, StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: &[u8] = b"vamm-list";
pub const VAMM_LIMIT: usize = 3usize;

// function checks if an addr is already added and adds it if not
// We also check that we have not reached the limit of vAMMs here
pub fn save_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // check if there is a vector
    let mut vamm_list: Vec<Addr> = singleton_read(deps.storage, VAMM_LIST)
        .may_load()?
        .unwrap_or_default();

    // check if we already added the vamm
    if vamm_list.contains(&input) {
        return Err(StdError::GenericErr {
            msg: "This vAMM is already added".to_string(),
        });
    };

    // check if we have reached the capacity
    if vamm_list.len() >= VAMM_LIMIT {
        return Err(StdError::GenericErr {
            msg: "The vAMM capacity is already reached".to_string(),
        });
    };

    // add the vamm to the vector
    vamm_list.push(input);
    singleton(deps.storage, VAMM_LIST).save(&vamm_list)
}

// this function reads Addrs stored in the VAMM_LIST.
// note that this function ONLY takes the first VAMM_LIMIT terms
pub fn read_vammlist(deps: Deps, limit: usize) -> StdResult<Vec<Addr>> {
    match singleton_read::<Vec<Addr>>(deps.storage, VAMM_LIST).may_load()? {
        None => Err(StdError::GenericErr {
            msg: "No vAMMs are stored".to_string(),
        }),
        Some(value) => {
            let take = limit.min(value.len());
            Ok(value[..take].to_vec())
        }
    }
}

// this function checks whether the vamm is stored already
pub fn is_vamm(storage: &dyn Storage, input: Addr) -> bool {
    if let Ok(value) = singleton_read::<Vec<Addr>>(storage, VAMM_LIST).load() {
        return value.contains(&input);
    }
    false
}

// this function deletes the entry under the given key
pub fn remove_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // check if there are any vamms stored
    let mut vamm_list = match singleton_read::<Vec<Addr>>(deps.storage, VAMM_LIST).may_load()? {
        None => {
            return Err(StdError::GenericErr {
                msg: "No vAMMs are stored".to_string(),
            })
        }
        Some(value) => value,
    };

    // check if the vamm is added
    if !vamm_list.contains(&input) {
        return Err(StdError::GenericErr {
            msg: "This vAMM has not been added".to_string(),
        });
    }

    // change vamm_list
    let index = vamm_list.clone().iter().position(|x| x.eq(&input)).unwrap();
    vamm_list.swap_remove(index);

    // saves the updated vamm_list
    singleton(deps.storage, VAMM_LIST).save(&vamm_list)
}

#[cw_serde]
pub struct Config {
    pub engine: Addr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
