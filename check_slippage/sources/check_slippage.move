/// Module: check_slippage
module check_slippage::check_slippage {
    use sui::coin::{Self, Coin};
    use std::type_name;
    use std::ascii::{String};
    use sui::event;

    public struct CheckSlippage has copy, drop {
        sender: address,
        token_in: String,
        amount_in: u64,
        token_out: String,
        amount_out_min: u64,
        expect_amount_out: u64,
        actually_amount_out: u64,
        token_fee: String,
        fee_amount: u64,
    }

    const EInvalidCoin: u64 = 0;
    const ESlippage: u64 = 1;

    public fun check_slippage_v2<T>(token_in: String, amount_in: u64, swap_result: &Coin<T>, amount_out_min: u64, expect_amount_out: u64, token_fee: String, fee_amount: u64, ctx: &mut TxContext){
        assert!(coin::value(swap_result) >= amount_out_min, ESlippage);

        event::emit(CheckSlippage {
            sender: ctx.sender(),
            token_in: token_in,
            amount_in: amount_in,
            token_out: type_name::get<T>().into_string(), 
            amount_out_min: amount_out_min,
            expect_amount_out: expect_amount_out,
            actually_amount_out: swap_result.value(),
            token_fee: token_fee,
            fee_amount: fee_amount
        })
    }

    public fun check_slippage<T>(result: &Coin<T>, amount_out_min: u64) {
        assert!(coin::value(result) >= amount_out_min, ESlippage);
    }

    public fun check_slippage_and_type<T>(result: &Coin<T>, expect_coin_type: String, amount_out_min: u64) {
        let coin_type = type_name::get<T>().into_string();
        assert!(expect_coin_type == coin_type, EInvalidCoin);
        assert!(coin::value(result) >= amount_out_min, ESlippage);
    }


    public fun cut_remainder(arg0: u64, arg1: u64) : u64 {
        arg1 - arg1 % arg0
    }
}
