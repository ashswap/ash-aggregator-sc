use aggregator::*;
use common_errors::*;
use common_structs::*;
use multiversx_sc::{codec::multi_types::OptionalValue, types::*};
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

const USDC_TOKEN_ID: &[u8] = b"USDC-abcdef";
const USDT_TOKEN_ID: &[u8] = b"USDT-abcdef";
const BUSD_TOKEN_ID: &[u8] = b"BUSD-abcdef";
const USER_TOTAL_TOKENS: u64 = 5_000_000_000;
const WRAPPED_EGLD_TOKEN_ID: &[u8] = b"WEGLD-abcdef";

struct AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub user_address: Address,
    pub mock_wrapper: ContractObjWrapper<protocol_mock::ContractObj<DebugApi>, ProtocolObjBuilder>,
    pub wrapper_wrapper: ContractObjWrapper<wrapper_mock::ContractObj<DebugApi>, WrapperObjBuilder>,
    pub agg_wrapper: ContractObjWrapper<aggregator::ContractObj<DebugApi>, AggregatorObjBuilder>,
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

fn setup_aggregator<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>(
    mock_builder: ProtocolObjBuilder,
    wrapper_mock_builder: WrapperObjBuilder,
    agg_builder: AggregatorObjBuilder,
) -> AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_addr = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address =
        blockchain_wrapper.create_user_account(&rust_biguint!(1_000_000_000_000_000_000));

    let mock_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
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

    blockchain_wrapper
        .execute_tx(&owner_addr, &mock_wrapper_wrapper, &rust_zero, |sc| {
            sc.init(WRAPPED_EGLD_TOKEN_ID.into());
        })
        .assert_ok();

    blockchain_wrapper
        .execute_tx(&owner_addr, &agg_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&mock_wrapper_wrapper.address_ref().clone()),
                WRAPPED_EGLD_TOKEN_ID.into(),
            );
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
    }
}

fn to_managed_biguint(value: RustBigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}

fn parse_biguint(str: &str) -> RustBigUint {
    let str_without_underscores = str.to_owned().replace("_", "");
    RustBigUint::parse_bytes(str_without_underscores.as_bytes(), 10).unwrap()
}

fn check_result<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>,
    expected_balances: Vec<TestTokenAmount>,
) where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    for expected in expected_balances {
        agg_setup.blockchain_wrapper.check_esdt_balance(
            &agg_setup.user_address,
            &expected.token,
            &expected.amount,
        );
    }
}

fn check_result_egld<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>,
    expected_balance: RustBigUint,
) where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    agg_setup
        .blockchain_wrapper
        .check_egld_balance(&agg_setup.user_address, &expected_balance);
}

fn aggregate<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>,
    test_steps: Vec<TestAggregatorStep>,
    test_limits: Vec<TestTokenAmount>,
    payments: Vec<TxTokenTransfer>,
) -> TxResult
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    agg_setup.blockchain_wrapper.execute_esdt_multi_transfer(
        &agg_setup.user_address,
        &agg_setup.agg_wrapper,
        &payments,
        |sc| {
            let mut steps = ManagedVec::new();
            for step in test_steps {
                let arguments = vec![managed_buffer!(&step.token_out)];
                steps.push(AggregatorStep {
                    token_in: managed_token_id!(step.token_in),
                    token_out: managed_token_id!(step.token_out),
                    amount_in: to_managed_biguint(step.amount_in),
                    pool_address: managed_address!(&step.pool_address),
                    function_name: managed_buffer!(b"exchange"),
                    arguments: ManagedVec::from(arguments),
                });
            }

            let mut limits = MultiValueEncoded::new();
            for limit in test_limits {
                limits.push(TokenAmount {
                    token: managed_token_id!(limit.token),
                    amount: to_managed_biguint(limit.amount),
                });
            }

            sc.aggregate(steps, limits);
        },
    )
}

