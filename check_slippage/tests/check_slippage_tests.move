
#[test_only]
module check_slippage::check_slippage_tests {
    use check_slippage::check_slippage::{Self};
    use sui::test_scenario::{Self};
    use sui::coin::{Self};
    use sui::sui::SUI;
    use check_slippage::my_coin::{MY_COIN};
    use check_slippage::another_my_coin::{ANOTHER_MY_COIN};
    use std::ascii::{Self};
    use std::type_name;

    #[test]
    fun test_check_slippage_with_sui_success() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<SUI>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min);
            transfer::public_transfer(result, user);
        };

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<SUI>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min - 1);
            transfer::public_transfer(result, user);
        };

        scenario.end();
    }

    #[test]
    #[expected_failure(abort_code = check_slippage::ESlippage)]
    fun test_check_slippage_with_sui_failed() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance + 1;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<SUI>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min);
            coin::burn_for_testing(result);
        };

        scenario.end();
    }

    #[test]
    fun test_check_slippage_any_token_success() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {   
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min);
            transfer::public_transfer(result, user);
        };

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min - 1);
            transfer::public_transfer(result, user);
        };

        scenario.end();
    }

    #[test]
    #[expected_failure(abort_code = check_slippage::ESlippage)]
    fun test_check_slippage_any_token_failed() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance + 1;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage(&result, amount_out_min);
            coin::burn_for_testing(result);
        };

        scenario.end();
    }

    #[test]
    fun test_check_slippage_and_type_sui_success() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<SUI>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result, ascii::string(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"), amount_out_min);
            transfer::public_transfer(result, user);
        };

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<SUI>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result,  ascii::string(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"), amount_out_min - 1);
            transfer::public_transfer(result, user);
        };

        scenario.end();
    }

    #[test]
    #[expected_failure(abort_code = check_slippage::EInvalidCoin)]
    fun test_check_slippage_and_type_sui_failed() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result, ascii::string(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"), amount_out_min);
            transfer::public_transfer(result, user);
        };

        scenario.end();
    }

     #[test]
    fun test_check_slippage_and_type_any_token_success() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {   
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result, ascii::string(b"0000000000000000000000000000000000000000000000000000000000000000::my_coin::MY_COIN"), amount_out_min);
            transfer::public_transfer(result, user);
        };

        scenario.next_tx(user);
        {
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result, ascii::string(b"0000000000000000000000000000000000000000000000000000000000000000::my_coin::MY_COIN"), amount_out_min - 1);
            transfer::public_transfer(result, user);
        };

        scenario.end();
    }

    #[test]
    #[expected_failure(abort_code = check_slippage::EInvalidCoin)]
    fun test_check_slippage_and_type_any_token_failed_sui() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {   
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            check_slippage::check_slippage_and_type(&result, ascii::string(b"0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"), amount_out_min);
            transfer::public_transfer(result, user);
        };
        scenario.end();
    }

    #[test]
    #[expected_failure(abort_code = check_slippage::EInvalidCoin)]
    fun test_check_slippage_and_type_any_token_failed_another_token() {
        let user: address = @0xAA;
        let result_balance: u64 = 1000;
        let amount_out_min = result_balance;

        let mut scenario = test_scenario::begin(user);

        scenario.next_tx(user);
        {   
            let result = coin::mint_for_testing<MY_COIN>(result_balance, scenario.ctx());
            let another_coin_type = type_name::get<ANOTHER_MY_COIN>().into_string();

            check_slippage::check_slippage_and_type(&result, another_coin_type, amount_out_min);
            transfer::public_transfer(result, user);
        };
        scenario.end();
    }
}
