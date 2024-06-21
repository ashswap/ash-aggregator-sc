/// Module: check_slippage
module check_slippage::check_slippage {
    use sui::coin::{Self, Coin};
    use std::type_name;
    use std::ascii::{String};

    const EInvalidCoin: u64 = 0;
    const ESlippage: u64 = 1;

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