fn aggregate_v2<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>(
    agg_setup: &mut AggregatorSetup<ProtocolObjBuilder, WrapperObjBuilder, AggregatorObjBuilder>,
    test_steps: Vec<TestAggregatorStep>,
    test_limits: Vec<TestTokenAmount>,
    egld_value: RustBigUint,
    payments: Vec<TxTokenTransfer>,
    return_egld: bool,
    protocol: OptionalValue<ManagedAddress<TxContextRef>>,
) -> TxResult
where
    ProtocolObjBuilder: 'static + Copy + Fn() -> protocol_mock::ContractObj<DebugApi>,
    WrapperObjBuilder: 'static + Copy + Fn() -> wrapper_mock::ContractObj<DebugApi>,
    AggregatorObjBuilder: 'static + Copy + Fn() -> aggregator::ContractObj<DebugApi>,
{
    if egld_value == rust_biguint!(0) {
        agg_setup.blockchain_wrapper.execute_esdt_multi_transfer(
            &agg_setup.user_address,
            &agg_setup.agg_wrapper,
            &payments,
            |sc| {
                let mut steps = ManagedVec::new();
                for step in test_steps {
                    let arguments = vec![managed_buffer!(&step.token_out)];
                    steps.push(AggregatorStep {
                        token_in: managed_token_id!(step.token_in),
                        token_out: managed_token_id!(step.token_out),
                        amount_in: to_managed_biguint(step.amount_in),
                        pool_address: managed_address!(&step.pool_address),
                        function_name: managed_buffer!(b"exchange"),
                        arguments: ManagedVec::from(arguments),
                    });
                }

                let mut limits = ManagedVec::new();
                for limit in test_limits {
                    limits.push(TokenAmount {
                        token: managed_token_id!(limit.token),
                        amount: to_managed_biguint(limit.amount),
                    });
                }

                sc.aggregate_esdt(steps, limits, return_egld, protocol);
            },
        )
    } else {
        agg_setup.blockchain_wrapper.execute_tx(
            &agg_setup.user_address,
            &agg_setup.agg_wrapper,
            &egld_value,
            |sc| {
                let mut steps = ManagedVec::new();
                for step in test_steps {
                    let arguments = vec![managed_buffer!(&step.token_out)];
                    steps.push(AggregatorStep {
                        token_in: managed_token_id!(step.token_in),
                        token_out: managed_token_id!(step.token_out),
                        amount_in: to_managed_biguint(step.amount_in),
                        pool_address: managed_address!(&step.pool_address),
                        function_name: managed_buffer!(b"exchange"),
                        arguments: ManagedVec::from(arguments),
                    });
                }

                let mut limits = ManagedVec::new();
                for limit in test_limits {
                    limits.push(TokenAmount {
                        token: managed_token_id!(limit.token),
                        amount: to_managed_biguint(limit.amount),
                    });
                }

                sc.aggregate_egld(steps, limits, protocol);
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
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let test_limits = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
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

    aggregate(&mut agg_setup, test_steps, test_limits, payments).assert_ok();
    check_result(&mut agg_setup, expected_balances);
}

#[test]
fn test_aggregate_simple_with_egld_return() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: WRAPPED_EGLD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let test_limits = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
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
            token: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS),
        },
    ];
    check_result_egld(&mut agg_setup, rust_biguint!(1_000_000_000_000_000_000));
    aggregate_v2(
        &mut agg_setup,
        test_steps,
        test_limits,
        rust_biguint!(0),
        payments,
        true,
        OptionalValue::None,
    )
    .assert_ok();
    check_result(&mut agg_setup, expected_balances);
    check_result_egld(
        &mut agg_setup,
        rust_biguint!(1_000_000_000_000_000_000 + amount * 95 / 100),
    )
}

#[test]
fn test_aggregate_simple_with_egld_input() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![TestAggregatorStep {
        token_in: WRAPPED_EGLD_TOKEN_ID.to_vec(),
        token_out: USDC_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let test_limits = vec![
        TestTokenAmount {
            token: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
    ];

    let payments = vec![];

    let expected_balances = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS + amount * 95 / 100),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS),
        },
        TestTokenAmount {
            token: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(USER_TOTAL_TOKENS),
        },
    ];
    check_result_egld(&mut agg_setup, rust_biguint!(1_000_000_000_000_000_000));
    aggregate_v2(
        &mut agg_setup,
        test_steps,
        test_limits,
        rust_biguint!(amount),
        payments,
        true,
        OptionalValue::None,
    )
    .assert_ok();
    check_result(&mut agg_setup, expected_balances);
    check_result_egld(
        &mut agg_setup,
        rust_biguint!(1_000_000_000_000_000_000 - amount),
    )
}

