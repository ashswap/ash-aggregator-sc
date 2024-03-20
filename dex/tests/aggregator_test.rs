#![allow(deprecated)]
use aggregator::*;
use fee::*;
use common_errors::*;
use common_structs::*;
use multiversx_sc::{codec::multi_types::OptionalValue, require, types::*};
use multiversx_sc_scenario::{multiversx_chain_vm::tx_mock::TxContextRef, testing_framework::*, *};
use wrapper_mock::EgldWrapperMock;
pub mod protocol_mock;

type RustBigUint = num_bigint::BigUint;

#[derive(Clone, Debug)]
struct TestTokenAmount {
    pub token: Vec<u8>,
    pub amount: RustBigUint,
}

#[derive(Clone, Debug)]
struct TestAggregatorStep {
    pub token_in: Vec<u8>,
    pub token_out: Vec<u8>,
    pub amount_in: RustBigUint,
    pub pool_address: Address,
}

const AGGREGATOR_WASM_PATH: &'static str = "aggregator/output/aggregator.wasm";
const WRAPPER_MOCK_WASM_PATH: &'static str = "wrapper-mock/output/wrapper-mock.wasm";
const FEE_WASM_PATH: &'static str = "fee/output/fee.wasm";

const USDC_TOKEN_ID: &[u8] = b"USDC-abcdef";
const USDT_TOKEN_ID: &[u8] = b"USDT-abcdef";
const BUSD_TOKEN_ID: &[u8] = b"BUSD-abcdef";
const USER_TOTAL_TOKENS: u64 = 5_000_000_000;
const WRAPPED_EGLD_TOKEN_ID: &[u8] = b"WEGLD-abcdef";

struct AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
    FeeObjBuilder: 'static + Copy + Fn() -> fee::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub user_address: Address,
    pub mock_wrapper: ContractObjWrapper<protocol_mock::ContractObj<DebugApi>, ProtocolObjBuilder>,
    pub wrapper_wrapper: ContractObjWrapper<wrapper_mock::ContractObj<DebugApi>, WrapperObjBuilder>,
    pub agg_wrapper: ContractObjWrapper<aggregator::ContractObj<DebugApi>, AggregatorObjBuilder>,
    pub fee_wrapper: ContractObjWrapper<fee::ContractObj<DebugApi>, FeeObjBuilder>,
}

fn set_esdt_balance(blockchain_wrapper: &mut BlockchainStateWrapper, address: &Address) {
    for token in vec![
        USDC_TOKEN_ID,
        USDT_TOKEN_ID,
        BUSD_TOKEN_ID,
        WRAPPED_EGLD_TOKEN_ID,
    ] {
        blockchain_wrapper.set_esdt_balance(address, token, &rust_biguint!(USER_TOTAL_TOKENS));
    }
}

fn setup_aggregator<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>(
    mock_builder: ProtocolObjBuilder,
    wrapper_mock_builder: WrapperObjBuilder,
    agg_builder: AggregatorObjBuilder,
    fee_builder: FeeObjBuilder,
) -> AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
    FeeObjBuilder: 'static + Copy + Fn() -> fee::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_addr = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address =
        blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));

    let mock_wrapper = blockchain_wrapper.create_sc_account(
        &rust_biguint!(1_000_000_000_000_000_000),
        Some(&owner_addr),
        mock_builder,
        AGGREGATOR_WASM_PATH,
    );

    let mock_wrapper_wrapper = blockchain_wrapper.create_sc_account(
        &parse_biguint("1_000_000_000_000_000_000_000"),
        Some(&owner_addr),
        wrapper_mock_builder,
        WRAPPER_MOCK_WASM_PATH,
    );
    blockchain_wrapper.set_esdt_balance(
        &mock_wrapper_wrapper.address_ref().clone(),
        WRAPPED_EGLD_TOKEN_ID,
        &parse_biguint("1_000_000_000_000_000_000_000"),
    );

    let agg_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        agg_builder,
        AGGREGATOR_WASM_PATH,
    );

    let fee_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        fee_builder,
        FEE_WASM_PATH,
    );

    blockchain_wrapper
        .execute_tx(&owner_addr, &mock_wrapper_wrapper, &rust_zero, |sc| {
            sc.init(WRAPPED_EGLD_TOKEN_ID.into());
        })
        .assert_ok();

    blockchain_wrapper
        .execute_tx(&owner_addr, &agg_wrapper, &rust_zero, |sc| {
            sc.init(managed_address!(&fee_wrapper.address_ref().clone()));
        })
        .assert_ok();

    set_esdt_balance(&mut blockchain_wrapper, &user_address);
    set_esdt_balance(&mut blockchain_wrapper, &mock_wrapper.address_ref());

    AggregatorSetup {
        blockchain_wrapper,
        user_address,
        mock_wrapper,
        wrapper_wrapper: mock_wrapper_wrapper,
        agg_wrapper,
        fee_wrapper
    }
}

