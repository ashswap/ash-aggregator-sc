multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getState)]
    fn get_state(&self) -> u8;

    #[view(getTokens)]
    fn get_tokens(&self) -> ManagedVec<TokenIdentifier>;

    #[view(getBalances)]
    fn get_balances(&self, token: &TokenIdentifier) -> BigUint;

    #[view(getTokenPrice)]
    fn get_token_price(&self, token: &TokenIdentifier, precision: &BigUint) -> BigUint;

    #[view(getAmpFactor)]
    fn get_amp_factor(&self) -> u64;

    #[view(getSwapFeePercent)]
    fn swap_fee_percent(&self) -> u64;
}