#[test]
fn test_aggregate_error() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
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

    let payments = vec![TxTokenTransfer {
        token_identifier: USDT_TOKEN_ID.to_vec(), // change it
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, vec![], payments)
        .assert_user_error(ERROR_INVALID_TOKEN_IN);

    // invalid amount in
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount + 1), // change it
        pool_address: mock_address.clone(),
    }];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, vec![], payments)
        .assert_user_error(ERROR_INVALID_AMOUNT_IN);

    // slippage
    let test_steps = vec![TestAggregatorStep {
        token_in: USDC_TOKEN_ID.to_vec(),
        token_out: BUSD_TOKEN_ID.to_vec(),
        amount_in: rust_biguint!(amount),
        pool_address: mock_address.clone(),
    }];

    let test_limits = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(amount),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, test_limits, payments)
        .assert_user_error(ERROR_SLIPPAGE_SCREW_YOU);
}

#[test]
fn test_aggregate_error_invalid_token_in_step2() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid token_in in step 2
    let test_steps = vec![
        TestAggregatorStep {
            token_in: USDT_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount),
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: USDC_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount),
            pool_address: mock_address.clone(),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDT_TOKEN_ID.to_vec(), // change it
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, vec![], payments)
        .assert_user_error(ERROR_INVALID_TOKEN_IN);
}

#[test]
fn test_aggregate_error_invalid_token_in_step3() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    // invalid token_in in step 3, ...
    let test_steps = vec![
        TestAggregatorStep {
            token_in: USDT_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount),
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: BUSD_TOKEN_ID.to_vec(),
            token_out: USDC_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount * 95 / 100),
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: WRAPPED_EGLD_TOKEN_ID.to_vec(),
            token_out: USDC_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount),
            pool_address: mock_address.clone(),
        },
    ];

    let test_limits = vec![
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDT_TOKEN_ID.to_vec(), // change it
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, test_limits, payments)
        .assert_user_error(ERROR_INVALID_TOKEN_IN);
}

#[test]
fn test_aggregate_error_invalid_amount_in_step2() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
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
            amount_in: rust_biguint!(amount), // change it
            pool_address: mock_address.clone(),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, vec![], payments)
        .assert_user_error(ERROR_INVALID_AMOUNT_IN);
}

#[test]
fn test_aggregate_error_invalid_amount_in_step3() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
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

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    aggregate(&mut agg_setup, test_steps, vec![], payments)
        .assert_user_error(ERROR_INVALID_AMOUNT_IN);
}

#[test]
fn test_aggregate_multi() {
    let mut agg_setup = setup_aggregator(
        protocol_mock::contract_obj,
        wrapper_mock::contract_obj,
        aggregator::contract_obj,
    );
    let mock_address = agg_setup.mock_wrapper.address_ref().clone();
    let amount = 1_000_000;

    let test_steps = vec![
        TestAggregatorStep {
            token_in: USDC_TOKEN_ID.to_vec(),
            token_out: USDT_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount * 40 / 100),
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: USDT_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(0),
            pool_address: mock_address.clone(),
        },
        TestAggregatorStep {
            token_in: USDC_TOKEN_ID.to_vec(),
            token_out: BUSD_TOKEN_ID.to_vec(),
            amount_in: rust_biguint!(amount * 60 / 100),
            pool_address: mock_address.clone(),
        },
    ];

    let test_limits = vec![
        TestTokenAmount {
            token: USDC_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: USDT_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
        TestTokenAmount {
            token: BUSD_TOKEN_ID.to_vec(),
            amount: rust_biguint!(0),
        },
    ];

    let payments = vec![TxTokenTransfer {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(amount),
    }];

    let expected_amount = amount * 40 / 100 * 95 / 100 * 95 / 100 + amount * 60 / 100 * 95 / 100;
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
            amount: rust_biguint!(USER_TOTAL_TOKENS + expected_amount),
        },
    ];

    aggregate(&mut agg_setup, test_steps, test_limits, payments).assert_ok();
    check_result(&mut agg_setup, expected_balances);
}