fn to_managed_biguint(value: RustBigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}

fn parse_biguint(str: &str) -> RustBigUint {
    let str_without_underscores = str.to_owned().replace("_", "");
    RustBigUint::parse_bytes(str_without_underscores.as_bytes(), 10).unwrap()
}

fn check_result<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>,
    expected_balances: Vec<TestTokenAmount>,
) where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
    FeeObjBuilder: 'static + Copy + Fn() -> fee::ContractObj<DebugApi>,
{
    for expected in expected_balances {
        agg_setup.blockchain_wrapper.check_esdt_balance(
            &agg_setup.user_address,
            &expected.token,
            &expected.amount,
        );
    }
}

fn check_result_egld<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>,
    expected_balance: RustBigUint,
) where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
    FeeObjBuilder: 'static + Copy + Fn() -> fee::ContractObj<DebugApi>,
{
    agg_setup
        .blockchain_wrapper
        .check_egld_balance(&agg_setup.user_address, &expected_balance);
}

fn aggregate<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder, FeeObjBuilder>,
    token_in: &[u8],
    token_out: &[u8],
    test_steps: Vec<TestAggregatorStep>,
    limit: RustBigUint,
    protocol: Option<&Address>,
    payment: TxTokenTransfer,
) -> TxResult
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
    FeeObjBuilder: 'static + Copy + Fn() -> fee::ContractObj<DebugApi>,
{
    if payment.token_identifier == b"EGLD" {
        agg_setup.blockchain_wrapper.execute_tx(
            &agg_setup.user_address, 
            &agg_setup.agg_wrapper, 
            &payment.value, 
            |sc| {
                let mut steps = ManagedVec::new();
                for step in test_steps {
                    let token_in;
                    if step.token_in == b"EGLD" {
                        token_in = managed_egld_token_id!();
                    } else {
                        token_in = managed_token_id_wrapped!(step.token_in);
                    }
                    let token_out;
                    if step.token_out == b"EGLD" {
                        token_out = managed_egld_token_id!();
                    } else {
                        token_out = managed_token_id_wrapped!(step.token_out.clone());
                    }
                    let arguments = vec![managed_buffer!(&step.token_out)];
                    steps.push(AggregatorStep {
                        token_in: token_in,
                        token_out:token_out,
                        amount_in: to_managed_biguint(step.amount_in),
                        pool_address: managed_address!(&step.pool_address),
                        function_name: managed_buffer!(b"exchange"),
                        arguments: ManagedVec::from(arguments),
                    });
                }
                if protocol.is_none() {
                        sc.aggregate(
                        managed_egld_token_id!(), 
                        managed_token_id_wrapped!(token_out),
                        to_managed_biguint(limit),
                        steps,
                        OptionalValue::None,
                    );
                } else {
                    sc.aggregate(
                        managed_egld_token_id!(), 
                        managed_token_id_wrapped!(token_out),
                        to_managed_biguint(limit),
                        steps,
                        OptionalValue::Some(managed_address!(protocol.unwrap())),
                    );
                }
            },
        )
    } else {
        agg_setup.blockchain_wrapper.execute_esdt_transfer(
            &agg_setup.user_address,
            &agg_setup.agg_wrapper,
            &payment.token_identifier,
            0,
            &payment.value,
            |sc| {
                let final_token_out;
                if token_out == b"EGLD" {
                    final_token_out = managed_egld_token_id!();
                } else {
                    final_token_out = managed_token_id_wrapped!(token_out);
                }
                let mut steps = ManagedVec::new();
                for step in test_steps {
                    let arguments = vec![managed_buffer!(&step.token_out)];
                    let token_in;
                    if step.token_in == b"EGLD" {
                        token_in = managed_egld_token_id!();
                    } else {
                        token_in = managed_token_id_wrapped!(step.token_in);
                    }
                    let token_out;
                    if step.token_out == b"EGLD" {
                        token_out = managed_egld_token_id!();
                    } else {
                        token_out = managed_token_id_wrapped!(step.token_out);
                    }
                    steps.push(AggregatorStep {
                        token_in: token_in,
                        token_out: token_out,
                        amount_in: to_managed_biguint(step.amount_in),
                        pool_address: managed_address!(&step.pool_address),
                        function_name: managed_buffer!(b"exchange"),
                        arguments: ManagedVec::from(arguments),
                    });
                }
                if protocol.is_none() {
                    sc.aggregate(
                        managed_token_id_wrapped!(token_in),
                        final_token_out,
                        to_managed_biguint(limit),
                        steps,
                        OptionalValue::None,
                    );
                } else {
                    sc.aggregate(
                        managed_token_id_wrapped!(token_in),
                        final_token_out,
                        to_managed_biguint(limit),
                        steps,
                        OptionalValue::Some(managed_address!(protocol.unwrap())),
                    );
                }
            },
        )
    }
}

