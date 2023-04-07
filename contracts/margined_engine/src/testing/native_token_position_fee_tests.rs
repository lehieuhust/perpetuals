use cosmwasm_std::{BankMsg, Coin, CosmosMsg, Uint128};
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Position, Side};
use margined_utils::scenarios::NativeTokenScenario;
use margined_utils::tools::fund_calculator::calculate_funds_needed;

// Note: these tests also verify the 10% fees for the amm are functioning

#[test]
fn test_initialization() {
    let NativeTokenScenario {
        router,
        owner,
        alice,
        bob,
        engine,
        ..
    } = NativeTokenScenario::new();

    // verfiy the balances
    let owner_balance = router.wrap().query_balance(&owner, "orai").unwrap().amount;
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(5_000_000_000));
    let bob_balance = router.wrap().query_balance(&bob, "orai").unwrap().amount;
    assert_eq!(bob_balance, Uint128::new(5_000_000_000));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_force_error_open_position_no_token_sent() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();

    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_open_long_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(4_880_000_000));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(37_500_000u128));
    assert_eq!(position.margin, Uint128::from(60_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));
}

#[test]
fn test_force_error_insufficient_token_long_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            vec![Coin::new(119_000_000u128, "orai")],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_open_short_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(150_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(4_880_000_000));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_negative(150_000_000u128));
    assert_eq!(position.margin, Uint128::from(60_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));
}

