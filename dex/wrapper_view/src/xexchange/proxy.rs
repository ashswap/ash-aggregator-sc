multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getState)]
    fn get_state(&self) -> u8;

    #[view(getTotalFeePercent)]
    fn get_total_fee_percent(&self) -> u64;

    #[view(getReserve)]
    fn get_reserve(&self, token_id: TokenIdentifier) -> BigUint;
}