#[test]
fn test_aggregate_simple() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS - amount),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 95 / 100),
        },
    ];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
}

#[test]
fn test_aggregate_simple_egld() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: b"EGLD".to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 95 / 100),
        },
    ];

    aggregate(
        &mut agg_setup,
        b"EGLD",
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: b"EGLD".to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
    check_result_egld(&mut agg_setup, rust_biguint!(1_000_000_000_000_000_000 - amount));
}

#[test]
fn test_aggregate_simple_return_egld() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: BUSD_TOKEN_ID.to_vec(),
        token_out: b"EGLD".to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS - amount),
        },
    ];

    aggregate(
        &mut agg_setup,
        BUSD_TOKEN_ID,
        b"EGLD",
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: BUSD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
    check_result_egld(&mut agg_setup, rust_biguint!(1_000_000_000_000_000_000 + amount * 95 / 100));
}

#[test]
fn test_aggregate_simple_egld_in_step() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: BUSD_TOKEN_ID.to_vec(),
        token_out: b"EGLD".to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: b"EGLD".to_vec(),
        token_out: USDC_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS - amount),
        },
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 95 / 100 * 95 / 100),
        },
    ];

    aggregate(
        &mut agg_setup,
        BUSD_TOKEN_ID,
        USDC_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: BUSD_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
    check_result_egld(&mut agg_setup, rust_biguint!(1_000_000_000_000_000_000));
}

#[test]
fn test_aggregate_two_routes() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDT_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS - 2 * amount),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 95 / 100 + amount * 95 / 100 * 95 / 100),
        },
    ];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(2 * amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
}

#[test]
fn test_aggregate_error_invalid_token_in() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid token in
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDT_TOKEN_ID.to_vec(), // change it
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(
        &mut agg_setup, 
        USDC_TOKEN_ID, 
        BUSD_TOKEN_ID, 
        test_steps, 
        rust_biguint!(0), 
        Option::None, 
        payment
    ).assert_user_error(ERROR_INVALID_TOKEN_IN);
}

