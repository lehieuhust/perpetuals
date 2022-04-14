use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, Env, ReplyOn, StdResult, Storage, SubMsg,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::AssetInfo;

use crate::{
    querier::query_vamm_calc_fee,
    state::{read_config, State},
};

use margined_perp::margined_vamm::CalcFeeResponse;
use margined_perp::querier::query_token_balance;

pub fn execute_transfer_from(
    storage: &dyn Storage,
    owner: &Addr,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;

    let msg: CosmosMsg = match config.eligible_collateral {
        AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: vec![Coin { denom, amount }],
        }),
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: owner.to_string(),
                recipient: receiver.to_string(),
                amount,
            })?,
        }),
    };

    let transfer_msg = SubMsg {
        msg,
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

pub fn execute_transfer(
    storage: &dyn Storage,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: receiver.to_string(),
            amount,
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

pub fn execute_transfer_to_insurance_fund(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(deps.storage)?;

    let token_balance = query_token_balance(
        deps,
        config.eligible_collateral.clone(),
        env.contract.address,
    )?;

    let amount_to_send = if token_balance < amount {
        token_balance
    } else {
        amount
    };

    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.insurance_fund.to_string(),
            amount: amount_to_send,
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

// Transfers the toll and spread fees to the the insurance fund and fee pool
pub fn transfer_fees(
    deps: Deps,
    from: Addr,
    vamm: Addr,
    notional: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let config = read_config(deps.storage)?;

    let CalcFeeResponse {
        spread_fee,
        toll_fee,
    } = query_vamm_calc_fee(&deps, vamm.into_string(), notional)?;

    let mut messages: Vec<SubMsg> = vec![];

    if !spread_fee.is_zero() {
        let msg =
            execute_transfer_from(deps.storage, &from, &config.insurance_fund, spread_fee).unwrap();
        messages.push(msg);
    };

    if !toll_fee.is_zero() {
        let msg = execute_transfer_from(deps.storage, &from, &config.fee_pool, toll_fee).unwrap();
        messages.push(msg);
    };

    Ok(messages)
}

pub fn withdraw(
    deps: Deps,
    env: Env,
    state: &mut State,
    receiver: &Addr,
    insurance_fund: &Addr,
    eligible_collateral: AssetInfo,
    amount: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let token_balance =
        query_token_balance(deps, eligible_collateral, env.contract.address.clone())?;
    let mut messages: Vec<SubMsg> = vec![];

    let mut shortfall = Uint128::zero();

    if token_balance < amount {
        shortfall = amount.checked_sub(token_balance)?;

        messages.push(
            execute_transfer_from(
                deps.storage,
                insurance_fund,
                &env.contract.address,
                shortfall,
            )
            .unwrap(),
        );
    }
    messages.push(execute_transfer(deps.storage, receiver, amount).unwrap());

    // add any shortfall to bad_debt
    state.bad_debt += shortfall;

    Ok(messages)
}