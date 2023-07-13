use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_utils::{cw_multi_test::Executor, testing::SimpleScenario};

use crate::testing::new_simple_scenario;

// Note: these tests also verify the 10% fees for the amm are functioning
#[test]
fn test_ten_percent_fee_open_long_position() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(37_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(4_880_000_000_000u128));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(position.margin, Uint128::from(60_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000_000u64));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000_000u64));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(60_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_short_position() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 60 margin, 10x leverage
    // (1000 - 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 150
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(150_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(4_880_000_000_000u128));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_negative(150_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(60_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(600_000_000_000u64));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000_000u64));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(60_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_long_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(20_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance_1, Uint128::from(4_950_000_000_000u128));

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(175_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(210_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(20_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(25_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(250_000_000_000u64));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000_000u64));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(200_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_up_long_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(20_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance_1, Uint128::from(4_950_000_000_000u128));

    // bob opens long position with 35 margin, 10x leverage, price up
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_positive(137_878_787_878u64)
    );

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(12_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(240_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 200 + (200 * 2 * 10%) = 240
    assert_eq!(position.size, Integer::new_positive(20_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(25_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(250_000_000_000u64));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(100_000_000_000u64));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(260_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_down_long_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 125 margin, 2x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(20_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    // bob opens short position with 125 margin, 2x leverage, price down
    // (1250 - 250) * (80 + baseAssetDelta) = 100k, baseAssetDelta = 20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(125_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(20_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 20 long position worth 166.67 now
    // (1000 + quoteAssetDelta) * (100 + 20) = 100k, quoteAssetDelta = -166.666666666666666666
    // unrealizedPnl = positionValue - cost = 166.666666666666666666 - 250 = -83.333333333333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(83_333_333_334u64));

    // alice opens long position with 50 margin, 5x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000_000u64),
            Uint128::from(5_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(20_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(75_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 50 + (50 * 5 * 10%) = 75
    assert_eq!(position.size, Integer::new_positive(20_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(125_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(250_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_short_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    // alice opens short position with 50 margin, 8x leverage
    // (800 - 400) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000_000u64),
            Uint128::from(8_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(125_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(90_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // then transferred margin = margin + fee = 50 + (50 * 8 * 10%) = 90
    assert_eq!(position.size, Integer::new_negative(25_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(100_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(200_000_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(155555555555u128));
}

#[test]
fn test_ten_percent_fee_short_position_price_down_short_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    // bob opens short position with 150 margin, 2x leverage, price down
    // (800 - 300) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 75
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(150_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(75_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 71.43 now
    // (500 + quoteAssetDelta) * (200 - 25) = 100k, quoteAssetDelta = -71.4285714286
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 71.4285714286 = 128.5714285714
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_positive(128_571_428_571u64)
    );

    // alice opens short position with 100 margin, 3x leverage
    // (500 - 300) * (200 + baseAssetDelta) = 100k, baseAssetDelta = 300
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000_000u64),
            Uint128::from(3_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(300_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // transferred margin = margin + fee = 100 + (100 * 3 * 10%) = 130
    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(130_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    assert_eq!(position.size, Integer::new_negative(25_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(100_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(200_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_up_short_again() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 200 margin, 1x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    // bob opens long position with 200 margin, 1x leverage, price up
    // (800 + 200) * (125 + baseAssetDelta) = 100k, baseAssetDelta = -25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 333.33 now
    // (1000 + quoteAssetDelta) * (100 - 25) = 100k, quoteAssetDelta = 333.3333333333
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 333.3333333333 = -133.3333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_negative(133_333_333_334u64)
    );

    // alice opens short position with 50 margin, 4x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000_000u64),
            Uint128::from(4_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // then transferred margin = margin + fee = 50 + (50 * 4 * 10%) = 70
    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(70_000_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    assert_eq!(position.size, Integer::new_negative(25_000_000_000u128));
    assert_eq!(position.margin, Uint128::from(200_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(200_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_reduce_position() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(37_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(position_1.notional, Uint128::from(600_000_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(60_000_000_000u64));

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_2.size, Integer::new_negative(17_500_000_000u128));
    assert_eq!(position_2.notional, Uint128::from(350_000_000_000u64));
    assert_eq!(position_2.margin, Uint128::from(350_000_000_000u64));

    let pnl_1 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_1.unrealized_pnl, Integer::new_negative(201_063_829_788u128));


    let pnl_2 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_2.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_reduce_long_position_zero_fee() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm.set_toll_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(37_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(position_1.notional, Uint128::from(600_000_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(60_000_000_000u64));

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_2.size, Integer::new_negative(17_500_000_000u128));
    assert_eq!(position_2.notional, Uint128::from(350_000_000_000u64));
    assert_eq!(position_2.margin, Uint128::from(350_000_000_000u64));

    let pnl_1 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_1.unrealized_pnl, Integer::new_negative(201_063_829_788u128));

    let pnl_2 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_2.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_reduce_position() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(150_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(400_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(125_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_negative(150_000_000_000u128));
    assert_eq!(position_1.notional, Uint128::from(600_000_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(60_000_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    // assert_eq!(pnl.unrealized_pnl, Integer::zero());
}

// #[test]
// fn test_ten_percent_fee_reduce_long_position_price_up_long_again() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(60_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(37_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(400_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(12_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(257_142_857_142u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(400_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(12_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_positive(25_000_000_000u128));
//     assert_eq!(position.margin, Uint128::from(145_714_285_714u64));
//     assert_eq!(position.notional, Uint128::from(285_714_285_714u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(171_428_571_428u64)
//     );
// }

// #[test]
// fn test_ten_percent_fee_reduce_long_position_price_down_long_again() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(500_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(400_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(12_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_negative(288_888_888_889u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(350_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(17_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_positive(32_500_000_000u128));
//     assert_eq!(position.notional, Uint128::from(548_888_888_889u64));
//     assert_eq!(position.margin, Uint128::from(398_888_888_889u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_negative(187_777_777_778u64)
//     );
// }

// #[test]
// fn test_ten_percent_fee_reduce_short_position_price_up_short_again() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(50_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(7_350_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(29_365_079_364u64));

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(150_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(17_640_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_negative(7_352_941_177u128));
//     assert_eq!(position.notional, Uint128::from(70_728_291_315u64));
//     assert_eq!(position.margin, Uint128::from(79_271_708_685u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8_636_788_056u64));
// }

// #[test]
// fn test_ten_percent_fee_reduce_short_position_price_down_short_again() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(250_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(100_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(233_333_333_333u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_negative(50_000_000_000u128));
//     assert_eq!(position.notional, Uint128::from(283_333_333_334u64));
//     assert_eq!(position.margin, Uint128::from(366_666_666_666u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(116_666_666_667u64)
//     );
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_remains_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(50_000_000_000u64),
//             Uint128::from(5_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(250_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(25_000_000_000u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_short_price_remains_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(200_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(80_000_000_000u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_up_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     // give engine some funds so it has enough collateral to pay profit
//     router
//         .execute_contract(
//             owner.clone(),
//             usdc.addr().clone(),
//             &Cw20ExecuteMsg::Mint {
//                 recipient: engine.addr().to_string(),
//                 amount: Uint128::from(1_000_000_000_000u64),
//             },
//             &[],
//         )
//         .unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(25_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(35_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(17_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(137_878_787_878u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             pnl.position_notional,
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(124_090_909_091u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_down_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(500_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(400_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(12_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_negative(288_888_888_889u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             pnl.position_notional,
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(140_000_000_000u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_short_price_up_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(200_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(50_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(7_350_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(29_365_079_364u64));

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             pnl.position_notional,
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(147_698_412_700u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_short_price_down_close_manually() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     // given some other traders open some amount of position
//     // to prevent vault doesn't have enough collateral to pay profit in this test case
//     router
//         .execute_contract(
//             owner.clone(),
//             usdc.addr().clone(),
//             &Cw20ExecuteMsg::Mint {
//                 recipient: engine.addr().to_string(),
//                 amount: Uint128::from(1_000_000_000_000u64),
//             },
//             &[],
//         )
//         .unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(250_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(100_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(50_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(233_333_333_333u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             pnl.position_notional,
//             Uint128::from(1_000_000_000u64),
//             Uint128::from(100_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(456_666_666_667u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::zero());
//     assert_eq!(position.notional, Uint128::zero());
//     assert_eq!(position.margin, Uint128::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_remains_close_opening_larger_short() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(125_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(45_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(45_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(60_000_000_000u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_negative(25_000_000_000u64));
//     assert_eq!(position.notional, Uint128::from(200_000_000_000u64));
//     assert_eq!(position.margin, Uint128::from(20_000_000_000u64));
// }

// #[test]
// fn test_ten_percent_fee_open_short_price_remains_close_opening_larger_long() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(20_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(25_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(90_000_000_000u64),
//             Uint128::from(5_000_000_000u64),
//             Uint128::from(45_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_1 - alice_balance_2,
//         Uint128::from(75_000_000_000u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_positive(20_000_000_000u64));
//     assert_eq!(position.notional, Uint128::from(250_000_000_000u64));
//     assert_eq!(position.margin, Uint128::from(50_000_000_000u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::zero());
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_up_close_opening_larger_short() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         fee_pool,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(25_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(35_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(17_500_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(
//         pnl.unrealized_pnl,
//         Integer::new_positive(137_878_787_878u64)
//     );

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(100_000_000_000u64),
//             Uint128::from(8_000_000_000u64),
//             Uint128::from(62_510_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_2 - alice_balance_1,
//         Uint128::from(31_363_636_363u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_negative(42_500_000_001u64));
//     assert_eq!(position.notional, Uint128::from(412_121_212_122u64));
//     assert_eq!(position.margin, Uint128::from(51_515_151_515u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(9u64));

//     let fee_pool_balance = usdc
//         .balance(&router.wrap(), fee_pool.addr().clone())
//         .unwrap();
//     assert_eq!(fee_pool_balance, Uint128::from(140_000_000_000u64));
// }

// #[test]
// fn test_ten_percent_fee_open_long_price_down_close_opening_larger_short() {
//     let SimpleScenario {
//         mut router,
//         owner,
//         alice,
//         bob,
//         usdc,
//         engine,
//         vamm,
//         ..
//     } = new_simple_scenario();

//     // 10% fee
//     let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
//     router.execute(owner.clone(), msg).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Buy,
//             Uint128::from(125_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(125_000_000_000u64),
//             Uint128::from(2_000_000_000u64),
//             Uint128::from(20_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(bob.clone(), msg).unwrap();

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(83_333_333_334u64));

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::Sell,
//             Uint128::from(60_000_000_000u64),
//             Uint128::from(10_000_000_000u64),
//             Uint128::from(1_450_000_000_000u64),
//             vec![],
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
//     assert_eq!(
//         alice_balance_1 - alice_balance_2,
//         Uint128::from(61_666_666_667u64)
//     );

//     let position = engine
//         .position(&router.wrap(), vamm.addr().to_string(), alice.to_string())
//         .unwrap();
//     assert_eq!(position.size, Integer::new_negative(130_000_000_001u64));
//     assert_eq!(position.notional, Uint128::from(433_333_333_334u64));
//     assert_eq!(position.margin, Uint128::from(43_333_333_333u64));

//     let pnl = engine
//         .get_unrealized_pnl(
//             &router.wrap(),
//             vamm.addr().to_string(),
//             alice.to_string(),
//             PnlCalcOption::SpotPrice,
//         )
//         .unwrap();
//     assert_eq!(pnl.unrealized_pnl, Integer::new_negative(3u64));
// }

#[test]
fn test_ten_percent_fee_open_short_price_up_close_opening_larger_long() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(25_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000_000u64),
            Uint128::from(4_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(7_349_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_negative(133_333_333_334u64)
    );

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(37_490_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(25_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(200_000_000_000u64));
    assert_eq!(position.margin, Uint128::from(200_000_000_000u64));

    let pnl_1 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_1.unrealized_pnl, Integer::new_negative(866666666667u64));

    let pnl_1 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_1.unrealized_pnl, Integer::new_negative(866666666667u64));

    let pnl_2 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            3,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_2.unrealized_pnl, Integer::zero());
}

#[test]
fn test_ten_percent_fee_open_short_price_down_close_opening_larger_long() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(500_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(100_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(50_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_positive(233_333_333_333u64)
    );

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::from(149_990_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(100_000_000_000u64));
    assert_eq!(position.notional, Uint128::from(500_000_000_000u64));
    assert_eq!(position.margin, Uint128::from(500_000_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_liquidation() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(5_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(35_962_877_033u64));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(4_761_904_761u64));
    assert_eq!(position.notional, Uint128::from(50_000_000_000u64));
    assert_eq!(position.margin, Uint128::from(5_000_000_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();

    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(22263450836u64));
    assert_eq!(position.notional, Uint128::from(60000000000u64));
    assert_eq!(position.margin, Uint128::from(60000000000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_liquidation_with_positive_margin() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(10_000_000_000u64),
            Uint128::from(10_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(10_000_000_000u64),
            Uint128::from(5_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8_506_224_077u64));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(9_090_909_090u64));
    assert_eq!(position.notional, Uint128::from(100_000_000_000u64));
    assert_eq!(position.margin, Uint128::from(10_000_000_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Uint128::zero(),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();

    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(5_148_005_148u64));
    assert_eq!(position.notional, Uint128::from(60_000_000_000u64));
    assert_eq!(position.margin, Uint128::from(60_000_000_000u64));
}