#[test]
fn test_aggregate_error_same_token() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid token in
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(), // change it
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(
        &mut agg_setup, 
        USDC_TOKEN_ID, 
        USDC_TOKEN_ID, 
        test_steps, 
        rust_biguint!(0), 
        Option::None, 
        payment
    ).assert_user_error(ERROR_SAME_TOKEN);
}

#[test]
fn test_aggregate_error_same_token_in_step() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDC_TOKEN_ID.to_vec(), // invalid same token
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(
        &mut agg_setup, 
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID, 
        test_steps, 
        rust_biguint!(0), 
        Option::None, 
        payment
    ).assert_user_error(ERROR_SAME_TOKEN);
}

#[test]
fn test_aggregate_error_invalid_address() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let agg_address: Address = agg_setup.agg_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid token in
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: agg_address.clone(), //same addr as aggregator
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(
        &mut agg_setup, 
        USDC_TOKEN_ID, 
        BUSD_TOKEN_ID, 
        test_steps, 
        rust_biguint!(0), 
        Option::None, 
        payment
    ).assert_user_error(ERROR_INVALID_POOL_ADDR);
}

#[test]
fn test_aggregate_error_two_routes_invalid_token_out() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(), // invalid token out
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDT_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(2 * amount),
        }
    ).assert_user_error(ERROR_INVALID_TOKEN_OUT);
}

#[test]
fn test_aggregate_error_two_routes_invalid_token_out_2() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: BUSD_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(), // invalid token out
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(2 * amount),
        }
    ).assert_user_error(ERROR_INVALID_TOKEN_OUT);
}

#[test]
fn test_aggregate_error_two_routes_invalid_token_in_step() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(), // invalid token in step
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(2 * amount),
        }
    ).assert_user_error(ERROR_INVALID_TOKEN_IN);
}

#[test]
fn test_aggregate_error_two_routes_invalid_amount_in_step() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount - 1), // invalid amount in
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: USDT_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }, TestAggregatorStep {
        token_in: USDT_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(0),
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(2 * amount),
        }
    ).assert_user_error(ERROR_INVALID_STEPS);
}

#[test]
fn test_aggregate_error_invalid_amount_in() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid amount in
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount + 1), // change it
        pool_address: mock_address.clone(),
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(&mut agg_setup, 
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        payment
    )
    .assert_user_error(ERROR_INVALID_AMOUNT_IN);
}

#[test]
fn test_aggregate_error_slippage_screw_you() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;
    // slippage
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(&mut agg_setup, 
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(amount), 
        Option::None,
        payment
    ).assert_user_error(ERROR_SLIPPAGE_SCREW_YOU);
}

#[test]
fn test_aggregate_error_invalid_amount_in_step() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid amount in
    let test_steps = vec![
        TestAggregatorStep {
            token_in: USDC_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount), // change it
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: BUSD_TOKEN_ID.to_vec(),
            token_out: USDT_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount * 95 / 100), // change it
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: USDT_TOKEN_ID.to_vec(),
            token_out: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount), // change it
            pool_address: mock_address.clone(),
        },
    ];

    let payment = TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    };

    aggregate(&mut agg_setup, 
        USDC_TOKEN_ID,
        WRAPPED_EGLD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::None,
        payment
    ).assert_user_error(ERROR_INVALID_TOKEN_IN);
}