#[test]
fn test_force_error_insufficient_token_short_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(150_000_000u64),
            vec![Coin::new(100_000_000u128, "orai")],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1, Uint128::new(4_950_000_000));

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(175_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(175_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(210_000_000u128)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(37_500_000u128));
    assert_eq!(position.margin, Uint128::from(200_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(200_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_up_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        fee_pool,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1, Uint128::new(4_950_000_000));

    // bob opens long position with 35 margin, 10x leverage, price up
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(137_878_787u64));

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1 - alice_balance_2, Uint128::new(240_000_000));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // transferred margin = margin + fee = 200 + (200 * 2 * 10%) = 240
    assert_eq!(position.size, Integer::new_positive(32_500_000u128));
    assert_eq!(position.margin, Uint128::from(225_000_000u64));
    assert_eq!(position.notional, Uint128::from(650_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::new(100_000_000));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(260_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_down_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 125 margin, 2x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens short position with 125 margin, 2x leverage, price down
    // (1250 - 250) * (80 + baseAssetDelta) = 100k, baseAssetDelta = 20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 20 long position worth 166.67 now
    // (1000 + quoteAssetDelta) * (100 + 20) = 100k, quoteAssetDelta = -166.666666666666666666
    // unrealizedPnl = positionValue - cost = 166.666666666666666666 - 250 = -83.333333333333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(83_333_334u64));

    // alice opens long position with 50 margin, 5x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(5_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(75_000_000u128)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // transferred margin = margin + fee = 50 + (50 * 5 * 10%) = 75
    assert_eq!(position.size, Integer::new_positive(40_000_000u128));
    assert_eq!(position.margin, Uint128::from(175_000_000u64));
    assert_eq!(position.notional, Uint128::from(500_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // alice opens short position with 50 margin, 8x leverage
    // (800 - 400) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(125_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(8_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(90_000_000u128)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    // then transferred margin = margin + fee = 50 + (50 * 8 * 10%) = 90
    assert_eq!(position.size, Integer::new_negative(150_000_000u128));
    assert_eq!(position.margin, Uint128::from(150_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_short_position_price_down_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens short position with 150 margin, 2x leverage, price down
    // (800 - 300) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 75
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(150_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(75_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(150_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 71.43 now
    // (500 + quoteAssetDelta) * (200 - 25) = 100k, quoteAssetDelta = -71.4285714286
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 71.4285714286 = 128.5714285714
    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(128_571_428u64));

    // alice opens short position with 100 margin, 3x leverage
    // (500 - 300) * (200 + baseAssetDelta) = 100k, baseAssetDelta = 300
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(3_000_000u64),
            Uint128::from(300_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(3_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // transferred margin = margin + fee = 100 + (100 * 3 * 10%) = 130
    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(130_000_000u128)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    assert_eq!(position.size, Integer::new_negative(325_000_000u128));
    assert_eq!(position.margin, Uint128::from(200_000_000u64));
    assert_eq!(position.notional, Uint128::from(500_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_up_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 200 margin, 1x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens long position with 200 margin, 1x leverage, price up
    // (800 + 200) * (125 + baseAssetDelta) = 100k, baseAssetDelta = -25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 333.33 now
    // (1000 + quoteAssetDelta) * (100 - 25) = 100k, quoteAssetDelta = 333.3333333333
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 333.3333333333 = -133.3333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(133_333_334u64));

    // alice opens short position with 50 margin, 4x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(4_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(4_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // then transferred margin = margin + fee = 50 + (50 * 4 * 10%) = 70
    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(70_000_000u128)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    assert_eq!(position.size, Integer::new_negative(50_000_000u128));
    assert_eq!(position.margin, Uint128::from(250_000_000u64));
    assert_eq!(position.notional, Uint128::from(400_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_reduce_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(20_000_000u128));
    assert_eq!(position.notional, Uint128::from(250_000_000u64));
    assert_eq!(position.margin, Uint128::from(60_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_reduce_long_position_zero_fee() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = vamm.set_toll_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(20_000_000u128));
    assert_eq!(position.notional, Uint128::from(250_000_000u64));
    assert_eq!(position.margin, Uint128::from(60_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_reduce_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(150_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(125_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(25_000_000u128));
    assert_eq!(position.notional, Uint128::from(200_000_000u64));
    assert_eq!(position.margin, Uint128::from(60_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_reduce_long_position_price_up_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(257_142_857u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(25_000_000u128));
    assert_eq!(position.margin, Uint128::from(145_714_285u64));
    assert_eq!(position.notional, Uint128::from(285_714_285u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(171_428_572u64));
}

#[test]
fn test_ten_percent_fee_reduce_long_position_price_down_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(500_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(500_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(288_888_889u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(32_500_000u128));
    assert_eq!(position.notional, Uint128::from(548_888_889u64));
    assert_eq!(position.margin, Uint128::from(398_888_889u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(187_777_778u64));
}

#[test]
fn test_ten_percent_fee_reduce_short_position_price_up_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(7_350_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(29_365_079u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(150_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(17_640_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(150_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(7_352_942u128));
    assert_eq!(position.notional, Uint128::from(70_728_290u64));
    assert_eq!(position.margin, Uint128::from(79_271_710u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8_636_799u64));
}

#[test]
fn test_ten_percent_fee_reduce_short_position_price_down_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(250_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(233_333_333u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(50_000_000u128));
    assert_eq!(position.notional, Uint128::from(283_333_334u64));
    assert_eq!(position.margin, Uint128::from(366_666_666u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(116_666_667u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_remains_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(5_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(250_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(25_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_short_price_remains_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(80_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_long_price_up_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        bank,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // give engine some funds so it has enough collateral to pay profit
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: engine.addr().to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    router.execute(bank.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(137_878_787u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(124_090_909u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_long_price_down_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(500_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(500_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(288_888_889u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(140_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_short_price_up_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(7_350_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(29_365_079u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(147_698_414u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_short_price_down_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        bank,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // give engine some funds so it has enough collateral to pay profit
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: engine.addr().to_string(),
        amount: vec![Coin::new(1000u128 * 10u128.pow(6), "orai")],
    });
    router.execute(bank.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(250_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(233_333_333u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(456_666_667u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.notional, Uint128::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_ten_percent_fee_open_long_price_remains_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(45_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(45_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(45_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(60_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(25_000_000u64));
    assert_eq!(position.notional, Uint128::from(200_000_000u64));
    assert_eq!(position.margin, Uint128::from(20_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_remains_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(20_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(90_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(45_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(90_000_000u64),
                Uint128::from(5_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(75_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(20_000_000u64));
    assert_eq!(position.notional, Uint128::from(250_000_000u64));
    assert_eq!(position.margin, Uint128::from(50_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_open_long_price_up_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        fee_pool,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(137_878_787u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(62_510_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(8_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(31_363_636u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(42_500_001u64));
    assert_eq!(position.notional, Uint128::from(412_121_213u64));
    assert_eq!(position.margin, Uint128::from(51_515_151u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(9u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(140_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(83_333_334u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(1_450_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(61_666_667u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(130_000_001u64));
    assert_eq!(position.notional, Uint128::from(433_333_334u64));
    assert_eq!(position.margin, Uint128::from(43_333_333u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(3u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_up_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(4_000_000u64),
            Uint128::from(7_349_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(4_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(133_333_334u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(37_490_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(12_499_999u64));
    assert_eq!(position.notional, Uint128::from(266_666_666u64));
    assert_eq!(position.margin, Uint128::from(26_666_666u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(21u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_down_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(500_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(500_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(233_333_333u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(149_990_000u64),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_2 - alice_balance_1,
        Uint128::from(640_000_000u64)
    );

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(49_999_999u64));
    assert_eq!(position.notional, Uint128::from(333_333_333u64));
    assert_eq!(position.margin, Uint128::from(33_333_333u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_liquidation() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(5_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            calculate_funds_needed(
                &router,
                engine.addr(),
                alice.clone(),
                Uint128::from(5_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Buy,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            calculate_funds_needed(
                &router,
                engine.addr(),
                bob.clone(),
                Uint128::from(50_000_000u64),
                Uint128::from(10_000_000u64),
                Side::Sell,
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(35_962_880u64));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(4_761_904u64));
    assert_eq!(position.notional, Uint128::from(50_000_000u64));
    assert_eq!(position.margin, Uint128::from(5_000_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::zero(),
            vec![Coin::new(82_925_760u128, "orai")],
        )
        .unwrap();

    assert_eq!(
        calculate_funds_needed(
            &router,
            engine.addr(),
            alice.clone(),
            Uint128::from(60_000_000u64),
            Uint128::from(1_000_000u64),
            Side::Sell,
            vamm.addr(),
        )
        .unwrap(),
        vec![Coin::new(82_925_760u128, "orai")]
    );

    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(17_501_548u64));
    assert_eq!(position.notional, Uint128::from(45_962_880u64));
    assert_eq!(position.margin, Uint128::from(45_962_880u64));
}

// TODO: Find these tests
// #[test]
// fn test_ten_percent_fee_open_long_price_down_liquidation_with_positive_margin() {
//     let NativeTokenScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         engine,
//         vamm,
//         ..
//     } = NativeTokenScenario::new();
//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(10_000_000u64),
//             Uint128::from(10_000_000u64),
//             Uint128::zero(),
//             calculate_funds_needed(
//                 &router,
//                 engine.addr(),
//                 alice.clone(),
//                 Uint128::from(10_000_000u64),
//                 Uint128::from(10_000_000u64),
//                 Side::Buy,
//                 vamm.addr(),
//             )
//             .unwrap(),
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(10_000_000u64),
//             Uint128::from(5_000_000u64),
//             Uint128::zero(),
//             calculate_funds_needed(
//                 &router,
//                 engine.addr(),
//                 bob.clone(),
//                 Uint128::from(10_000_000u64),
//                 Uint128::from(5_000_000u64),
//                 Side::Sell,
//                 vamm.addr(),
//             )
//             .unwrap(),
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router,
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8_506_226u64));

//     let position: Position = engine
//         .position(&router, vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_positive(9_090_909u64));
//     assert_eq!(position.notional, Uint128::from(100_000_000u64));
//     assert_eq!(position.margin, Uint128::from(10_000_000u64));

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(60_000_000u64),
//             Uint128::from(1_000_000u64),
//             Uint128::zero(),
//             vec![Coin::new(6_000_000u128, "orai")],
//         )
//         .unwrap();

//     assert_eq!(
//         calculate_funds_needed(
//             &router,
//             engine.addr(),
//             alice.clone(),
//             Uint128::from(60_000_000u64),
//             Uint128::from(1_000_000u64),
//             Side::Sell,
//             vamm.addr(),
//         )
//         .unwrap(),
//         vec![Coin::new(6_000_000u128, "orai")]
//     );

//     router.execute(alice.clone(), msg).unwrap();

//     let position: Position = engine
//         .position(&router, vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_positive(3_318_903u64));
//     assert_eq!(position.notional, Uint128::from(34_599_222u64));
//     assert_eq!(position.margin, Uint128::from(4_599_222u64));
// }
