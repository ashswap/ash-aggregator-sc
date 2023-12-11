multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getState)]
    fn get_state(&self) -> u8;

    #[view(getLpTokenSupply)]
    fn get_lp_token_supply(&self) -> BigUint;

    #[view(getBalances)]
    fn get_balances(&self) -> ManagedVec<BigUint>;

    #[view(getA)]
    fn get_a(&self) -> BigUint;

    #[view(getGamma)]
    fn get_gamma(&self) -> BigUint;

    #[view(getVirtualPrice)]
    fn get_virtual_price(&self) -> BigUint;

    #[view(getPriceOracle)]
    fn get_price_oracle(&self) -> BigUint;

    #[view(getPriceScale)]
    fn get_price_scale(&self) -> BigUint;

    #[view(getFutureAGammaTime)]
    fn get_future_a_gamma_time(&self) -> u64;

    #[view(getInitialAGammaTime)]
    fn get_initial_a_gamma_time(&self) -> u64;

    #[view(getD)]
    fn get_d(&self) -> BigUint;

    #[view(getMidFee)]
    fn get_mid_fee(&self) -> BigUint;

    #[view(getOutFee)]
    fn get_out_fee(&self) -> BigUint;

    #[view(getFeeGamma)]
    fn get_fee_gamma(&self) -> BigUint;

    #[view(getAllowedExtraProfit)]
    fn get_allowed_extra_profit(&self) -> BigUint;

    #[view(getAdjustmentStep)]
    fn get_adjustment_step(&self) -> BigUint;

    #[view(getAdminFee)]
    fn get_admin_fee(&self) -> BigUint;

    #[view(getLastPrices)]
    fn get_last_prices(&self) -> BigUint;

    #[view(getLastPriceTs)]
    fn get_last_price_ts(&self) -> u64;

    #[view(getMaHalfTime)]
    fn get_ma_half_time(&self) -> u64;

    #[view(getXcpProfit)]
    fn get_xcp_profit(&self) -> BigUint;

    #[view(getXcpProfitA)]
    fn get_xcp_profit_a(&self) -> BigUint;

    #[view(isNotAdjusted)]
    fn is_not_adjusted(&self) -> bool;

}