#[test]
fn test_fee_simple(){
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let protocol_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    let ashswap_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.register_ashswap_fee(50_000u64, managed_address!(&ashswap_address)); // 50%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_address)); // 10%
            let ashswap_percent = sc.ashswap_fee_percent().get();
            assert_eq!(ashswap_percent, 50_000u64);
            let protocol_percent = sc.protocol_fee_percent(managed_address!(&protocol_address)).get();
            assert_eq!(protocol_percent, 10_000u64);
        }
    ).assert_ok();
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];

    let expected_balances = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS - amount),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 9 / 10 * 95 / 100),
        },
    ];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    check_result(&mut agg_setup, expected_balances);
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10),
    );
    let expected_protocol_fee = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10 / 2),
        },
    ];
    let expected_ashswap_fee = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10 / 2),
        },
    ];
    agg_setup.blockchain_wrapper.execute_query(
        &agg_setup.fee_wrapper, 
        |sc| {
            let protocol_fee = sc.get_claimable_protocol_fee(managed_address!(&protocol_address), 0u64, 100u64);
            let mut i = 0;
            for fee in protocol_fee.into_iter() {
                assert_eq!(fee.amount, to_managed_biguint(expected_protocol_fee[i].amount.clone()));
                i += 1;
            }
            i = 0;
            let ashswap_fee = sc.get_claimable_ashswap_fee(0u64, 100u64);
            for fee in ashswap_fee.into_iter() {
                assert_eq!(fee.amount, to_managed_biguint(expected_ashswap_fee[i].amount.clone()));
                i += 1;
            }
        }
    ).assert_ok();
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.claim_protocol_fee(managed_address!(&protocol_address));
            sc.claim_ashswap_fee();
        }
    ).assert_ok();
    // balance of fee contract should be 0
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(0),
    );
    // balance of protocol and ashswap should be updated
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
}

#[test]
fn test_fee_accummulated() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let protocol_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    let ashswap_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.register_ashswap_fee(50_000u64, managed_address!(&ashswap_address)); // 50%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_address)); // 10%
            let ashswap_percent = sc.ashswap_fee_percent().get();
            assert_eq!(ashswap_percent, 50_000u64);
            let protocol_percent = sc.protocol_fee_percent(managed_address!(&protocol_address)).get();
            assert_eq!(protocol_percent, 10_000u64);
        }
    ).assert_ok();
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps.clone(), 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.claim_protocol_fee(managed_address!(&protocol_address));
            sc.claim_ashswap_fee();
        }
    ).assert_ok();
    // balance of fee contract should be 0
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(0),
    );
    // balance of protocol and ashswap should be updated
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10),
    );
}

#[test]
fn test_fee_accummulated_2_tokens() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let protocol_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    let ashswap_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.register_ashswap_fee(50_000u64, managed_address!(&ashswap_address)); // 50%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_address)); // 10%
            let ashswap_percent = sc.ashswap_fee_percent().get();
            assert_eq!(ashswap_percent, 50_000u64);
            let protocol_percent = sc.protocol_fee_percent(managed_address!(&protocol_address)).get();
            assert_eq!(protocol_percent, 10_000u64);
        }
    ).assert_ok();
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps.clone(), 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    let test_steps = vec![TestAggregatorStep {
        token_in: USDT_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];
    aggregate(
        &mut agg_setup,
        USDT_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDT_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();

    // check claimable amount
    let expected_protocol_fee = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10 / 2),
        },
    ];
    let expected_ashswap_fee = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount / 10 / 2),
        },
    ];
    agg_setup.blockchain_wrapper.execute_query(
        &agg_setup.fee_wrapper, 
        |sc| {
            let protocol_fee = sc.get_claimable_protocol_fee(managed_address!(&protocol_address), 0u64, 100u64);
            let mut i = 0;
            for fee in protocol_fee.into_iter() {
                assert_eq!(fee.amount, to_managed_biguint(expected_protocol_fee[i].amount.clone()));
                i += 1;
            }
            i = 0;
            let ashswap_fee = sc.get_claimable_ashswap_fee(0u64, 100u64);
            for fee in ashswap_fee.into_iter() {
                assert_eq!(fee.amount, to_managed_biguint(expected_ashswap_fee[i].amount.clone()));
                i += 1;
            }
        }
    ).assert_ok();

    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.claim_protocol_fee(managed_address!(&protocol_address));
            sc.claim_ashswap_fee();
        }
    ).assert_ok();
    
    // claimable amount must = 0
    agg_setup.blockchain_wrapper.execute_query(
        &agg_setup.fee_wrapper, 
        |sc| {
            let protocol_fee = sc.get_claimable_protocol_fee(managed_address!(&protocol_address), 0u64, 100u64);
            let mut i = 0;
            for fee in protocol_fee.into_iter() {
                assert_eq!(fee.amount, managed_biguint!(0));
                i += 1;
            }
            i = 0;
            let ashswap_fee = sc.get_claimable_ashswap_fee(0u64, 100u64);
            for fee in ashswap_fee.into_iter() {
                assert_eq!(fee.amount, managed_biguint!(0));
                i += 1;
            }
        }
    ).assert_ok();
    // balance of fee contract should be 0
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(0),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDT_TOKEN_ID,
        &rust_biguint!(0),
    );
    // balance of protocol and ashswap should be updated
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDT_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDT_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
}

#[test]
fn test_fee_1_esdt_1_egld() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let protocol_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let ashswap_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.register_ashswap_fee(50_000u64, managed_address!(&ashswap_address)); // 50%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_address)); // 10%
            let ashswap_percent = sc.ashswap_fee_percent().get();
            assert_eq!(ashswap_percent, 50_000u64);
            let protocol_percent = sc.protocol_fee_percent(managed_address!(&protocol_address)).get();
            assert_eq!(protocol_percent, 10_000u64);
        }
    ).assert_ok();
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps.clone(), 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();

    let test_steps = vec![TestAggregatorStep {
        token_in: b"EGLD".to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];
    aggregate(
        &mut agg_setup,
        b"EGLD",
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: b"EGLD".to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.claim_protocol_fee(managed_address!(&protocol_address));
            sc.claim_ashswap_fee();
        }
    ).assert_ok();
    // balance of fee contract should be 0
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &agg_setup.fee_wrapper.address_ref().clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(0),
    );
    // balance of protocol and ashswap should be updated
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_egld_balance(
        &protocol_address.clone(),
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_egld_balance(
        &protocol_address.clone(),
        &rust_biguint!(amount / 10 / 2),
    );
}

#[test]
fn test_fee_2_procotols() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
        fee::contract_obj,
    );
    let protocol_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    let protocol_2_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    let ashswap_address =
        agg_setup.blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.register_ashswap_fee(50_000u64, managed_address!(&ashswap_address)); // 50%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_address)); // 10%
            sc.register_protocol_fee(10_000u64, managed_address!(&protocol_2_address)); // 10%
        }
    ).assert_ok();
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount * 9 / 10), // charge 10% fee
        pool_address: mock_address.clone(),
    }];

    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps.clone(), 
        rust_biguint!(0), 
        Option::Some(&protocol_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    aggregate(
        &mut agg_setup,
        USDC_TOKEN_ID,
        BUSD_TOKEN_ID,
        test_steps, 
        rust_biguint!(0), 
        Option::Some(&protocol_2_address),
        TxTokenTransfer {
            token_identifier: USDC_TOKEN_ID.to_vec(),
            nonce: 0,
            value: rust_biguint!(amount),
        }
    ).assert_ok();
    agg_setup.blockchain_wrapper.execute_tx(
        &agg_setup.user_address, 
        &agg_setup.fee_wrapper, 
        &rust_biguint!(0), 
        |sc| {
            sc.claim_protocol_fee(managed_address!(&protocol_address));
            sc.claim_protocol_fee(managed_address!(&protocol_2_address));
            sc.claim_ashswap_fee();
        }
    ).assert_ok();
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &ashswap_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
    agg_setup.blockchain_wrapper.check_esdt_balance(
        &protocol_2_address.clone(),
        &USDC_TOKEN_ID,
        &rust_biguint!(amount / 10 / 2),
    );
}